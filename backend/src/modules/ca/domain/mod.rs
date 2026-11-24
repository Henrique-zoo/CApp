//! Camada de domínio prevista para o módulo de Centros Acadêmicos.
//!
//! [`entities`], [`rules`] e [`events`] reservam os modelos, suas invariantes,
//! limites e fatos relevantes do negócio. Essa camada permanece independente
//! de HTTP e banco de dados.

pub mod entities;
pub mod events;
pub mod rules;
pub use entities::AcademicCenter;
pub use entities::AcademicCenterError;
pub use entities::AcademicCenterStatus;
pub use entities::CACC_DIDACTIC_ID;
pub use entities::CAER_DIDACTIC_ID;
pub use entities::MAX_ACRONYM_LENGTH;
pub use entities::MAX_DESCRIPTION_LENGTH;
pub use entities::MAX_NAME_LENGTH;
