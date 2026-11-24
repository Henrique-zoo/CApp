//! Casos de uso de consulta e leitura de Centros Acadêmicos.

mod mandate;
pub use get_poll::*;
pub use list_polls::*;
pub use mandate::*;
pub mod get_poll;
pub mod list_polls;
