//! Estrutura do módulo de Centros Acadêmicos.
//!
//! O escopo previsto abrange cadastro de CAs, perfis administrativos, cargos e permissões. Um usuário
//! pode ter perfis administrativos distintos em diferentes CAs.
//!
//! As camadas [`api`], [`application`], [`domain`] e [`infrastructure`] ainda
//! são espaços reservados; este módulo não implementa esses comportamentos.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
