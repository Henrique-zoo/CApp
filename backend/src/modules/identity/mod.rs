//! Estrutura do módulo de identidade institucional.
//!
//! O escopo abrange autenticação, provisionamento automático de usuário,
//! identidades externas e preferências padrão. A identidade do usuário
//! não representa pertencimento nem concessão de privilégios em um Centro Acadêmico.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::*;
pub use domain::*;
pub use infrastructure::*;
