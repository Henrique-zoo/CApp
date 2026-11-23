//! Extrai os bytes originais para o codec eleitoral detectar chaves JSON duplicadas.

use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    http::{StatusCode, header::CONTENT_TYPE},
};

use crate::api::{error::ApiError, state::AppState};

pub(super) struct SignedJson(pub(super) Bytes);

impl FromRequest<AppState> for SignedJson {
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &AppState) -> Result<Self, ApiError> {
        let mut types = request.headers().get_all(CONTENT_TYPE).iter();
        let media_type = types
            .next()
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next());
        if types.next().is_some()
            || !media_type
                .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"))
        {
            return Err(ApiError::new(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media_type",
                "Envie a mensagem eleitoral como application/json.",
                None,
            ));
        }
        Bytes::from_request(request, state)
            .await
            .map(Self)
            .map_err(|error| {
                if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
                    ApiError::new(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "payload_too_large",
                        "A mensagem eleitoral excede o limite.",
                        None,
                    )
                } else {
                    ApiError::bad_request()
                }
            })
    }
}
