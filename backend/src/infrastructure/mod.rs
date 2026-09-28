//! Infraestrutura técnica compartilhada entre os domínios.
//!
//! [`database`] fornece as migrations usadas pelo servidor e pelos testes.
//! Os demais submódulos reservam as integrações com provedores de identidade,
//! dados acadêmicos, arquivos e notificações, ainda sem adaptadores executáveis.
//! A conexão PostgreSQL atual é criada no executável e entregue à API pelo
//! [`AppState`](crate::api::state::AppState).

pub mod database;
pub mod firebase;
pub mod microsoft;
pub mod notifications;
pub mod storage;
