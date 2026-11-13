//! Fiscalização do App Check na fronteira HTTP, antes de autenticação e I/O do domínio.
//!
//! Somente GET/HEAD /health e /ready dispensam o atestado. As demais rotas do
//! router global exigem um único X-Firebase-AppCheck e um App ID permitido.
//! Esta proteção comprova a origem do app; não substitui Authorization nem
//! altera a identidade ou as permissões de uma pessoa.

use super::{
    error::{ApiError, ApiResult},
    state::AppState,
};
use crate::modules::identity::infrastructure::firebase::AppCheckError;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method},
    middleware::Next,
    response::Response,
};

pub(super) async fn require_app_check(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> ApiResult<Response> {
    if matches!(*request.method(), Method::GET | Method::HEAD)
        && matches!(request.uri().path(), "/health" | "/ready")
    {
        return Ok(next.run(request).await);
    }

    let token = app_check_token(request.headers())?;
    let app = state
        .clients
        .app_check
        .verify(token)
        .await
        .map_err(|error| match error {
            AppCheckError::Invalid => invalid(),
            AppCheckError::Unavailable => ApiError::new(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "app_check_unavailable",
                "Não foi possível verificar o aplicativo. Tente novamente.",
                None,
            ),
        })?;

    request.extensions_mut().insert(app);
    Ok(next.run(request).await)
}

fn invalid() -> ApiError {
    ApiError::new(
        axum::http::StatusCode::FORBIDDEN,
        "app_check_invalid",
        "Não foi possível verificar este aplicativo.",
        None,
    )
}

fn app_check_token(headers: &HeaderMap) -> ApiResult<&str> {
    let mut values = headers.get_all("x-firebase-appcheck").iter();
    let value = values.next().ok_or_else(|| {
        ApiError::new(
            axum::http::StatusCode::FORBIDDEN,
            "app_check_required",
            "A verificação do aplicativo é necessária.",
            None,
        )
    })?;

    if values.next().is_some() {
        return Err(invalid());
    }

    let token = value.to_str().map_err(|_| invalid())?;

    if token.is_empty() || token.len() > 16_384 || token.bytes().any(|b| b.is_ascii_whitespace()) {
        return Err(invalid());
    }

    Ok(token)
}

#[cfg(test)]
mod tests;
