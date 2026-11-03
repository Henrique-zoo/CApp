//! Modelos institucionais e evidências de elegibilidade persistidas.
//!
//! [`entities`] contém instituição e resultado de verificação; [`CatalogStatus`]
//! representa disponibilidade no catálogo. [`rules`] e [`events`] são reservados.
//!
//! Os tipos não executam SQL nem conhecem HTTP. Os derives `sqlx::FromRow` e
//! `sqlx::Type` constituem o acoplamento de mapeamento adotado pelo projeto;
//! as consultas e o gerenciamento de conexões continuam na infraestrutura.

pub mod entities;
pub mod events;
pub mod rules;
mod status;

pub use status::CatalogStatus;
