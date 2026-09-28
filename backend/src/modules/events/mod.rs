//! Estrutura do módulo de eventos.
//!
//! O escopo previsto abrange publicação e homologação de eventos, inscrições e controle de vagas.
//!
//! As camadas [`api`], [`application`], [`domain`] e [`infrastructure`] ainda
//! são espaços reservados; este módulo não implementa esses comportamentos.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
