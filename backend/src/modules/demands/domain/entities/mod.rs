//! Entidades do módulo de demandas.
//!
//! Reúne os modelos de domínio de demanda acadêmica, histórico de alterações,
//! fóruns e estados do ciclo de vida.

mod comment;
mod demand;
mod notification;

pub use comment::*;
pub use demand::*;
pub use notification::*;
