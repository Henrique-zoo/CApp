//! Espaços reservados para integrações técnicas compartilhadas entre domínios.
//!
//! Os submódulos delimitam acesso ao banco, provedores de identidade, dados
//! acadêmicos, arquivos e notificações. Ainda não contêm adaptadores executáveis.
//! A conexão PostgreSQL atual é criada no executável e entregue à API pelo
//! [`AppState`](crate::api::state::AppState).

pub mod database;
pub mod firebase;
pub mod microsoft;
pub mod notifications;
pub mod storage;
