//! Casos de uso de escrita e modificação de estado de Centros Acadêmicos.

mod configure_initial_mandate;
mod manage_mandate;
pub use close_poll::*;
pub use configure_initial_mandate::*;
pub use create_poll::*;
pub use manage_mandate::*;
pub use vote_poll::*;
pub mod close_poll;
pub mod create_poll;
pub mod vote_poll;
