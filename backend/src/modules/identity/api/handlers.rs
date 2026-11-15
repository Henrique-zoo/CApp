//! Adaptação HTTP do caso de uso de verificação institucional.
//!
//! O handler extrai o Firebase Bearer token e o access token Microsoft do JSON,
//! aplica timeout de 15 segundos e devolve a conta provisionada e a evidência. Não exige confirmação
//! anterior, pois essa mesma operação deve permitir sua criação e renovação.
//!
//! A restauração atende o acesso com credencial interna reutilizando a evidência
//! registrada, e a conclusão do perfil adapta o formulário depois da autenticação
//! por senha. Os handlers escolhem DTOs, extraem dados e convertem erros para HTTP;
//! os serviços conferem propriedade, validade institucional e regras da conta.
//! App Check pertence ao middleware da composição. Nenhum handler recebe senha,
//! cria token Firebase ou toma o curso declarado pelo cliente como evidência.

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
            dto::{
                AccountSessionResponse, CompleteProfileRequest, InstitutionalAuthRequest,
                InstitutionalAuthResponse,
            },
            extractors,
        },
        application::services::{accounts, authentication},
        domain::entities::user::ProfileCompletion,
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

/// Atende `GET /auth/session` sem iniciar uma consulta institucional ao Graph.
///
/// # Entrada
///
/// `state` fornece o verificador Firebase e os repositórios compartilhados.
/// `headers` deve conter um único Bearer Firebase; a rota não recebe corpo nem
/// access token Microsoft. A validação de App Check ocorre antes, no middleware.
///
/// # Retorno
///
/// HTTP 200 com [`AccountSessionResponse`] da conta autenticada, mesmo quando o
/// perfil está incompleto. A propriedade `verification` pode ser `null`, indicando
/// a necessidade de sincronização, sem que a autenticação Firebase seja descartada.
/// A restauração não devolve sugestão de e-mail, provisiona conta ou renova prazos.
///
/// # Erros e tempo limite
///
/// Bearer ausente/malformado retorna 401; conta ainda não provisionada, 404;
/// conta inativa, 403. Outras recusas seguem a conversão de erros da aplicação.
/// A chamada de serviço inteira é limitada a 15 segundos; timeout retorna 503
/// com `service_unavailable`. Falhas não produzem um envelope de sucesso parcial.
pub(super) async fn restore_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let token = extractors::firebase_bearer_token(&headers)?;
    let result = timeout(
        Duration::from_secs(15),
        accounts::restore(&state.clients.firebase, &state.pg_database, token),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())??;

    Ok(Json(AccountSessionResponse {
        user: result.user.into(),
        verification: result.verification.map(Into::into),
    }))
}

/// Atende `PUT /users/me/profile` com dados internos de uma conta já autenticada.
///
/// # Entrada
///
/// `state` fornece verificador e persistência; `headers` contém o Bearer Firebase.
/// `body` deve corresponder a [`CompleteProfileRequest`], com apelido, período atual,
/// ingresso e eventual nome declarado. Senha e e-mail de entrada já foram tratados
/// no Firebase; UID, estado, curso e contato institucional não fazem parte do JSON.
/// O backend exige token originado do provedor `password` da mesma identidade.
///
/// # Retorno e efeitos
///
/// HTTP 200 com o usuário após a escrita validada. O serviço controla a transição
/// para conta ativa e recusa reativação; a conclusão não modifica a evidência nem
/// concede vínculo estudantil. Repetições usam a mesma conta e preservam a primeira
/// data de conclusão. A resposta não cria uma sessão ou outra credencial.
///
/// # Erros e tempo limite
///
/// Toda rejeição do extractor JSON, inclusive campos desconhecidos, retorna 400.
/// Credencial Bearer ausente/malformada retorna 401. Perfil inválido retorna 400;
/// senha, evidência ou estado de conta incompatíveis retornam 403. Demais falhas
/// seguem a conversão dos erros de aplicação. O serviço é limitado a 15 segundos
/// e timeout retorna 503; cancelar a espera não desfaz uma escrita já concluída.
pub(super) async fn complete_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<CompleteProfileRequest>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let token = extractors::firebase_bearer_token(&headers)?;
    let Json(body) = body.map_err(|_| ApiError::bad_request())?;
    let data = ProfileCompletion {
        name: body.name,
        nickname: body.nickname,
        current_semester: body.current_semester,
        institution_entry_semester: body.institution_entry_semester,
    };
    let user = timeout(
        Duration::from_secs(15),
        accounts::complete_profile(&state.clients.firebase, &state.pg_database, token, data),
    )
    .await
    .map_err(|_| ApiError::service_unavailable())??;

    Ok(Json(super::dto::UserResponse::from(user)))
}
