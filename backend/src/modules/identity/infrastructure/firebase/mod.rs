//! Identidades de usuários e aplicativos verificadas pelos adaptadores Firebase.
//!
//! [`FirebaseTokenVerifier`] autentica usuários por ID token; [`AppCheckVerifier`]
//! comprova o aplicativo cliente antes das operações HTTP. Os resultados e as
//! regras de claims são separados: um atestado de aplicativo não identifica uma
//! pessoa e um ID token não substitui a comprovação do aplicativo.
//!
//! O módulo privado `jwks.rs` compartilha a implementação de HTTP, cache, rotação
//! e contenção de downloads. Cada adaptador fixa sua origem de chaves e possui
//! um cache independente, com política de validade explícita. Nenhum cache
//! armazena sessões, JWTs ou evidências institucionais. O middleware global da
//! API consome App Check, enquanto os casos de uso consomem identidade do usuário.
//! Os adaptadores não decidem permissões do domínio nem configuram o login mobile.

mod app_check;
mod jwks;
mod verifier;

pub use app_check::{AppCheckError, AppCheckVerifier, VerifiedApp};
pub use verifier::FirebaseTokenVerifier;
