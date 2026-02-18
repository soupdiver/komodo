use bson::{Document, doc};
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::resource::{Resource, ResourceListItem, ResourceQuery};

#[typeshare]
pub type IngressInstance = Resource<IngressInstanceConfig, ()>;

#[typeshare]
pub type IngressInstanceListItem =
  ResourceListItem<IngressInstanceListItemInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IngressInstanceListItemInfo {
  /// Whether ingress instance is enabled
  pub enabled: bool,
  /// Whether this is the default instance for containers without explicit instance label
  pub is_default: bool,
  /// Number of routes managed by this instance
  pub route_count: u32,
}

#[typeshare(serialized_as = "Partial<IngressInstanceConfig>")]
pub type _PartialIngressInstanceConfig = PartialIngressInstanceConfig;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder, Partial)]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[partial(skip_serializing_none, from, diff)]
pub struct IngressInstanceConfig {
  /// Whether the ingress instance is enabled
  #[serde(default)]
  #[builder(default)]
  pub enabled: bool,

  /// Whether this is the default instance for containers without explicit instance label.
  /// Only one instance can be default.
  #[serde(default)]
  #[builder(default)]
  pub is_default: bool,

  /// The server this ingress instance's Traefik runs on (server id).
  /// When a target container is on this same server, backend URLs use
  /// host.docker.internal instead of the server's address.
  #[serde(default)]
  #[builder(default)]
  pub server_id: String,
}

impl IngressInstanceConfig {
  pub fn builder() -> IngressInstanceConfigBuilder {
    IngressInstanceConfigBuilder::default()
  }
}

#[allow(clippy::derivable_impls)]
impl Default for IngressInstanceConfig {
  fn default() -> Self {
    Self {
      enabled: Default::default(),
      is_default: Default::default(),
      server_id: Default::default(),
    }
  }
}

// QUERY
#[typeshare]
pub type IngressInstanceQuery =
  ResourceQuery<IngressInstanceQuerySpecifics>;

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, DefaultBuilder,
)]
pub struct IngressInstanceQuerySpecifics {
  /// Filter ingress instances by enabled.
  /// - `None`: Don't filter by enabled
  /// - `Some(true)`: Only include instances with `enabled: true`
  /// - `Some(false)`: Only include instances with `enabled: false`
  pub enabled: Option<bool>,
}

impl super::resource::AddFilters for IngressInstanceQuerySpecifics {
  fn add_filters(&self, filters: &mut Document) {
    if let Some(enabled) = self.enabled {
      filters.insert("config.enabled", enabled);
    }
  }
}

// INGRESS ROUTES (for dashboard display)
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IngressRoute {
  /// The server name where the container is running
  pub server_name: String,
  /// The container name
  pub container_name: String,
  /// The Traefik router name (from labels)
  pub router_name: String,
  /// The routing rule (e.g., "Host(`example.com`)")
  pub rule: String,
  /// The service name (from labels)
  pub service_name: String,
  /// The backend URL (resolved IP:port)
  pub backend_url: String,
  /// The entrypoints (e.g., ["web", "websecure"])
  pub entrypoints: Vec<String>,
}

// CACHED STATE (internal, not typeshare'd - used only in Core)
#[derive(Debug, Clone)]
pub struct CachedIngressState {
  /// List of routes for dashboard display
  pub routes: Vec<IngressRoute>,
  /// The Traefik dynamic config JSON
  pub traefik_config: serde_json::Value,
}
