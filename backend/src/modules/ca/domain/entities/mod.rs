//! Entidades do módulo de Centros Acadêmicos.
//!
<<<<<<< HEAD
//! Reúne os modelos de domínio do Centro Acadêmico, mandatos de gestão,
//! ciclo de vida, invariantes e integridade estrutural.

pub mod academic_center;
mod mandate;
pub use academic_center::*;
pub use mandate::*;
pub use poll::*;
mod poll;
=======
//! Reúne os modelos de domínio do Centro Acadêmico, seu ciclo de vida,
//! invariantes e regras de integridade estrutural.

pub mod academic_center;
pub mod active_context;

pub use academic_center::{
    AcademicCenter, AcademicCenterError, AcademicCenterStatus, CACC_DIDACTIC_ID, CAER_DIDACTIC_ID,
    MAX_ACRONYM_LENGTH, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH,
};
pub use active_context::{
    ActiveCaContext, ActiveContextError, ActiveContextSource, ManagementRoleAssignment,
};
>>>>>>> origin/pr/331

pub mod active_context;
pub mod context_center;
pub use active_context::*;
