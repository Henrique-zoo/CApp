//! Política abrangente de retenção, minimização e descarte seguro de dados sensíveis.
//!
//! Este módulo centraliza os modelos, classificações de sensibilidade, prazos legais
//! de expiração e rotinas de sanitização/mascaramento de dados do CApp, em conformidade
//! com a LGPD (Lei 13.709/2018), o Marco Civil da Internet (Lei 12.965/2014) e as
//! normas acadêmicas da Universidade de Brasília (UnB).

pub mod categories;
pub mod elections;
pub mod error;
pub mod referential;
pub mod sanitizer;
pub mod schedule;

pub use categories::{DataCategory, DisposalMethod, LegalBasis, SensitivityLevel};
pub use elections::{
    AnonymousBallot, ElectoralAuditStatus, ElectoralPrivacyGuardian, TurnoutRecord,
};
pub use error::RetentionError;
pub use referential::{DisposalStrategy, ReferentialIntegrityGuardian, RetentionHold};
pub use sanitizer::{
    APPROVED_MATRICULAS, FORBIDDEN_MATRICULA, PiiSanitizer, StudentRecord, UNB_EMAIL_DOMAIN,
};
pub use schedule::{RetentionPeriod, RetentionSchedule};
