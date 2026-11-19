//! Regras e invariantes de domínio dos materiais acadêmicos.

mod material_rules;

pub(crate) use material_rules::{validate_external_link, validate_title};
