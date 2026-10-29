//! Verificação de Firebase ID tokens com chaves públicas compartilhadas.
//!
//! [`FirebaseTokenVerifier`] valida assinatura RS256, audiência, emissor e claims
//! temporais. O cache contém somente chaves de assinatura; não guarda sessões,
//! ID tokens ou confirmações de vínculo. Não consulta revogação nem situação atual
//! da conta e não configura o fluxo de login do mobile.

mod verifier;

pub use verifier::FirebaseTokenVerifier;
