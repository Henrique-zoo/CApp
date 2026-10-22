//! Contratos JSON, erros e rejeições HTTP sem serviços externos reais.
use super::*;
use crate::{
    api::{
        error::ApiError,
        state::{AppState, Clients},
    },
    infrastructure::database::PgDatabase,
    modules::identity::{
        application::{
            error::AuthenticationError,
            ports::{InstitutionalAffiliation, InstitutionalProfile},
            services::authentication::InstitutionalVerificationResult,
        },
        infrastructure::{firebase::FirebaseTokenVerifier, microsoft::MicrosoftGraphClient},
    },
};
use axum::{
    body::{Body, to_bytes},
    extract::FromRequestParts,
    http::{HeaderMap, HeaderValue, Request, StatusCode},
    response::IntoResponse,
};
use serde_json::{Value, json};
use tower::ServiceExt;

/// Converte o corpo da resposta em JSON para verificar o contrato público.
async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap()
}

/// Constrói estado com banco fechado para provar que rejeições locais não precisam de SQL.
async fn state() -> AppState {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://test:test@127.0.0.1:1/test")
        .unwrap();
    pool.close().await;
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    AppState::new(
        PgDatabase::new(pool),
        Clients::new(
            FirebaseTokenVerifier::new("test-project".into(), http.clone()).unwrap(),
            MicrosoftGraphClient::new(http),
        ),
    )
}

/// Bearer aceita variações de caixa sem modificar o token.
#[test]
fn bearer_scheme_is_case_insensitive() {
    for scheme in ["Bearer", "bearer", "BEARER"] {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            format!("{scheme} abc.def.ghi").parse().unwrap(),
        );
        assert_eq!(
            extractors::firebase_bearer_token(&headers).unwrap(),
            "abc.def.ghi"
        );
    }
}

/// Credenciais ausentes, repetidas, malformadas ou não textuais são recusadas.
#[test]
fn invalid_bearer_headers_are_rejected() {
    assert!(extractors::firebase_bearer_token(&HeaderMap::new()).is_err());
    for raw in [
        "",
        "Basic token",
        "Bearer",
        "Bearer ",
        "Bearer  token",
        " Bearer token",
        "Bearer a b",
        "Bearer a\tb",
        "Bearer\ttoken",
    ] {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", raw.parse().unwrap());
        assert!(
            extractors::firebase_bearer_token(&headers).is_err(),
            "{raw}"
        );
    }
    let mut headers = HeaderMap::new();
    headers.append("authorization", HeaderValue::from_static("Bearer first"));
    headers.append("authorization", HeaderValue::from_static("Bearer second"));
    assert!(extractors::firebase_bearer_token(&headers).is_err());
    headers.clear();
    headers.insert(
        "authorization",
        HeaderValue::from_bytes(b"Bearer \xff").unwrap(),
    );
    assert!(extractors::firebase_bearer_token(&headers).is_err());
}

/// O DTO normaliza ausência, null e string vazia, mas não aceita tipos incompatíveis.
#[test]
fn microsoft_token_input_contract() {
    for value in [
        json!({}),
        json!({"microsoftAccessToken": null}),
        json!({"microsoftAccessToken": ""}),
    ] {
        assert!(
            serde_json::from_value::<dto::InstitutionalAuthRequest>(value)
                .unwrap()
                .microsoft_access_token
                .is_none()
        );
    }
    for token in ["token", " "] {
        assert_eq!(
            serde_json::from_value::<dto::InstitutionalAuthRequest>(
                json!({"microsoftAccessToken": token})
            )
            .unwrap()
            .microsoft_access_token
            .as_deref(),
            Some(token)
        );
    }
    for token in [json!(1), json!(false), json!([]), json!({})] {
        assert!(
            serde_json::from_value::<dto::InstitutionalAuthRequest>(
                json!({"microsoftAccessToken":token})
            )
            .is_err()
        );
    }
}

/// Respostas distinguem os vínculos e preservam campos opcionais e prazo sem expor tokens.
#[test]
fn verification_json_preserves_affiliation_and_expiry() {
    let expires_at = chrono::DateTime::parse_from_rfc3339("2026-10-21T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    for (affiliation, expected) in [
        (
            InstitutionalAffiliation::Student {
                course_name: Some("Curso".into()),
            },
            json!({"type":"student", "courseName":"Curso"}),
        ),
        (
            InstitutionalAffiliation::Student { course_name: None },
            json!({"type":"student", "courseName":null}),
        ),
        (
            InstitutionalAffiliation::NonStudent,
            json!({"type":"nonStudent"}),
        ),
    ] {
        let response =
            dto::InstitutionalVerificationResponse::from(InstitutionalVerificationResult {
                profile: InstitutionalProfile {
                    id: "microsoft-id".into(),
                    name: None,
                    mail: None,
                    affiliation,
                },
                expires_at,
            });
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({"profile": {
            "microsoftObjectId":"microsoft-id", "name":null, "email":null, "affiliation":expected
        }, "expiresAt":"2026-10-21T12:00:00Z"})
        );
    }
}

