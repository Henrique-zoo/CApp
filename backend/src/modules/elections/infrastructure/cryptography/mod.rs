//! Implementações criptográficas de portas da aplicação.
//!
//! O JSON de entrada é desserializado na API. Aqui existe apenas a
//! codificação determinística exigida pela assinatura do protocolo.

pub(super) mod authority_codec;
mod ballot_codec;
mod ballot_verifier;
mod board_evidence;
mod encoding;
mod registration_codec;
mod registration_verifier;
mod rsa_key;

pub use ballot_verifier::RsaPssBallotVerifier;
pub use registration_verifier::RsaPssVoterKeyProofVerifier;
