//! Infraestrutura técnica compartilhada entre os domínios.
//!
//! [`database`] fornece migrations e o adaptador PostgreSQL comum aos repositórios.
//! [`microsoft`] permanece reservado. Os adaptadores executáveis de identidade
//! de usuários e de aplicativos (Firebase Auth e App Check) ficam em
//! [`crate::modules::identity::infrastructure`]. [`storage`] e [`notifications`]
//! também são espaços reservados.
//!
//! O executável entrega os recursos da aplicação em [`crate::api::state::AppState`]
//! e usa [`tls`] para configurar a escuta HTTPS.

pub mod database;
pub mod microsoft;
pub mod notifications;
pub mod storage;
pub mod tls;
