//! Camada de domínio do módulo de Centros Acadêmicos.
//!
//! [`entities`], [`rules`] e [`events`] definem os modelos, suas invariantes
//! e os fatos relevantes do negócio. Essa camada permanece independente
//! de HTTP e banco de dados.

pub mod entities;
pub mod events;
pub mod rules;

pub use entities::{
    AcademicCenter, AcademicCenterError, AcademicCenterStatus, CACC_DIDACTIC_ID, CAER_DIDACTIC_ID,
    MAX_ACRONYM_LENGTH, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH,
};
