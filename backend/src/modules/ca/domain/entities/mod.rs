//! Entidades do módulo de Centros Acadêmicos.
//!
//! Reúne os modelos de domínio do Centro Acadêmico, seu ciclo de vida,
//! invariantes e regras de integridade estrutural.

pub mod academic_center;

pub use academic_center::{
    AcademicCenter, AcademicCenterError, AcademicCenterStatus, CACC_DIDACTIC_ID, CAER_DIDACTIC_ID,
    MAX_ACRONYM_LENGTH, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH,
};
