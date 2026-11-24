//! Autoriza operações editoriais no Centro Acadêmico correspondente.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    infrastructure::database::PgDatabase,
    modules::ca::{
        application::ports::{AcademicCenterRepository, AuthorizationServicePort},
        domain::entities::{AcademicCenterStatus, Permission},
    },
};

use crate::modules::news::application::ports::{NewsAuthorizationError, NewsAuthorizationPort};

pub(crate) struct AcademicCenterNewsAuthorization {
    pub db: PgDatabase,
    pub roles: Arc<dyn AuthorizationServicePort>,
}

impl AcademicCenterNewsAuthorization {
    async fn permits(
        &self,
        actor: Uuid,
        ca: Uuid,
        permission: Permission,
    ) -> Result<bool, NewsAuthorizationError> {
        let Some(center) = self.db.find_by_id(ca).await.map_err(|error| {
            NewsAuthorizationError::TechnicalFailure {
                user_id: actor,
                reason: error.to_string(),
            }
        })?
        else {
            return Ok(false);
        };
        if center.status() != AcademicCenterStatus::Active {
            return Ok(false);
        }

        Ok(center.created_by_id() == Some(actor)
            || self
                .roles
                .check_permission(actor, ca, &permission)
                .await
                .is_ok())
    }
}

#[async_trait]
impl NewsAuthorizationPort for AcademicCenterNewsAuthorization {
    async fn can_publish_news(
        &self,
        actor: Uuid,
        ca: Uuid,
    ) -> Result<bool, NewsAuthorizationError> {
        self.permits(actor, ca, Permission::ApproveNews).await
    }

    async fn can_archive_news(
        &self,
        actor: Uuid,
        ca: Uuid,
    ) -> Result<bool, NewsAuthorizationError> {
        self.permits(actor, ca, Permission::ApproveNews).await
    }

    async fn can_submit_news(
        &self,
        actor: Uuid,
        ca: Uuid,
        author: Uuid,
    ) -> Result<bool, NewsAuthorizationError> {
        if actor == author {
            return Ok(true);
        }
        self.permits(actor, ca, Permission::PublishNewsDirect).await
    }
}
