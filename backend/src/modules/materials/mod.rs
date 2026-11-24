//! Módulo de materiais acadêmicos.
//!
//! O domínio representa materiais disponibilizados por Centros Acadêmicos,
//! associados a disciplinas e autoria, com conteúdo fornecido por arquivos
//! armazenados ou links externos.
//!
//! A camada [`domain`] contém as entidades, os estados e as regras fundamentais
//! de validação dos materiais.
//!
//! As camadas [`api`], [`application`] e [`infrastructure`] reservam,
//! respectivamente, os pontos de integração HTTP, os casos de uso e os
//! adaptadores de persistência. Essas funcionalidades ainda não estão
//! implementadas neste módulo.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
