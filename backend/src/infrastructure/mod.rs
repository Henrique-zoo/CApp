//! Infraestrutura técnica compartilhada entre os domínios.
//!
//! [`database`] fornece migrations e o adaptador PostgreSQL comum aos repositórios.
//! Os diretórios globais [`firebase`] e [`microsoft`] permanecem reservados;
//! os adaptadores executáveis de autenticação ficam em
//! [`crate::modules::identity::infrastructure`]. [`storage`] e [`notifications`]
//! também são espaços reservados.
//!
//! O executável constrói os recursos e os entrega em [`crate::api::state::AppState`].

pub mod database;
pub mod firebase;
pub mod microsoft;
pub mod notifications;
pub mod storage;
