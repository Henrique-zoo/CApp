//! Composição HTTP, erros públicos e estado compartilhado da aplicação.
//!
//! [`router::create_router`] combina as rotas de saúde, prontidão e identidade.
//! O chamador fornece [`state::AppState`] para os handlers e um verificador
//! App Check para o middleware. A mesma fábrica atende ao servidor e aos testes
//! em memória.
//! [`error::ApiError`] define o envelope JSON usado nos erros de autenticação.

//! [`shutdown`] fornece o tratamento de sinais e a execução do encerramento
//! controlado com drenagem de conexões e liberação de recursos.

mod app_check;
pub mod auth;
pub mod error;
pub mod router;
pub mod shutdown;
pub mod state;
pub use auth::ActiveCaContext;
pub use auth::AuthorizationErrorResponse;
pub use auth::HasAuthorizationService;
pub use auth::PermissionRequirement;
pub use auth::RequirePermission;
pub use auth::require_permission_middleware;
pub mod logging;
pub mod request_id;
pub mod trace;
