//! Composição HTTP e estado compartilhado da aplicação.
//!
//! [`router::create_router`] registra as verificações de saúde e prontidão.
//! O chamador fornece [`state::AppState`] com `Router::with_state`, permitindo
//! usar o mesmo router no servidor e nos testes sem abrir uma porta HTTP.

pub mod auth;
pub mod router;
pub mod state;

pub use auth::{
    ActiveCaContext, AuthorizationErrorResponse, HasAuthorizationService, PermissionRequirement,
    RequirePermission, require_permission_middleware,
};
