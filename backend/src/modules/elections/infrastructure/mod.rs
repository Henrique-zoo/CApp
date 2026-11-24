//! Persistência transacional e exportação pública do quadro eleitoral.
//!
//! [`codec`] converte valores binários para as representações textuais do contrato.
//! [`repository`] persiste o quadro e fornece a exportação pública.

mod administration;
pub mod codec;
mod cryptography;
mod preparation;
mod registration;
pub mod repository;
mod tally;

pub use cryptography::{RsaPssBallotVerifier, RsaPssVoterKeyProofVerifier};
pub use registration::PgVoterRegistrationRepository;
pub use repository::PgElectionBoardRepository;
