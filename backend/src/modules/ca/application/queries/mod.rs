//! Casos de uso de consulta e leitura de Centros Acadêmicos.

mod mandate;
pub use mandate::*;
pub use get_poll::*;
pub use list_polls::*;
pub mod get_poll;
pub mod list_polls;