/// Cada erro de aplicação produz status, código e desafio HTTP apropriados.
#[tokio::test]
async fn authentication_errors_have_stable_public_codes() {
    for (error, status, code) in [
        (AuthenticationError::InvalidToken, 401, "invalid_token"),
        (AuthenticationError::ExpiredToken, 401, "expired_token"),
        (
            AuthenticationError::IdentityMismatch,
            401,
            "identity_mismatch",
        ),
        (
            AuthenticationError::InvalidMicrosoftId,
            401,
            "invalid_microsoft_id",
        ),
        (
            AuthenticationError::UnknownInstitution,
            403,
            "unknown_institution",
        ),
        (
            AuthenticationError::InstitutionalVerificationRequired,
            403,
            "institutional_verification_required",
        ),
        (
            AuthenticationError::ProviderUnavailable,
            503,
            "authentication_provider_unavailable",
        ),
        (AuthenticationError::Internal, 500, "internal_server_error"),
    ] {
        let response = ApiError::from(error).into_response();
        assert_eq!(response.status().as_u16(), status);
        assert_eq!(
            response.headers().contains_key("www-authenticate"),
            status == 401
        );
        if status == 401 {
            assert_eq!(response.headers()["www-authenticate"], "Bearer");
        }
        let body = response_json(response).await;
        assert_eq!(body["error"]["code"], code);
        assert!(body["error"]["message"].as_str().unwrap().len() > 0);
        assert!(body["error"]["details"].is_null());
    }
}

/// A restrição estudantil, quando usada por uma funcionalidade, preserva a instituição.
#[tokio::test]
async fn membership_error_exposes_only_public_institution_context() {
    let response = ApiError::from(AuthenticationError::StudentMembershipRequired {
        institution_name: "Instituição A".into(),
    })
    .into_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "student_membership_required");
    assert_eq!(
        body["error"]["details"],
        json!({"institution_name":"Instituição A"})
    );
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Instituição A")
    );
}

/// Os construtores comuns de erro mantêm status e códigos previsíveis.
#[tokio::test]
async fn generic_api_errors_have_expected_statuses() {
    for (error, status, code) in [
        (ApiError::bad_request(), 400, "bad_request"),
        (ApiError::unauthorized(), 401, "unauthorized"),
        (ApiError::forbidden(), 403, "forbidden"),
        (ApiError::service_unavailable(), 503, "service_unavailable"),
        (
            ApiError::internal_server_error(),
            500,
            "internal_server_error",
        ),
    ] {
        let response = error.into_response();
        assert_eq!(response.status().as_u16(), status);
        assert_eq!(response_json(response).await["error"]["code"], code);
    }
}

/// A rota exige Bearer mesmo quando o JSON é inválido e não acessa o banco fechado.
#[tokio::test]
async fn route_rejects_missing_bearer_before_json_errors() {
    let app = router().with_state(state().await);
    let response = app
        .oneshot(
            Request::post("/auth/institutional/verify")
                .header("content-type", "application/json")
                .body(Body::from("not-json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.headers()["www-authenticate"], "Bearer");
}

/// A rota converte rejeições de JSON e token Microsoft ausente em 400.
#[tokio::test]
async fn route_rejects_invalid_microsoft_token_bodies() {
    let app = router().with_state(state().await);
    for (content_type, body) in [
        ("application/json", "{}"),
        ("application/json", "{\"microsoftAccessToken\":null}"),
        ("application/json", "{\"microsoftAccessToken\":\"\"}"),
        ("application/json", "{\"microsoftAccessToken\":42}"),
        ("application/json", "not-json"),
        ("text/plain", "{}"),
        ("application/json", "[]"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::post("/auth/institutional/verify")
                    .header("authorization", "Bearer test-token")
                    .header("content-type", content_type)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(
            response_json(response).await["error"]["code"],
            "bad_request"
        );
    }
}

/// Token Firebase malformado é rejeitado pelo verificador real antes de SQL ou Graph.
#[tokio::test]
async fn route_rejects_invalid_firebase_token() {
    let app = router().with_state(state().await);
    let response = app
        .oneshot(
            Request::post("/auth/institutional/verify")
                .header("authorization", "Bearer invalid")
                .header("content-type", "application/json")
                .body(Body::from("{\"microsoftAccessToken\":\"test\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response_json(response).await["error"]["code"],
        "invalid_token"
    );
}

/// O extractor protegido rejeita ausência e formato inválido de credenciais antes de SQL.
#[tokio::test]
async fn protected_extractor_rejects_invalid_credentials() {
    let state = state().await;
    for authorization in [None, Some("Basic token"), Some("Bearer invalid")] {
        let mut request = Request::builder();
        if let Some(value) = authorization {
            request = request.header("authorization", value);
        }
        let (mut parts, _) = request.body(()).unwrap().into_parts();
        let error =
            extractors::AuthenticatedInstitutionalIdentity::from_request_parts(&mut parts, &state)
                .await
                .unwrap_err();
        assert_eq!(error.into_response().status(), StatusCode::UNAUTHORIZED);
    }
}
