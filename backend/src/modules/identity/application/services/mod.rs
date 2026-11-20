//! Serviços de aplicação implementados como funções livres.
//!
//! [`authentication`] coordena verificação institucional com persistência e
//! reutilização da evidência; [`accounts`] cuida da restauração e perfil;
//! [`academic_data`] gerencia consulta e associação das informações acadêmicas.

pub mod academic_data;
pub mod accounts;
pub mod authentication;
