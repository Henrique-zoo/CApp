//! Persistência transacional e exportação pública do quadro eleitoral.
//!
//! [`codec`] converte valores binários para as representações textuais do contrato.
//! [`repository`] persiste o quadro e fornece a exportação pública.

pub mod codec;
pub mod repository;

pub use repository::PgElectionBoard;
