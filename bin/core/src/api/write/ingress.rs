use komodo_client::{
  api::write::*,
  entities::{
    ingress::IngressInstance, permission::PermissionLevel,
    update::Update,
  },
};
use resolver_api::Resolve;

use crate::{permission::get_check_permissions, resource};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateIngressInstance {
  #[instrument(name = "CreateIngressInstance", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<IngressInstance> {
    resource::create::<IngressInstance>(&self.name, self.config, user)
      .await
  }
}

impl Resolve<WriteArgs> for CopyIngressInstance {
  #[instrument(name = "CopyIngressInstance", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<IngressInstance> {
    let IngressInstance { config, .. } =
      get_check_permissions::<IngressInstance>(
        &self.id,
        user,
        PermissionLevel::Write.into(),
      )
      .await?;
    resource::create::<IngressInstance>(&self.name, config.into(), user)
      .await
  }
}

impl Resolve<WriteArgs> for DeleteIngressInstance {
  #[instrument(name = "DeleteIngressInstance", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<IngressInstance> {
    Ok(resource::delete::<IngressInstance>(&self.id, args).await?)
  }
}

impl Resolve<WriteArgs> for UpdateIngressInstance {
  #[instrument(name = "UpdateIngressInstance", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<IngressInstance> {
    Ok(
      resource::update::<IngressInstance>(
        &self.id,
        self.config,
        user,
      )
      .await?,
    )
  }
}

impl Resolve<WriteArgs> for RenameIngressInstance {
  #[instrument(name = "RenameIngressInstance", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(
      resource::rename::<IngressInstance>(&self.id, &self.name, user)
        .await?,
    )
  }
}
