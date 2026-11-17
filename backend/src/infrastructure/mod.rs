//! Infraestrutura técnica compartilhada entre os domínios.
//!
//! [`database`] fornece migrations e o adaptador PostgreSQL comum aos repositórios.
//! [`microsoft`] permanece reservado. Os adaptadores executáveis de identidade
//! de usuários e de aplicativos (Firebase Auth e App Check) ficam em
//! [`crate::modules::identity::infrastructure`]. [`storage`] e [`notifications`]
//! também são espaços reservados.
//!
//! O executável constrói os recursos e os entrega em [`crate::api::state::AppState`].

pub mod database;
pub mod microsoft;
pub mod notifications;
pub mod storage;
