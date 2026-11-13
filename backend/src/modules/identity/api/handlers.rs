//! Adaptação HTTP do caso de uso de verificação institucional.
//!
//! O handler extrai o Firebase Bearer token e o access token Microsoft do JSON,
//! aplica timeout de 15 segundos e devolve a conta provisionada e a evidência. Não exige confirmação
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
            dto::{InstitutionalAuthRequest, InstitutionalAuthResponse},
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
/// HTTP 200 com `InstitutionalAuthResponse`, após verificação de identidade e
/// persistência. O serviço provisiona a conta incompleta e atribui o prazo; o
/// handler não cria sessão Firebase. O tempo total é limitado a 15 segundos.
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

    Ok(Json(InstitutionalAuthResponse::from(verification_result)))
}

/// GET /auth/session restaura usuário e evidência com o ID token Firebase.
pub(super) async fn restore_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let token = extractors::firebase_bearer_token(&headers)?;
    let result = timeout(
        Duration::from_secs(15),
        crate::modules::identity::application::services::accounts::restore(
            &state.clients.firebase,
            &state.pg_database,
            token,
        ),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())??;

    Ok(Json(super::dto::AccountSessionResponse {
        user: result.user.into(),
        verification: result.verification.map(Into::into),
    }))
}

/// PUT /users/me/profile conclui o perfil; credenciais são tratadas pelo Firebase.
pub(super) async fn complete_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<super::dto::CompleteProfileRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let token = extractors::firebase_bearer_token(&headers)?;
    let Json(body) = body.map_err(|_| ApiError::bad_request())?;
    let data = crate::modules::identity::domain::entities::user::ProfileCompletion {
        name: body.name,
        institutional_email: body.institutional_email,
        course_name: body.course_name,
        nickname: body.nickname,
        current_semester: body.current_semester,
        institution_entry_semester: body.institution_entry_semester,
    };
    let user = timeout(
        Duration::from_secs(15),
        crate::modules::identity::application::services::accounts::complete_profile(
            &state.clients.firebase,
            &state.pg_database,
            token,
            data,
        ),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())??;

    Ok(Json(super::dto::UserResponse::from(user)))
}
