use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  ingress::{_PartialIngressInstanceConfig, IngressInstance},
  update::Update,
};

use super::KomodoWriteRequest;

//

/// Create an ingress instance. Response: [IngressInstance].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(IngressInstance)]
#[error(serror::Error)]
pub struct CreateIngressInstance {
  /// The name given to newly created ingress instance.
  pub name: String,
  /// Optional partial config to initialize the instance with.
  #[serde(default)]
  pub config: _PartialIngressInstanceConfig,
}

//

/// Creates a new ingress instance with given `name` and the configuration
/// of the instance at the given `id`. Response: [IngressInstance].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(IngressInstance)]
#[error(serror::Error)]
pub struct CopyIngressInstance {
  /// The name of the new ingress instance.
  pub name: String,
  /// The id of the ingress instance to copy.
  pub id: String,
}

//

/// Deletes the ingress instance at the given id, and returns the deleted instance.
/// Response: [IngressInstance]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(IngressInstance)]
#[error(serror::Error)]
pub struct DeleteIngressInstance {
  /// The id or name of the ingress instance to delete.
  pub id: String,
}

//

/// Update the ingress instance at the given id, and return the updated instance. Response: [IngressInstance].
///
/// Note. This method updates only the fields which are set in the [PartialIngressInstanceConfig][crate::entities::ingress::PartialIngressInstanceConfig],
/// effectively merging diffs into the final document. This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(IngressInstance)]
#[error(serror::Error)]
pub struct UpdateIngressInstance {
  /// The id of the ingress instance to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialIngressInstanceConfig,
}

//

/// Rename the IngressInstance at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RenameIngressInstance {
  /// The id or name of the IngressInstance to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}
