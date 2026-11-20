//! Entidades do domínio de identidade e dados acadêmicos.

pub mod academic_data;
pub mod institution;
pub mod institutional_verification;
pub mod user;

pub use academic_data::*;
pub use institution::*;
pub use institutional_verification::*;
pub use user::*;
