//! Composição HTTP, observabilidade e estado compartilhado da aplicação.
//!
//! [`router::create_router`] registra as verificações de saúde, prontidão e middlewares
//! de rastreabilidade e correlação de requisições ([`request_id`] e [`trace`]).
//! O chamador fornece [`state::AppState`] com `Router::with_state`, permitindo
//! usar o mesmo router no servidor e nos testes sem abrir uma porta HTTP.
//!
//! # Módulos
//!
//! - [`logging`]: inicialização e formatação configurável de logs estruturados.
//! - [`request_id`]: interceptação, geração e injeção do cabeçalho `x-request-id`.
//! - [`trace`]: spans de rastreabilidade HTTP e registro de latência/status via `TraceLayer`.
//! - [`router`]: montagem e composição das rotas HTTP da aplicação.
//! - [`state`]: estado compartilhado e conexões de infraestrutura.

pub mod logging;
pub mod request_id;
pub mod router;
pub mod state;
pub mod trace;
