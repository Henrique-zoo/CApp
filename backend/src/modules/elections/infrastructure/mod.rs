//! Persistência transacional e exportação pública do quadro eleitoral.
//!
//! [`codec`] converte valores binários para as representações textuais do contrato.
//! [`repository`] persiste o quadro e fornece a exportação pública.
//! As portas eleitorais usam o mesmo [`PgDatabase`](crate::infrastructure::database::PgDatabase)
//! que o módulo de identidade. Cada implementação permanece no arquivo da operação.

mod administration;
mod clock;
pub mod codec;
mod cryptography;
mod preparation;
mod registration;
pub mod repository;
mod tally;

pub use clock::start_election_clock;
pub use cryptography::{RsaPssBallotVerifier, RsaPssVoterKeyProofVerifier};
