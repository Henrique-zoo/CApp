//! Módulo de materiais acadêmicos.
//!
//! O domínio representa materiais disponibilizados por Centros Acadêmicos,
//! associados a disciplinas e autoria, com conteúdo fornecido por arquivos
//! armazenados ou links externos.
//!
//! A aplicação coordena publicação, autorização e recuperação. A persistência
//! guarda os metadados no PostgreSQL e os PDFs no armazenamento privado.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
