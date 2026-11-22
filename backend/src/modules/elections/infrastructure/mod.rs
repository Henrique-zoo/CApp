//! Persistência transacional e exportação pública do quadro eleitoral.

mod registration;
pub mod repository;

pub use registration::PgVoterRegistry;
pub use repository::PgElectionBoard;
