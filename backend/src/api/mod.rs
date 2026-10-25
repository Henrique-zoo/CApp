//! Composição HTTP, estado compartilhado e encerramento controlado da aplicação.
//!
//! [`router::create_router`] registra as verificações de saúde e prontidão.
//! O chamador fornece [`state::AppState`] com `Router::with_state`, permitindo
//! usar o mesmo router no servidor e nos testes sem abrir uma porta HTTP.
//!
//! [`shutdown`] fornece as rotinas de tratamento de sinais e execução de encerramento
//! controlado (*graceful shutdown*) com drenagem de conexões e liberação de recursos.

pub mod router;
pub mod shutdown;
pub mod state;
