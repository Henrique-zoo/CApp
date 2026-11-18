//! Integração com Microsoft Graph para perfil, tenant e vínculo estudantil.
//!
//! [`MicrosoftGraphClient`] usa um cliente HTTP reutilizável e recebe a credencial
//! por operação. O grupo vem da instituição consultada pelo serviço, não de uma
//! variável de ambiente do adaptador. DTOs do protocolo permanecem privados.

mod client;
mod dto;

pub use client::MicrosoftGraphClient;
