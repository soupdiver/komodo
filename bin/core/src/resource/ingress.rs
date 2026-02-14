use database::mungos::mongodb::Collection;
use komodo_client::entities::{
  Operation, ResourceTarget, ResourceTargetVariant,
  ingress::{
    IngressInstance, IngressInstanceConfig, IngressInstanceConfigDiff,
    IngressInstanceListItem, IngressInstanceListItemInfo,
    IngressInstanceQuerySpecifics, PartialIngressInstanceConfig,
  },
  resource::Resource,
  update::Update,
  user::User,
};

use crate::state::db_client;

impl super::KomodoResource for IngressInstance {
  type Config = IngressInstanceConfig;
  type PartialConfig = PartialIngressInstanceConfig;
  type ConfigDiff = IngressInstanceConfigDiff;
  type Info = ();
  type ListItem = IngressInstanceListItem;
  type QuerySpecifics = IngressInstanceQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::IngressInstance
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::IngressInstance(id.into())
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().ingress_instances
  }

  async fn to_list_item(
    instance: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    // Count routes for this instance from the ingress cache
    let route_count = crate::state::ingress_cache()
      .load()
      .get(&instance.name)
      .map(|state| state.routes.len() as u32)
      .unwrap_or(0);

    IngressInstanceListItem {
      name: instance.name,
      id: instance.id,
      template: instance.template,
      tags: instance.tags,
      resource_type: ResourceTargetVariant::IngressInstance,
      info: IngressInstanceListItemInfo {
        enabled: instance.config.enabled,
        is_default: instance.config.is_default,
        route_count,
      },
    }
  }

  async fn busy(_id: &String) -> anyhow::Result<bool> {
    Ok(false)
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateIngressInstance
  }

  fn user_can_create(user: &User) -> bool {
    user.admin
  }

  async fn validate_create_config(
    config: &mut Self::PartialConfig,
    _user: &User,
  ) -> anyhow::Result<()> {
    use database::mungos::mongodb::bson::doc;

    // If is_default is being set to true, ensure no other instance has is_default=true
    if config.is_default == Some(true) {
      let existing_default = Self::coll()
        .find_one(doc! { "config.is_default": true })
        .await?;
      if let Some(default_instance) = existing_default {
        anyhow::bail!(
          "An ingress instance with is_default=true already exists: '{}'",
          default_instance.name
        );
      }
    }
    Ok(())
  }

  async fn post_create(
    _created: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    crate::ingress::rebuild_ingress_cache().await;
    Ok(())
  }

  // UPDATE

  fn update_operation() -> Operation {
    Operation::UpdateIngressInstance
  }

  async fn validate_update_config(
    id: &str,
    config: &mut Self::PartialConfig,
    _user: &User,
  ) -> anyhow::Result<()> {
    use database::mungos::mongodb::bson::doc;

    // If is_default is being set to true, ensure no other instance has is_default=true
    if config.is_default == Some(true) {
      let existing_default = Self::coll()
        .find_one(doc! {
          "config.is_default": true,
          "id": { "$ne": id }
        })
        .await?;
      if let Some(default_instance) = existing_default {
        anyhow::bail!(
          "An ingress instance with is_default=true already exists: '{}'. Set its is_default to false first.",
          default_instance.name
        );
      }
    }
    Ok(())
  }

  async fn post_update(
    _updated: &Self,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    crate::ingress::rebuild_ingress_cache().await;
    Ok(())
  }

  // RENAME

  fn rename_operation() -> Operation {
    Operation::RenameIngressInstance
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteIngressInstance
  }

  async fn pre_delete(
    _resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn post_delete(
    _resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    crate::ingress::rebuild_ingress_cache().await;
    Ok(())
  }
}
