//! Estrutura do módulo de identidade institucional.
//!
//! O escopo previsto abrange autenticação, provisionamento e dados acadêmicos. A identidade do usuário
//! não representa pertencimento a um Centro Acadêmico.
//!
//! As camadas [`api`], [`application`], [`domain`] e [`infrastructure`] ainda
//! são espaços reservados; este módulo não implementa esses comportamentos.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
