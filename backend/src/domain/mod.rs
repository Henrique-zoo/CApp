//! Modelos e entidades de domínio do backend.
//!
//! Centraliza as definições de domínio do sistema para facilitar a importação
//! e manter a compatibilidade com a estrutura do projeto.

pub mod academic_center;

pub use academic_center::{
    AcademicCenter, AcademicCenterError, AcademicCenterStatus, CACC_DIDACTIC_ID, CAER_DIDACTIC_ID,
    MAX_ACRONYM_LENGTH, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH,
};
