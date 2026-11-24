//! Estrutura do módulo de demandas.
//!
//! O escopo abrange demandas institucionais e solicitações relacionadas ao espaço físico do CA.
//!
//! A camada [`domain`] define os modelos puros, fóruns, histórico e regras de transição.
//! As camadas [`api`], [`application`] e [`infrastructure`] complementam os fluxos do módulo.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
