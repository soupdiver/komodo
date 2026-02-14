use std::collections::HashMap;

use serde_json::{Map, Value};

/// Result of parsing Traefik labels
pub struct ParsedLabels {
  pub routers: HashMap<String, Map<String, Value>>,
  pub services: HashMap<String, Map<String, Value>>,
}

/// Parse Traefik labels into a structured format.
/// Labels like "traefik.http.routers.myapp.rule" = "Host(`x.com`)"
/// are converted into nested JSON objects.
pub fn parse_traefik_labels(
  labels: &HashMap<String, String>,
) -> ParsedLabels {
  let mut routers: HashMap<String, Map<String, Value>> = HashMap::new();
  let mut services: HashMap<String, Map<String, Value>> = HashMap::new();

  for (key, value) in labels {
    // Skip non-traefik labels
    if !key.starts_with("traefik.") {
      continue;
    }

    // Strip "traefik." prefix
    let stripped = key.strip_prefix("traefik.").unwrap();
    let parts: Vec<&str> = stripped.split('.').collect();

    if parts.len() < 3 {
      // Invalid label format
      continue;
    }

    // parts[0] = "http" or "tcp" or "udp"
    // parts[1] = "routers" or "services" or "middlewares"
    // parts[2] = router/service/middleware name
    // parts[3..] = nested keys

    let protocol = parts[0];
    let resource_type = parts[1];
    let resource_name = parts[2];

    // Only handle http for now
    if protocol != "http" {
      continue;
    }

    match resource_type {
      "routers" => {
        let router_map = routers.entry(resource_name.to_string()).or_default();
        insert_nested_value(router_map, &parts[3..], value);
      }
      "services" => {
        let service_map =
          services.entry(resource_name.to_string()).or_default();
        insert_nested_value(service_map, &parts[3..], value);
      }
      _ => {
        // Skip middlewares, etc. for now
      }
    }
  }

  ParsedLabels { routers, services }
}

/// Insert a value into a nested JSON object based on a key path.
/// For example, ["tls", "certresolver"] with value "letsencrypt"
/// becomes: {"tls": {"certResolver": "letsencrypt"}}
fn insert_nested_value(
  map: &mut Map<String, Value>,
  keys: &[&str],
  value: &str,
) {
  if keys.is_empty() {
    return;
  }

  if keys.len() == 1 {
    // Leaf value - handle special cases
    let key = normalize_key(keys[0]);
    map.insert(key, parse_value(keys[0], value));
    return;
  }

  // Nested path - traverse/create nested objects
  let key = normalize_key(keys[0]);
  let nested_map =
    map.entry(key).or_insert_with(|| Value::Object(Map::new()));

  if let Value::Object(nested) = nested_map {
    insert_nested_value(nested, &keys[1..], value);
  }
}

/// Normalize Traefik label keys to match JSON field names.
/// Traefik uses lowercase with dots, JSON API uses camelCase for some fields.
fn normalize_key(key: &str) -> String {
  match key {
    "certresolver" => "certResolver".to_string(),
    "loadbalancer" => "loadBalancer".to_string(),
    "passhostheader" => "passHostHeader".to_string(),
    _ => key.to_string(),
  }
}

/// Parse a label value, handling special cases.
fn parse_value(key: &str, value: &str) -> Value {
  match key {
    // entrypoints: "web,websecure" -> ["web", "websecure"]
    "entrypoints" => Value::Array(
      value
        .split(',')
        .map(|s| Value::String(s.trim().to_string()))
        .collect(),
    ),
    // middlewares: "auth,ratelimit" -> ["auth", "ratelimit"]
    "middlewares" => Value::Array(
      value
        .split(',')
        .map(|s| Value::String(s.trim().to_string()))
        .collect(),
    ),
    // tls: "true" -> {}
    "tls" if value == "true" => Value::Object(Map::new()),
    // default: string value
    _ => Value::String(value.to_string()),
  }
}
