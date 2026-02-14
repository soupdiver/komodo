use anyhow::Context;
use database::mongo_indexed::Document;
use database::mungos::mongodb::bson::doc;
use komodo_client::{
  api::read::*,
  entities::{
    ingress::{IngressInstance, IngressInstanceListItem, IngressRoute},
    permission::PermissionLevel,
  },
};
use resolver_api::Resolve;

use crate::{
  helpers::query::get_all_tags, permission::get_check_permissions,
  resource, state::db_client,
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetIngressInstance {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<IngressInstance> {
    Ok(
      get_check_permissions::<IngressInstance>(
        &self.ingress_instance,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListIngressInstances {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<IngressInstanceListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<IngressInstance>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullIngressInstances {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullIngressInstancesResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<IngressInstance>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetIngressInstancesSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetIngressInstancesSummaryResponse> {
    let query = match resource::get_resource_object_ids_for_user::<
      IngressInstance,
    >(user)
    .await?
    {
      Some(ids) => doc! {
        "_id": { "$in": ids }
      },
      None => Document::new(),
    };
    let total = db_client()
      .ingress_instances
      .count_documents(query)
      .await
      .context("failed to count all ingress instance documents")?;
    let res = GetIngressInstancesSummaryResponse {
      total: total as u32,
    };
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListIngressRoutes {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<IngressRoute>> {
    // Verify user has read access to the instance
    let instance = get_check_permissions::<IngressInstance>(
      &self.instance,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;

    // Get routes from the ingress cache
    let routes = crate::state::ingress_cache()
      .load()
      .get(&instance.name)
      .map(|state| state.routes.clone())
      .unwrap_or_default();

    Ok(routes)
  }
}
