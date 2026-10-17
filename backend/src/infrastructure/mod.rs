//! Infraestrutura técnica compartilhada entre os domínios.
//!
//! [`database`] fornece as migrations usadas pelo servidor e pelos testes.
//! [`firebase`] contém o adaptador de validação dos Firebase ID tokens.
//! Os módulos [`microsoft`], [`notifications`] e [`storage`] reservam outras
//! integrações ainda não implementadas.
//! A conexão PostgreSQL atual é criada no executável e entregue à API pelo
//! [`AppState`](crate::api::state::AppState).

pub mod database;
pub mod firebase;
pub mod microsoft;
pub mod notifications;
pub mod storage;
