//! Entidades do módulo de Centros Acadêmicos.
//!
//! Reúne os modelos de domínio do Centro Acadêmico, mandatos de gestão,
//! ciclo de vida, invariantes e integridade estrutural.

mod academic_center;
mod mandate;

pub use academic_center::*;
pub use mandate::*;
