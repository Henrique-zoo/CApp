//! Entidades usadas na verificação institucional.
//!
//! [`institution`] configura o tenant e o grupo de estudantes.
//! [`institutional_verification`] registra evidência por UID Firebase, independente
//! de provisionamento de usuário interno. Os derives SQLx mapeiam linhas para esses
//! tipos; não aplicam migrations nem consultam o banco.

pub mod institution;
pub mod institutional_verification;
