//! Caso de uso: consulta individual de pergunta e resposta frequente (FAQ).
//!
//! Acesso liberado para estudantes e gestores autenticados.
//! Valida a existência do Centro Acadêmico e sua situação ativa, bem como a
//! existência e pertencimento da pergunta frequente ao Centro Acadêmico especificado.

use uuid::Uuid;

use crate::modules::faq::{
    application::{
        error::FaqApplicationError, ports::FaqRepository, queries::get_faq_entry::GetFaqEntryQuery,
    },
    domain::entities::faq_entry::FaqEntry,
};

/// Serviço que coordena a recuperação de uma pergunta frequente específica por id.
pub struct GetFaqEntryUseCase<R> {
    repository: R,
}

impl<R> GetFaqEntryUseCase<R>
where
    R: FaqRepository,
{
    /// Cria o caso de uso com o repositório fornecido.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Executa a busca de uma pergunta frequente individual.
    ///
    /// # Regras de negócio
    ///
    /// 1. O Centro Acadêmico deve existir (`404 academic_center_not_found`).
    /// 2. O Centro Acadêmico deve estar ativo (`422 academic_center_inactive`).
    /// 3. A pergunta frequente deve existir e pertencer ao CA informado (`404 faq_entry_not_found`).
    pub async fn execute(&self, query: GetFaqEntryQuery) -> Result<FaqEntry, FaqApplicationError> {
        let access = self
            .repository
            .check_ca_management_access(query.academic_center_id, Uuid::nil())
            .await?;

        if !access.ca_exists {
            return Err(FaqApplicationError::AcademicCenterNotFound(
                query.academic_center_id,
            ));
        }

        if !access.ca_active {
            return Err(FaqApplicationError::AcademicCenterInactive(
                query.academic_center_id,
            ));
        }

        let entry = self.repository.find_by_id(query.faq_entry_id).await?;
        let Some(entry) = entry else {
            return Err(FaqApplicationError::FaqEntryNotFound(query.faq_entry_id));
        };

        if entry.academic_center_id() != query.academic_center_id {
            return Err(FaqApplicationError::FaqEntryNotFound(query.faq_entry_id));
        }

        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::faq::infrastructure::repository::InMemoryFaqRepository;
    use chrono::Utc;

    const CA_COMPUTACAO: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000010");
    const CA_MATEMATICA: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000020");
    const AUTHOR_ID: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000100");

    #[tokio::test]
    async fn should_get_faq_entry_successfully() {
        let repo = InMemoryFaqRepository::new();
        repo.add_ca(CA_COMPUTACAO, true, Some(AUTHOR_ID));

        let entry_id = Uuid::new_v4();
        let entry = FaqEntry::new(
            entry_id,
            CA_COMPUTACAO,
            "Como solicitar apoio acadêmico?".to_string(),
            "Entre em contato com a gestão do CA.".to_string(),
            Some("Apoio Acadêmico".to_string()),
            AUTHOR_ID,
            Utc::now(),
        )
        .unwrap();
        repo.save(&entry).await.unwrap();

        let use_case = GetFaqEntryUseCase::new(&repo);
        let found = use_case
            .execute(GetFaqEntryQuery::new(CA_COMPUTACAO, entry_id))
            .await
            .expect("deve encontrar entrada");

        assert_eq!(found.id(), entry_id);
        assert_eq!(found.question(), "Como solicitar apoio acadêmico?");
        assert_eq!(found.answer(), "Entre em contato com a gestão do CA.");
    }

    #[tokio::test]
    async fn should_reject_when_entry_not_found() {
        let repo = InMemoryFaqRepository::new();
        repo.add_ca(CA_COMPUTACAO, true, Some(AUTHOR_ID));

        let use_case = GetFaqEntryUseCase::new(&repo);
        let nonexistent_entry = Uuid::new_v4();
        let err = use_case
            .execute(GetFaqEntryQuery::new(CA_COMPUTACAO, nonexistent_entry))
            .await
            .unwrap_err();

        assert_eq!(
            err,
            FaqApplicationError::FaqEntryNotFound(nonexistent_entry)
        );
    }

    #[tokio::test]
    async fn should_reject_when_entry_belongs_to_different_ca() {
        let repo = InMemoryFaqRepository::new();
        repo.add_ca(CA_COMPUTACAO, true, Some(AUTHOR_ID));
        repo.add_ca(CA_MATEMATICA, true, None);

        let entry_id = Uuid::new_v4();
        let entry = FaqEntry::new(
            entry_id,
            CA_MATEMATICA,
            "Pergunta de Matemática?".to_string(),
            "Resposta.".to_string(),
            None,
            AUTHOR_ID,
            Utc::now(),
        )
        .unwrap();
        repo.save(&entry).await.unwrap();

        let use_case = GetFaqEntryUseCase::new(&repo);
        // Tenta acessar a pergunta de Matemática pelo CA de Computação
        let err = use_case
            .execute(GetFaqEntryQuery::new(CA_COMPUTACAO, entry_id))
            .await
            .unwrap_err();

        assert_eq!(err, FaqApplicationError::FaqEntryNotFound(entry_id));
    }

    #[tokio::test]
    async fn should_reject_when_ca_not_found() {
        let repo = InMemoryFaqRepository::new();
        let unknown = Uuid::new_v4();

        let use_case = GetFaqEntryUseCase::new(&repo);
        let err = use_case
            .execute(GetFaqEntryQuery::new(unknown, Uuid::new_v4()))
            .await
            .unwrap_err();

        assert_eq!(err, FaqApplicationError::AcademicCenterNotFound(unknown));
    }

    #[tokio::test]
    async fn should_reject_when_ca_inactive() {
        let repo = InMemoryFaqRepository::new();
        let inactive = Uuid::new_v4();
        repo.add_ca(inactive, false, None);

        let use_case = GetFaqEntryUseCase::new(&repo);
        let err = use_case
            .execute(GetFaqEntryQuery::new(inactive, Uuid::new_v4()))
            .await
            .unwrap_err();

        assert_eq!(err, FaqApplicationError::AcademicCenterInactive(inactive));
    }
}
