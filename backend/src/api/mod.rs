//! Composição HTTP, erros públicos e estado compartilhado da aplicação.
//!
//! [`router::create_router`] combina as rotas de saúde, prontidão e identidade.
//! O chamador fornece [`state::AppState`] por `Router::with_state`, permitindo usar
//! o mesmo router no servidor e nos testes em memória.
//! [`error::ApiError`] define o envelope JSON usado nos erros de autenticação.

pub mod error;
pub mod router;
pub mod state;
