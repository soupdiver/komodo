mod labels;
mod traefik_config;

use std::{collections::HashMap, sync::Arc};

use axum::{extract::Path, routing::get, Json, Router};
use futures::TryStreamExt;
use komodo_client::entities::{
  ingress::{CachedIngressState, IngressRoute},
  server::Server,
};

use crate::state::{db_client, ingress_cache, server_status_cache};

/// Returns an unauthenticated Axum router for the /ingress endpoints.
/// Traefik polls these endpoints to get dynamic configuration.
pub fn router() -> Router {
  Router::new()
    .route("/{instance}/config", get(traefik_config_handler))
}

/// GET /ingress/{instance}/config
/// Returns the Traefik dynamic config JSON for the specified instance.
async fn traefik_config_handler(
  Path(instance): Path<String>,
) -> Json<serde_json::Value> {
  let cache = ingress_cache().load();
  match cache.get(&instance) {
    Some(state) => Json(state.traefik_config.clone()),
    None => {
      // Return empty config if instance not found
      Json(serde_json::json!({
        "http": {
          "routers": {},
          "services": {}
        }
      }))
    }
  }
}

/// Rebuild the entire ingress cache by:
/// 1. Loading all IngressInstance resources from DB
/// 2. Finding the default instance
/// 3. Iterating all containers with ingress_labels
/// 4. Parsing labels and resolving backend URLs
/// 5. Grouping routes by instance
/// 6. Building Traefik JSON for each instance
/// 7. Storing in the global cache
pub async fn rebuild_ingress_cache() {
  use database::mongo_indexed::Document;

  // Load all ingress instances
  let instances = match db_client()
    .ingress_instances
    .find(Document::new())
    .await
  {
    Ok(cursor) => match cursor.try_collect::<Vec<_>>().await {
      Ok(instances) => instances,
      Err(e) => {
        warn!("failed to collect ingress instances: {e:#}");
        return;
      }
    },
    Err(e) => {
      warn!("failed to load ingress instances: {e:#}");
      return;
    }
  };

  // Find the default instance (if any)
  let default_instance = instances
    .iter()
    .find(|i| i.config.is_default)
    .map(|i| i.name.clone());

  // Build a map from instance name -> (enabled, routes, traefik_config)
  let mut instance_routes: HashMap<String, Vec<IngressRoute>> =
    HashMap::new();

  // Get all servers to resolve IPs
  let servers = match db_client().servers.find(Document::new()).await {
    Ok(cursor) => match cursor.try_collect::<Vec<_>>().await {
      Ok(servers) => servers,
      Err(e) => {
        warn!("failed to collect servers: {e:#}");
        return;
      }
    },
    Err(e) => {
      warn!("failed to load servers for ingress: {e:#}");
      return;
    }
  };
  let server_map: HashMap<String, Server> =
    servers.into_iter().map(|s| (s.id.clone(), s)).collect();

  // Iterate server status cache to find containers with ingress_labels
  let server_status_list = server_status_cache().get_list().await;

  for status in &server_status_list {
    let Some(server) = server_map.get(&status.id) else {
      continue;
    };

    let server_ip = traefik_config::resolve_server_ip(server);

    let Some(containers) = &status.containers else {
      continue;
    };

    for container in containers {
      if container.ingress_labels.is_empty() {
        continue;
      }

      // Parse labels to determine which instance this belongs to
      let instance_name = container
        .ingress_labels
        .get("komodo.ingress.instance")
        .cloned()
        .or_else(|| default_instance.clone());

      let Some(instance_name) = instance_name else {
        // No explicit instance and no default -> skip
        continue;
      };

      // Parse Traefik labels
      let parsed_labels =
        labels::parse_traefik_labels(&container.ingress_labels);

      // Extract router and service info
      for (router_name, router_labels) in &parsed_labels.routers {
        let rule = router_labels
          .get("rule")
          .and_then(|v| v.as_str())
          .unwrap_or("")
          .to_string();

        let entrypoints = router_labels
          .get("entrypoints")
          .and_then(|v| v.as_array())
          .map(|arr| {
            arr
              .iter()
              .filter_map(|v| v.as_str().map(String::from))
              .collect()
          })
          .unwrap_or_default();

        // Get the service name from router
        let service_name = router_labels
          .get("service")
          .and_then(|v| v.as_str())
          .map(String::from)
          .unwrap_or_else(|| router_name.clone());

        // Resolve backend URL from service.loadBalancer.servers
        let backend_url =
          if let Some(service_obj) = parsed_labels.services.get(&service_name)
          {
            service_obj
              .get("loadBalancer")
              .and_then(|lb| lb.as_object())
              .and_then(|lb| lb.get("servers"))
              .and_then(|servers| servers.as_array())
              .and_then(|servers| servers.first())
              .and_then(|server| server.as_object())
              .and_then(|server| server.get("url"))
              .and_then(|url| url.as_str())
              .map(String::from)
          } else {
            None
          };

        let backend_url = backend_url.unwrap_or_else(|| {
          format!("http://{}:<unknown>", server_ip)
        });

        let route = IngressRoute {
          server_name: server.name.clone(),
          container_name: container.name.clone(),
          router_name: router_name.clone(),
          rule,
          service_name,
          backend_url,
          entrypoints,
        };

        instance_routes
          .entry(instance_name.clone())
          .or_default()
          .push(route);
      }
    }
  }

  // Build the final cache: for each instance, generate Traefik JSON
  let mut new_cache: HashMap<String, CachedIngressState> = HashMap::new();

  for instance in instances {
    if !instance.config.enabled {
      // Skip disabled instances
      continue;
    }

    let routes = instance_routes
      .remove(&instance.name)
      .unwrap_or_default();

    // Rebuild container labels map for this instance
    let mut container_labels_list: Vec<HashMap<String, String>> =
      Vec::new();

    for status in &server_status_list {
      let Some(server) = server_map.get(&status.id) else {
        continue;
      };
      let server_ip = traefik_config::resolve_server_ip(server);

      let Some(containers) = &status.containers else {
        continue;
      };

      for container in containers {
        if container.ingress_labels.is_empty() {
          continue;
        }

        let container_instance_name = container
          .ingress_labels
          .get("komodo.ingress.instance")
          .cloned()
          .or_else(|| default_instance.clone());

        if container_instance_name.as_deref() != Some(&instance.name) {
          continue;
        }

        // Resolve backend URL and inject into labels
        let mut labels_with_backend =
          container.ingress_labels.clone();

        // We need the server IP and container ports for backend resolution
        labels_with_backend.insert(
          "__komodo_server_ip".to_string(),
          server_ip.clone(),
        );
        labels_with_backend.insert(
          "__komodo_container_ports".to_string(),
          serde_json::to_string(&container.ports).unwrap_or_default(),
        );

        container_labels_list.push(labels_with_backend);
      }
    }

    let traefik_config =
      traefik_config::build(&container_labels_list);

    new_cache.insert(
      instance.name.clone(),
      CachedIngressState {
        routes,
        traefik_config,
      },
    );
  }

  ingress_cache().store(Arc::new(new_cache));
}
