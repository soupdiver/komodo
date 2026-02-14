use std::collections::HashMap;

use komodo_client::entities::{docker::container::Port, server::Server};
use serde_json::{Map, Value, json};

use super::labels;

/// Build the Traefik HTTP provider JSON configuration from a list of container labels.
/// Each container contributes routers and services.
/// Services with the same name are merged (multiple containers -> multiple servers in loadBalancer).
pub fn build(
  container_labels_list: &[HashMap<String, String>],
) -> Value {
  let mut all_routers: Map<String, Value> = Map::new();
  let mut all_services: Map<String, Value> = Map::new();

  for labels in container_labels_list {
    let parsed = labels::parse_traefik_labels(labels);

    // Merge routers
    for (router_name, router_config) in parsed.routers {
      all_routers
        .insert(router_name, Value::Object(router_config));
    }

    // Merge services - handle loadBalancer.servers specially
    for (service_name, service_config) in parsed.services {
      // Resolve backend URL
      let backend_url = resolve_backend_url_from_labels(labels);

      // Get or create the service entry
      let service_entry = all_services
        .entry(service_name.clone())
        .or_insert_with(|| {
          json!({
            "loadBalancer": {
              "servers": []
            }
          })
        });

      // Merge loadBalancer config
      if let Some(lb) = service_config.get("loadBalancer") {
        if let Value::Object(service_obj) = service_entry {
          service_obj.insert("loadBalancer".to_string(), lb.clone());
        }
      }

      // Add the backend URL to servers array
      if let Value::Object(service_obj) = service_entry {
        if let Some(Value::Object(lb)) =
          service_obj.get_mut("loadBalancer")
        {
          let servers = lb
            .entry("servers".to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
          if let Value::Array(servers_arr) = servers {
            servers_arr.push(json!({ "url": backend_url }));
          }
        }
      }
    }
  }

  json!({
    "http": {
      "routers": all_routers,
      "services": all_services
    }
  })
}

/// Resolve the backend URL from container labels.
/// Labels include special keys:
/// - `__komodo_server_ip`: The server IP
/// - `__komodo_container_ports`: JSON array of Port objects
/// - `traefik.http.services.<name>.loadbalancer.server.port`: The container port to use
fn resolve_backend_url_from_labels(
  labels: &HashMap<String, String>,
) -> String {
  let server_ip = labels
    .get("__komodo_server_ip")
    .map(|s| s.as_str())
    .unwrap_or("localhost");

  let ports_json = labels
    .get("__komodo_container_ports")
    .and_then(|s| serde_json::from_str::<Vec<Port>>(s).ok())
    .unwrap_or_default();

  // Try to find the port from labels
  // Look for traefik.http.services.<name>.loadbalancer.server.port
  let mut container_port: Option<u16> = None;

  for (key, value) in labels {
    if key.contains("loadbalancer.server.port") {
      if let Ok(port) = value.parse::<u16>() {
        container_port = Some(port);
        break;
      }
    }
  }

  // If port not specified, try to use the first exposed port
  if container_port.is_none() && !ports_json.is_empty() {
    container_port = Some(ports_json[0].private_port);
  }

  let Some(container_port_val) = container_port else {
    return format!("http://{}:<unknown>", server_ip);
  };

  // Find the published host port
  let host_port = ports_json
    .iter()
    .find(|p| p.private_port == container_port_val)
    .and_then(|p| p.public_port);

  let Some(host_port_val) = host_port else {
    warn!(
      "Container port {} not published to host for backend URL resolution",
      container_port_val
    );
    return format!("http://{}:<unpublished>", server_ip);
  };

  format!("http://{}:{}", server_ip, host_port_val)
}

/// Resolve the server IP for backend URL construction.
/// Prefers external_address if set, otherwise uses the address field.
pub fn resolve_server_ip(server: &Server) -> String {
  let addr = if !server.config.external_address.is_empty() {
    &server.config.external_address
  } else {
    &server.config.address
  };

  strip_protocol_and_port(addr)
}

/// Strip protocol and port from an address string.
/// "http://10.0.0.5:8120" -> "10.0.0.5"
fn strip_protocol_and_port(addr: &str) -> String {
  addr
    .trim_start_matches("http://")
    .trim_start_matches("https://")
    .split(':')
    .next()
    .unwrap_or("localhost")
    .to_string()
}
