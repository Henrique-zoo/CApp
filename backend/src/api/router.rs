//! Rotas globais de saúde, prontidão e identidade do backend.
//!
//! A composição é independente do transporte: o chamador pode servir o router
//! com Axum ou enviar requisições em memória. Nenhuma conexão é aberta ao
//! construir o router; o acesso ao PostgreSQL ocorre no handler de prontidão.
//! A rota `/identity/me` delega a validação do Bearer Firebase ID token ao
//! verificador de identidade configurado em [`AppState`]. Isso não implementa
//! autorização de recursos de negócio nem comprova o tenant institucional.

use std::time::Duration;

use axum::{Router, extract::State, routing::get};
use reqwest::StatusCode;
use tokio::time::timeout;

use crate::{api::state::AppState, modules::identity::api::handlers::current_identity};

/// Monta as rotas globais, ainda dependentes de [`AppState`].
///
/// # Rotas
///
/// | Método e caminho | Comportamento |
/// | --- | --- |
/// | `GET /health` | Retorna HTTP 200 e `O servidor está saudável.` sem consultar dependências. |
/// | `GET /identity/me` | Valida o Firebase ID token e retorna a identidade; erros de autenticação retornam HTTP 401. |
/// | `GET /ready` | Executa `SELECT 1 AS ok` no pool; retorna HTTP 200 no sucesso e HTTP 503 em erro ou timeout. |
///
/// A prontidão limita a espera pela aquisição da conexão e execução da consulta
/// a dois segundos. Sua resposta não contém corpo; falhas são registradas em
/// `tracing` no nível `warn`, sem devolver o diagnóstico SQL ao cliente.
/// A consulta verifica comunicação com o banco, não a existência das tabelas,
/// a versão do esquema nem a disponibilidade das demais integrações.
///
/// # Retorno
///
/// Um [`Router<AppState>`] que deve receber o estado por [`Router::with_state`]
/// antes de atender requisições. Não inicia servidor nem aplica migrations.
/// A rota `/identity/me` verifica a identidade por meio do adaptador configurado;
/// ela não define permissões de negócio nem confirma o vínculo ao tenant
/// institucional da UnB.
///
/// # Exemplos
///
/// Uma verificação de saúde em memória dispensa um PostgreSQL em execução:
///
/// ```
/// use axum::{body::Body, http::{Request, StatusCode}};
/// use backend::api::{router::create_router, state::AppState};
/// use sqlx::postgres::PgPoolOptions;
/// use tower::ServiceExt;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPoolOptions::new()
///     .connect_lazy("postgres://capp:capp@localhost:5432/capp")?;
/// let app = create_router().with_state(AppState::new(pool.clone()));
/// let response = app
///     .oneshot(Request::builder().uri("/health").body(Body::empty())?)
///     .await?;
/// assert_eq!(response.status(), StatusCode::OK);
/// pool.close().await;
/// # Ok(())
/// # }
/// ```
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(|| async { "O servidor está saudável." }))
        .route("/identity/me", get(current_identity))
        .route(
            "/ready",
            get(|State(state): State<AppState>| async move {
                let result = timeout(
                    Duration::from_secs(2),
                    sqlx::query("SELECT 1 AS ok").fetch_one(&state.pool),
                )
                .await;

                match result {
                    Ok(Ok(_)) => StatusCode::OK,
                    Ok(Err(error)) => {
                        tracing::warn!(%error, "Falha ao verificar o PostgreSQL.");
                        StatusCode::SERVICE_UNAVAILABLE
                    }
                    Err(error) => {
                        tracing::warn!(%error, "Timeout ao verificar o PostgreSQL.");
                        StatusCode::SERVICE_UNAVAILABLE
                    }
                }
            }),
        )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header::AUTHORIZATION},
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::create_router;
    use crate::{
        api::state::AppState,
        modules::identity::application::queries::current_identity::{
            AuthenticatedIdentity, IdentityTokenVerifier, IdentityVerificationError,
        },
    };

    struct MockIdentityTokenVerifier;

    #[async_trait]
    impl IdentityTokenVerifier for MockIdentityTokenVerifier {
        async fn verify_id_token(
            &self,
            token: &str,
        ) -> Result<AuthenticatedIdentity, IdentityVerificationError> {
            match token {
                "valid-test-token" => Ok(AuthenticatedIdentity {
                    uid: "firebase-user-123".to_owned(),
                    email: Some("232012345@aluno.unb.br".to_owned()),
                    email_verified: true,
                }),
                "external-email-token" => Ok(AuthenticatedIdentity {
                    uid: "external-user".to_owned(),
                    email: Some("student@example.com".to_owned()),
                    email_verified: true,
                }),
                "missing-email-token" => Ok(AuthenticatedIdentity {
                    uid: "user-without-email".to_owned(),
                    email: None,
                    email_verified: false,
                }),
                "malformed-student-email-token" => Ok(AuthenticatedIdentity {
                    uid: "malformed-student".to_owned(),
                    email: Some("12345@aluno.unb.br".to_owned()),
                    email_verified: true,
                }),
                "forbidden-matricula-token" => Ok(AuthenticatedIdentity {
                    uid: "forbidden-student".to_owned(),
                    email: Some("242003979@aluno.unb.br".to_owned()),
                    email_verified: true,
                }),
                "provider-down-token" => Err(IdentityVerificationError::ProviderUnavailable),
                _ => Err(IdentityVerificationError::InvalidToken),
            }
        }
    }

    fn test_app() -> axum::Router {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://capp:capp@localhost:5432/capp")
            .expect("A URL de teste deve ser válida");

        let state =
            AppState::with_identity_token_verifier(pool, Arc::new(MockIdentityTokenVerifier));

        create_router().with_state(state)
    }

    #[tokio::test]
    async fn identity_route_rejects_missing_token() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/identity/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn identity_route_rejects_malformed_authorization_header() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/identity/me")
                    .header(AUTHORIZATION, "Bearer ")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn identity_route_rejects_invalid_token() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/identity/me")
                    .header(AUTHORIZATION, "Bearer invalid-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn identity_route_returns_authenticated_identity() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/identity/me")
                    .header(AUTHORIZATION, "Bearer valid-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["uid"], "firebase-user-123");
        assert_eq!(payload["email"], "232012345@aluno.unb.br");
        assert_eq!(payload["email_verified"], true);
    }

    #[tokio::test]
    async fn identity_route_rejects_external_email_with_required_message() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/identity/me")
                    .header(AUTHORIZATION, "Bearer external-email-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            "Apenas contas @aluno.unb.br são permitidas"
        );
    }

    #[tokio::test]
    async fn identity_route_rejects_missing_email_with_required_message() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/identity/me")
                    .header(AUTHORIZATION, "Bearer missing-email-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            "Apenas contas @aluno.unb.br são permitidas"
        );
    }

    #[tokio::test]
    async fn identity_route_rejects_malformed_student_email_with_required_message() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/identity/me")
                    .header(AUTHORIZATION, "Bearer malformed-student-email-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            "Apenas contas @aluno.unb.br são permitidas"
        );
    }

    #[tokio::test]
    async fn identity_route_rejects_forbidden_matricula_with_required_message() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/identity/me")
                    .header(AUTHORIZATION, "Bearer forbidden-matricula-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            "Apenas contas @aluno.unb.br são permitidas"
        );
    }

    #[tokio::test]
    async fn identity_route_reports_provider_unavailability() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/identity/me")
                    .header(AUTHORIZATION, "Bearer provider-down-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
