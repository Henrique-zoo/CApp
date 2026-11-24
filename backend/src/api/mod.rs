//! Composição HTTP, erros públicos e estado compartilhado da aplicação.
//!
//! [`router::create_router`] combina as rotas de saúde, prontidão e identidade.
//! O chamador fornece [`state::AppState`] para os handlers e um verificador
//! App Check para o middleware. A mesma fábrica atende ao servidor e aos testes
//! em memória.
//! [`error::ApiError`] define o envelope JSON usado nos erros de autenticação.

mod app_check;
pub mod error;
pub mod auth;
pub mod router;
pub mod state;

pub use auth::{
    ActiveCaContext, AuthorizationErrorResponse, HasAuthorizationService, PermissionRequirement,
    RequirePermission, require_permission_middleware,
};
