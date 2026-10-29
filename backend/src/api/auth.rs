//! Extratores, middlewares e adaptadores HTTP para controle de autorização no Axum.
//!
//! Fornece componentes para proteção declarativa de rotas, extração do contexto
//! ativo de navegação (`ActiveCaContext`) e imposição rigorosa de permissões (`RequirePermission`).

use std::{marker::PhantomData, sync::Arc};

use axum::{
    extract::{FromRequestParts, Request},
    http::{HeaderMap, StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::modules::ca::domain::entities::active_context::ActiveCaContext;
use crate::modules::ca::{
    application::ports::{AuthorizationServiceError, AuthorizationServicePort},
    domain::entities::permission::Permission,
};

/// Resposta JSON padronizada para falhas de autenticação e autorização HTTP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationErrorResponse {
    /// Código HTTP de status da resposta.
    #[serde(skip)]
    pub status: StatusCode,

    /// Categoria simplificada do erro (ex: "forbidden", "unauthorized", "bad_request").
    pub error: String,

    /// Mensagem detalhada da recusa para fins de retorno ao cliente.
    pub message: String,
}

impl AuthorizationErrorResponse {
    /// Constrói um erro HTTP 403 Forbidden.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            error: "forbidden".to_string(),
            message: message.into(),
        }
    }

    /// Constrói um erro HTTP 401 Unauthorized.
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            error: "unauthorized".to_string(),
            message: message.into(),
        }
    }

    /// Constrói um erro HTTP 400 Bad Request.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error: "bad_request".to_string(),
            message: message.into(),
        }
    }
}

impl IntoResponse for AuthorizationErrorResponse {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = serde_json::to_string(&self).unwrap_or_else(|_| {
            r#"{"error":"internal_error","message":"Falha ao serializar erro."}"#.to_string()
        });

        (
            status,
            [("content-type", "application/json; charset=utf-8")],
            body,
        )
            .into_response()
    }
}

impl From<AuthorizationServiceError> for AuthorizationErrorResponse {
    fn from(err: AuthorizationServiceError) -> Self {
        match err {
            AuthorizationServiceError::Forbidden(msg) => Self::forbidden(msg),
            AuthorizationServiceError::AcademicCenterMismatch {
                role_ca_id,
                target_ca_id,
            } => Self::forbidden(format!(
                "Isolamento violado: o cargo pertence ao CA '{role_ca_id}', mas a operação destina-se ao CA '{target_ca_id}'."
            )),
            AuthorizationServiceError::NoActiveRoleInCa { user_id, ca_id } => {
                Self::forbidden(format!(
                    "O usuário '{user_id}' não possui cargo ativo na gestão do Centro Acadêmico '{ca_id}'."
                ))
            }
            AuthorizationServiceError::RepositoryFailure(msg) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: "internal_error".to_string(),
                message: msg,
            },
        }
    }
}

/// Helper para extrair UUIDs dos cabeçalhos HTTP.
fn extract_uuid_header(
    headers: &HeaderMap,
    primary_header: &str,
    fallback_header: Option<&str>,
) -> Result<Uuid, AuthorizationErrorResponse> {
    let header_value = headers
        .get(primary_header)
        .or_else(|| fallback_header.and_then(|h| headers.get(h)));

    match header_value {
        Some(val) => {
            let s = val.to_str().map_err(|_| {
                AuthorizationErrorResponse::bad_request(format!(
                    "O cabeçalho '{primary_header}' contém caracteres inválidos."
                ))
            })?;
            Uuid::parse_str(s).map_err(|_| {
                AuthorizationErrorResponse::bad_request(format!(
                    "O cabeçalho '{primary_header}' não contém um UUID válido."
                ))
            })
        }
        None => Err(AuthorizationErrorResponse::bad_request(format!(
            "Cabeçalho obrigatório ausente: '{primary_header}'."
        ))),
    }
}

/// Extrator Axum para recuperar o contexto ativo de CA a partir da requisição HTTP.
///
/// Lê os cabeçalhos:
/// - `X-User-ID`: identificador do usuário autenticado.
/// - `X-Active-CA-ID` (ou `X-CA-ID`): identificador do Centro Acadêmico ativo na interface.
impl<S> FromRequestParts<S> for ActiveCaContext
where
    S: Send + Sync,
{
    type Rejection = AuthorizationErrorResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user_id = match extract_uuid_header(&parts.headers, "X-User-ID", None) {
            Ok(id) => id,
            Err(_) => {
                return Err(AuthorizationErrorResponse::unauthorized(
                    "Usuário não autenticado: cabeçalho 'X-User-ID' ausente ou inválido.",
                ));
            }
        };

        let ca_id = extract_uuid_header(&parts.headers, "X-Active-CA-ID", Some("X-CA-ID"))?;

        ActiveCaContext::new(user_id, ca_id).map_err(|err| {
            AuthorizationErrorResponse::bad_request(format!("Contexto ativo inválido: {err}"))
        })
    }
}

