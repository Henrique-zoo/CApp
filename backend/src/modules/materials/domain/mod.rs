//! Domínio de materiais acadêmicos.
//!
//! [`entities`] contém os modelos e seus estados;
//! [`rules`] implementa as invariantes de negócio;
//! [`errors`] reúne os erros compartilhados do domínio.
//!
//! Esta camada não executa operações HTTP nem consultas ao banco de dados.

pub mod entities;
pub mod errors;
pub mod events;
pub mod rules;

pub use errors::StudyMaterialError;
