//! Serviços de aplicação implementados como funções livres.
//!
//! [`authentication`] coordena verificação institucional com persistência e
//! reutilização da evidência. Os adaptadores são fornecidos pelo chamador através
//! dos contratos de `application::ports`, sem construção de clientes pelo serviço.

pub mod authentication;
