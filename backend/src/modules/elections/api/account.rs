//! Conta proprietária dos recursos eleitorais privados.

use std::time::Duration;

use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, request::Parts},
};
use uuid::Uuid;

use crate::{
    api::{error::ApiError, state::AppState},
    modules::elections::application::registration::VoterIdentity,
    modules::identity::{
        api::extractors::firebase_bearer_token,
        application::{
            error::AuthenticationError,
            ports::{IdentityTokenVerifier, UserRepository},
        },
        domain::UserStatus,
    },
};

/// Confere a sessão antes de ler o corpo. A vigência do vínculo é avaliada no cadastro ou voto.
pub(super) struct AuthenticatedAccount(pub(super) VoterIdentity);

impl AuthenticatedAccount {
    pub(super) async fn authenticate(
        headers: &HeaderMap,
        state: &AppState,
    ) -> Result<Self, ApiError> {
        let token = firebase_bearer_token(headers)?;
        let identity = state.clients.firebase.verify(token).await?;
        let account = state
            .pg_database
            .get_by_firebase_uid(&identity.firebase_uid)
            .await
            .map_err(AuthenticationError::from)?
            .ok_or(AuthenticationError::AccountNotProvisioned)?;
        if account.status() == UserStatus::Inactive {
            return Err(ApiError::from(AuthenticationError::AccountInactive));
        }
        Ok(Self(VoterIdentity {
            account_id: account.id(),
            microsoft_object_id: identity
                .microsoft_object_id
                .as_deref()
                .and_then(|id| Uuid::parse_str(id).ok()),
        }))
    }
}

impl FromRequestParts<AppState> for AuthenticatedAccount {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        tokio::time::timeout(
            Duration::from_secs(15),
            Self::authenticate(&parts.headers, state),
        )
        .await
        .map_err(|_| ApiError::service_unavailable())?
    }
}
