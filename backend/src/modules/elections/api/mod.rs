//! Rotas HTTP do quadro eleitoral e dos recursos privados da própria conta.

use std::time::Duration;

use axum::{
    BoxError, Router,
    error_handling::HandleErrorLayer,
    extract::DefaultBodyLimit,
    routing::{get, post, put},
};
use tower::{ServiceBuilder, timeout::TimeoutLayer};

use crate::{
    api::{error::ApiError, state::AppState},
    modules::elections::domain::profile::MAX_SIGNED_MESSAGE_BYTES,
};

pub mod dto;
mod error;
pub mod handlers;
mod serde;

const JSON_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

fn with_json_timeout<S>(router: Router<S>, deadline: Duration) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.route_layer(
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|_: BoxError| async {
                ApiError::service_unavailable()
            }))
            .layer(TimeoutLayer::new(deadline)),
    )
}

/// Expõe as consultas públicas. A exportação do quadro é transmitida em partes.
pub fn public_board_routes() -> Router<AppState> {
    let json = Router::new()
        .route("/elections/{id}/tally", get(handlers::tally_result))
        .route(
            "/elections/{id}/board/checkpoint",
            get(handlers::checkpoint),
        );

    with_json_timeout(json, JSON_REQUEST_TIMEOUT)
        .route("/elections/{id}/board", get(handlers::snapshot))
}

/// Agrupa as rotas JSON autenticadas de cadastro, votação e administração.
/// O router raiz aplica App Check ao grupo.
pub fn protected_json_routes() -> Router<AppState> {
    let json = Router::new()
        .route(
            "/elections/{id}/receipts/me",
            get(handlers::read_own_receipt),
        )
        .route(
            "/elections/{id}/registration/me",
            get(handlers::read_own_registration).put(handlers::register_voter_key),
        )
        .route("/elections/{id}/ballots", post(handlers::submit_ballot))
        .route(
            "/elections/{id}/tally/shares/{authority}",
            put(handlers::submit_share),
        )
        .route(
            "/elections/{id}/authority/me",
            get(handlers::own_authority).put(handlers::register_authority),
        )
        .route("/elections/{id}/manifest", put(handlers::publish_manifest))
        .route("/elections/{id}/period", put(handlers::reschedule))
        .route("/elections/{id}/cancellation", put(handlers::cancel))
        .layer(DefaultBodyLimit::max(MAX_SIGNED_MESSAGE_BYTES));

    with_json_timeout(json, JSON_REQUEST_TIMEOUT)
}

#[cfg(test)]
mod tests {
    use std::{future::pending, time::Duration};

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        routing::get,
    };
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::modules::elections::api::with_json_timeout;

    #[tokio::test]
    async fn json_deadline_returns_the_api_error_and_only_covers_its_group() {
        let json = Router::new().route("/json", get(pending::<&'static str>));
        let router = with_json_timeout(json, Duration::ZERO).route(
            "/stream",
            get(|| async {
                tokio::time::sleep(Duration::from_millis(5)).await;
                "quadro"
            }),
        );

        let request = |path| Request::builder().uri(path).body(Body::empty()).unwrap();

        let response = router.clone().oneshot(request("/json")).await.unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["error"]["code"], "service_unavailable");

        let response = router.oneshot(request("/stream")).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
