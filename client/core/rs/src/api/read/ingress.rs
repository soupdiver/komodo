use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::ingress::{
  IngressInstance, IngressInstanceListItem, IngressInstanceQuery,
  IngressRoute,
};

use super::KomodoReadRequest;

//

/// Get a specific ingress instance. Response: [IngressInstance].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetIngressInstanceResponse)]
#[error(serror::Error)]
pub struct GetIngressInstance {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub ingress_instance: String,
}

#[typeshare]
pub type GetIngressInstanceResponse = IngressInstance;

//

/// List ingress instances matching optional query. Response: [ListIngressInstancesResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListIngressInstancesResponse)]
#[error(serror::Error)]
pub struct ListIngressInstances {
  /// Structured query to filter ingress instances.
  #[serde(default)]
  pub query: IngressInstanceQuery,
}

#[typeshare]
pub type ListIngressInstancesResponse = Vec<IngressInstanceListItem>;

/// List full ingress instances matching optional query. Response: [ListFullIngressInstancesResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullIngressInstancesResponse)]
#[error(serror::Error)]
pub struct ListFullIngressInstances {
  /// Structured query to filter ingress instances.
  #[serde(default)]
  pub query: IngressInstanceQuery,
}

#[typeshare]
pub type ListFullIngressInstancesResponse = Vec<IngressInstance>;

//

/// Gets a summary of data relating to all ingress instances.
/// Response: [GetIngressInstancesSummaryResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetIngressInstancesSummaryResponse)]
#[error(serror::Error)]
pub struct GetIngressInstancesSummary {}

/// Response for [GetIngressInstancesSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetIngressInstancesSummaryResponse {
  pub total: u32,
}

//

/// List all ingress routes for a specific instance. Response: [ListIngressRoutesResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListIngressRoutesResponse)]
#[error(serror::Error)]
pub struct ListIngressRoutes {
  /// The id or name of the ingress instance.
  pub instance: String,
}

#[typeshare]
pub type ListIngressRoutesResponse = Vec<IngressRoute>;
