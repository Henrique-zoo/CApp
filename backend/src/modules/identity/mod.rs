//! Módulo de identidade institucional.
//!
//! A implementação atual valida a identidade autenticada por meio de
//! `GET /identity/me`. A identidade retornada não comprova pertencimento a um
//! Centro Acadêmico.
//!
//! O provisionamento de perfis, a associação acadêmica e a autorização dos
//! recursos de negócio ainda não estão implementados. A restrição ao tenant
//! institucional depende da configuração do Firebase Authentication e do
//! Microsoft Entra ID.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
