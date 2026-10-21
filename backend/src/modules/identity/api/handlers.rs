//! Adaptação HTTP do caso de uso de verificação institucional.
//!
//! O handler extrai o Firebase Bearer token e o access token Microsoft do JSON,
//! aplica timeout de 15 segundos e devolve perfil e validade. Não exige confirmação
//! anterior, pois essa mesma operação deve permitir sua criação e renovação.

use std::time::Duration;

use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::HeaderMap,
    response::IntoResponse,
};
use tokio::time::timeout;

use crate::{
    api::{
        error::{ApiError, ApiResult},
        state::AppState,
    },
    modules::identity::{
        api::{
            dto::{InstitutionalAuthRequest, InstitutionalVerificationResponse},
            extractors,
        },
        application::services::authentication,
    },
};

/// Cria ou renova a confirmação institucional e retorna seu perfil e validade.
///
/// # Entrada
///
/// `POST /auth/institutional/verify` exige `Authorization: Bearer <Firebase ID token>`
/// e JSON com `microsoftAccessToken`. Credenciais são interpretadas separadamente;
/// esta rota não usa o extractor que exige confirmação institucional preexistente.
///
/// # Retorno
///
/// HTTP 200 com `InstitutionalVerificationResponse`, após confirmação positiva e
/// persistência. O prazo vem do serviço; o handler não cria sessão Firebase nem
/// provisiona usuário. O tempo total da chamada ao serviço é limitado a 15 segundos.
///
/// # Erros
///
/// Credencial Bearer ausente ou malformada retorna 401. Toda rejeição do extractor
/// JSON é convertida em 400, assim como access token ausente, nulo ou vazio.
/// Erros de aplicação seguem a conversão em `super::error`; timeout retorna 503 com
/// `service_unavailable`. O cancelamento por timeout não garante reversão de uma
/// escrita já concluída no PostgreSQL.
pub(super) async fn verify_institutional(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<InstitutionalAuthRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let firebase_token = extractors::firebase_bearer_token(&headers)?;

    let Json(body) = body.map_err(|_| ApiError::bad_request())?;

    let microsoft_token = body
        .microsoft_access_token
        .as_deref()
        .ok_or_else(ApiError::bad_request)?;

    let verification_result = timeout(
        Duration::from_secs(15),
        authentication::verify_institutional_identity(
            &state.clients.microsoft,
            &state.clients.firebase,
            &state.pg_database,
            firebase_token,
            microsoft_token,
        ),
    )
    .await
    .map_err(|_| {
        tracing::warn!("Tempo limite de verificação institucional excedido");
        ApiError::service_unavailable()
    })??;

    Ok(Json(InstitutionalVerificationResponse::from(
        verification_result,
    )))
}
