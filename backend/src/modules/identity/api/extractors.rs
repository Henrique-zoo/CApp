//! Autenticação HTTP com Firebase e confirmação institucional persistida.
//!
//! O parser Bearer é reutilizado pela rota de verificação. O extractor de identidade
//! exige evidência vigente, inclusive de não estudantes, e pode proteger handlers,
//! nunca pela operação que cria/renova a confirmação. Nenhuma rota atual exige o
//! extractor; sua existência não instala proteção global no router.

use std::time::Duration;

use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, request::Parts},
};
use reqwest::header::AUTHORIZATION;
use uuid::Uuid;

use crate::{
    api::{
        error::{ApiError, ApiResult},
        state::AppState,
    },
    modules::identity::application::services::{self, authentication::AuthenticatedIdentity},
};

/// Conta autenticada, disponível para operações que também podem ser exercidas
/// após o vencimento da verificação institucional.
pub(crate) struct AuthenticatedAccount {
    pub account_id: Uuid,
    pub microsoft_object_id: Option<Uuid>,
}

impl AuthenticatedAccount {
    /// Valida a credencial e delega a resolução da conta ao módulo de identidade.
    pub(crate) async fn authenticate(headers: &HeaderMap, state: &AppState) -> ApiResult<Self> {
        let token = firebase_bearer_token(headers)?;
        let (user, identity) = tokio::time::timeout(
            Duration::from_secs(10),
            services::accounts::authenticate_account(
                &state.clients.firebase,
                &state.pg_database,
                token,
            ),
        )
        .await
        .map_err(|_| ApiError::service_unavailable())??;

        Ok(Self {
            account_id: user.id(),
            microsoft_object_id: identity
                .microsoft_object_id
                .as_deref()
                .and_then(|value| Uuid::parse_str(value).ok()),
        })
    }
}

impl FromRequestParts<AppState> for AuthenticatedAccount {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> ApiResult<Self> {
        Self::authenticate(&parts.headers, state).await
    }
}

/// Identidade autenticada com confirmação institucional ainda válida.
///
/// Exige somente `Authorization: Bearer <Firebase ID token>`.
/// Rejeições impedem a execução do handler. Usa o serviço de autenticação e uma
/// consulta PostgreSQL; não consulta o Graph. O verificador Firebase pode buscar
/// chaves públicas se seu cache precisar de atualização. A identidade extraída
/// não concede permissões específicas como administração de CA.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct AuthenticatedInstitutionalIdentity(
    /// Identidade disponibilizada à operação protegida.
    pub(crate) AuthenticatedIdentity,
);

impl FromRequestParts<AppState> for AuthenticatedInstitutionalIdentity {
    /// Erro convertido pelo Axum antes da execução do handler.
    type Rejection = ApiError;

    /// Autentica os cabeçalhos sem consumir o corpo nem alterar a evidência persistida.
    ///
    /// # Retorno
    ///
    /// Identidade institucional para o handler após validar Firebase, objeto
    /// Microsoft e evidência. A chamada ao serviço tem timeout total de dez segundos.
    /// Não prorroga a confirmação e não recebe access token Microsoft.
    ///
    /// # Erros
    ///
    /// Rejeita credenciais inválidas, ausência de evidência institucional
    /// válida, falhas das dependências e tempo limite excedido.
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = firebase_bearer_token(&parts.headers)?;

        let identity = tokio::time::timeout(
            Duration::from_secs(10),
            services::authentication::authenticate_institutional_request(
                &state.clients.firebase,
                &state.pg_database,
                token,
            ),
        )
        .await
        .map_err(|_| {
            tracing::warn!("Tempo limite de autenticação excedido");
            ApiError::service_unavailable()
        })??;
        Ok(Self(identity))
    }
}

/// Extrai uma única credencial Bearer sem modificá-la.
///
/// # Erros
///
/// Retorna erro de autenticação para cabeçalho ausente, repetido,
/// esquema inválido ou token vazio ou contendo espaços em branco.
/// O esquema é comparado sem diferenciar maiúsculas e minúsculas. Não decodifica
/// o JWT, não o valida criptograficamente e devolve uma referência ao cabeçalho.
pub(crate) fn firebase_bearer_token(headers: &HeaderMap) -> ApiResult<&str> {
    let mut values = headers.get_all(AUTHORIZATION).iter();

    let authorization = values
        .next()
        .ok_or_else(ApiError::unauthorized)?
        .to_str()
        .map_err(|_| ApiError::unauthorized())?;

    if values.next().is_some() {
        return Err(ApiError::unauthorized());
    }

    let (schema, token) = authorization
        .split_once(' ')
        .ok_or_else(ApiError::unauthorized)?;

    if !schema.eq_ignore_ascii_case("Bearer")
        || token.is_empty()
        || token.chars().any(char::is_whitespace)
    {
        return Err(ApiError::unauthorized());
    }

    Ok(token)
}
