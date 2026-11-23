//! Persistência transacional e exportação pública do quadro eleitoral.
//!
//! [`codec`] converte valores binários para as representações textuais do contrato.
//! [`repository`] persiste o quadro e fornece a exportação pública.

pub mod codec;
mod registration;
pub mod repository;

pub use registration::{PgVoterRegistry, WireVoterKeyProofVerifier};
pub use repository::PgElectionBoard;
