//! Modelos institucionais, dados acadêmicos e evidências de elegibilidade.
//!
//! [`entities`] contém instituição, usuário, dados acadêmicos e verificação institucional;
//! [`rules`] encapsula as invariantes e validações de negócio;
//! [`events`] declara fatos relevantes do domínio;
//! [`status`] define os ciclos de vida e estados cadastrais.
//!
//! A camada de domínio é pura, agnóstica a bancos de dados, frameworks ou formatos de serialização.

pub mod entities;
pub mod events;
pub mod rules;
mod status;

pub use status::{CatalogStatus, CatalogStatusError, UserStatus, UserStatusError};
