//! Handlers HTTP do módulo de identidade institucional.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};

use crate::{
    api::state::AppState,
    modules::identity::{
        api::dto::CurrentIdentityResponse,
        application::queries::current_identity::IdentityVerificationError,
    },
};

const UNB_STUDENT_EMAIL_DOMAIN: &str = "aluno.unb.br";
const FORBIDDEN_STUDENT_MATRICULA: &str = "242003979";
const INSTITUTIONAL_ACCOUNT_REQUIRED_MESSAGE: &str = "Apenas contas @aluno.unb.br são permitidas";

/// Valida o e-mail discente da UnB e rejeita identificadores não permitidos.
///
/// O endereço deve conter uma matrícula de nove dígitos e o domínio
/// institucional discente. A matrícula explicitamente proibida é recusada.
fn is_allowed_unb_student_email(email: &str) -> bool {
    let Some((matricula, domain)) = email.rsplit_once('@') else {
        return false;
    };

    domain.eq_ignore_ascii_case(UNB_STUDENT_EMAIL_DOMAIN)
        && matricula.len() == 9
        && matricula
            .bytes()
            .all(|character| character.is_ascii_digit())
        && matricula != FORBIDDEN_STUDENT_MATRICULA
}

/// Retorna a identidade do usuário após validar o token de autenticação.
pub async fn current_identity(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CurrentIdentityResponse>, (StatusCode, &'static str)> {
    let authorization = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let Some(authorization) = authorization else {
        return Err((
            StatusCode::UNAUTHORIZED,
            "É necessário realizar a autenticação.",
        ));
    };

    let Some((scheme, token)) = authorization.split_once(' ') else {
        return Err((StatusCode::UNAUTHORIZED, "Token de autenticação inválido."));
    };

    if !scheme.eq_ignore_ascii_case("Bearer") || token.trim().is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "Token de autenticação inválido."));
    }

    match state
        .identity_token_verifier
        .verify_id_token(token.trim())
        .await
    {
        Ok(identity) => {
            match identity.email.as_deref() {
                Some(email) if is_allowed_unb_student_email(email) => {}
                _ => {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        INSTITUTIONAL_ACCOUNT_REQUIRED_MESSAGE,
                    ));
                }
            }

            Ok(Json(CurrentIdentityResponse {
                uid: identity.uid,
                email: identity.email,
                email_verified: identity.email_verified,
            }))
        }
        Err(IdentityVerificationError::InvalidToken) => Err((
            StatusCode::UNAUTHORIZED,
            "Token de autenticação inválido ou expirado.",
        )),
        Err(IdentityVerificationError::MissingConfiguration) => {
            tracing::error!("A configuração de autenticação está ausente.");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "A autenticação do servidor não está configurada.",
            ))
        }
        Err(IdentityVerificationError::ProviderUnavailable) => {
            tracing::warn!("O provedor de autenticação está indisponível.");
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "O serviço de autenticação está temporariamente indisponível.",
            ))
        }
    }
}
