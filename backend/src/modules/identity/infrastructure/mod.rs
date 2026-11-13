//! Adaptadores externos usados pelos casos de uso de identidade.
//!
//! [`firebase`] valida identidades de usuários por ID token e de aplicativos por
//! App Check, compartilhando a implementação privada de JWKS; [`microsoft`] consulta o Graph;
//! [`repository`] implementa os contratos de persistência sobre o `PgDatabase`
//! compartilhado. Esses módulos dependem dos contratos de aplicação, sem expor
//! SQLx, Reqwest ou os DTOs dos provedores nas assinaturas desses contratos.

pub mod firebase;
pub mod microsoft;
pub mod repository;
