//! Regras de validação dos materiais acadêmicos.

use crate::modules::materials::domain::errors::StudyMaterialError;

/// Verifica se o título contém algum caractere diferente de espaço.
///
/// A normalização definitiva é realizada pela entidade.
pub(crate) fn validate_title(title: &str) -> Result<(), StudyMaterialError> {
    if title.trim().is_empty() {
        return Err(StudyMaterialError::EmptyTitle);
    }

    Ok(())
}

/// Verifica se o endereço externo contém algum caractere diferente de espaço.
///
/// A regra não verifica esquema, acessibilidade ou destino remoto.
pub(crate) fn validate_external_link(url: &str) -> Result<(), StudyMaterialError> {
    if url.trim().is_empty() {
        return Err(StudyMaterialError::EmptyExternalLink);
    }

    Ok(())
}