/// Contrato para estados compartilhados que fornecem o serviço de autorização.
pub trait HasAuthorizationService: Send + Sync {
    /// Retorna uma referência compartilhada ao serviço de autorização.
    fn authorization_service(&self) -> Arc<dyn AuthorizationServicePort>;
}

impl HasAuthorizationService for Arc<dyn AuthorizationServicePort> {
    fn authorization_service(&self) -> Arc<dyn AuthorizationServicePort> {
        Arc::clone(self)
    }
}

/// Trait marcador para vincular tipos de tempo de compilação a uma [`Permission`].
pub trait PermissionRequirement: Send + Sync + 'static {
    /// Retorna a permissão canônica exigida por este marcador.
    fn permission() -> Permission;
}

// Marcadores para os tipos de permissões mais frequentes
/// Requisito de permissão para administração de integrantes da gestão.
pub struct ManageMembersPermission;
impl PermissionRequirement for ManageMembersPermission {
    fn permission() -> Permission {
        Permission::ManageMembers
    }
}

/// Requisito de permissão para publicação direta de notícias.
pub struct PublishNewsDirectPermission;
impl PermissionRequirement for PublishNewsDirectPermission {
    fn permission() -> Permission {
        Permission::PublishNewsDirect
    }
}

/// Requisito de permissão para homologação de notícias propostas.
pub struct ApproveNewsPermission;
impl PermissionRequirement for ApproveNewsPermission {
    fn permission() -> Permission {
        Permission::ApproveNews
    }
}

/// Requisito de permissão para publicação direta de eventos.
pub struct PublishEventsDirectPermission;
impl PermissionRequirement for PublishEventsDirectPermission {
    fn permission() -> Permission {
        Permission::PublishEventsDirect
    }
}

/// Requisito de permissão para homologação de eventos propostos.
pub struct ApproveEventsPermission;
impl PermissionRequirement for ApproveEventsPermission {
    fn permission() -> Permission {
        Permission::ApproveEvents
    }
}

/// Requisito de permissão para publicação de prestação de contas.
pub struct PublishFinancesPermission;
impl PermissionRequirement for PublishFinancesPermission {
    fn permission() -> Permission {
        Permission::PublishFinancialStatements
    }
}

/// Extrator declarativo Axum que exige que o usuário possua a permissão `P` no CA ativo.
///
/// Rejeita a requisição imediatamente com HTTP 403 Forbidden caso o usuário não possua
/// a permissão requerida no Centro Acadêmico ativo informado.
pub struct RequirePermission<P: PermissionRequirement> {
    /// O contexto ativo validado e autorizado.
    pub context: ActiveCaContext,
    _marker: PhantomData<P>,
}

impl<P: PermissionRequirement> RequirePermission<P> {
    /// Retorna o identificador do usuário validado.
    pub fn user_id(&self) -> Uuid {
        self.context.user_id
    }

    /// Retorna o identificador do Centro Acadêmico validado.
    pub fn active_ca_id(&self) -> Uuid {
        self.context.active_ca_id
    }
}

impl<S, P> FromRequestParts<S> for RequirePermission<P>
where
    S: HasAuthorizationService + Send + Sync,
    P: PermissionRequirement,
{
    type Rejection = AuthorizationErrorResponse;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let context = ActiveCaContext::from_request_parts(parts, state).await?;
        let auth_service = state.authorization_service();

        auth_service
            .check_permission(context.user_id, context.active_ca_id, &P::permission())
            .await
            .map_err(AuthorizationErrorResponse::from)?;

        Ok(Self {
            context,
            _marker: PhantomData,
        })
    }
}

/// Middleware funcional para verificar permissões em rotas do Axum.
///
/// Intercepta a requisição, extrai o contexto ativo dos cabeçalhos e valida a permissão
/// indicada via [`AuthorizationServicePort`]. Retorna HTTP 403 Forbidden em caso de recusa.
pub async fn require_permission_middleware(
    auth_service: Arc<dyn AuthorizationServicePort>,
    permission: Permission,
    request: Request,
    next: Next,
) -> Result<Response, AuthorizationErrorResponse> {
    let (mut parts, body) = request.into_parts();
    let context = ActiveCaContext::from_request_parts(&mut parts, &()).await?;

    auth_service
        .check_permission(context.user_id, context.active_ca_id, &permission)
        .await
        .map_err(AuthorizationErrorResponse::from)?;

    let reconstructed_req = Request::from_parts(parts, body);
    Ok(next.run(reconstructed_req).await)
}
