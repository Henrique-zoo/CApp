//! Caso de uso: remoção de pergunta e resposta frequente (FAQ).
//!
//! Apenas membros com cargo de gestão ativo no respectivo Centro Acadêmico
//! possuem autorização para remover perguntas e respostas da sua base.

use crate::modules::faq::application::{
    commands::delete_faq_entry::DeleteFaqEntryCommand, error::FaqApplicationError,
    ports::FaqRepository,
};

/// Serviço que coordena o caso de uso de remoção de entradas de FAQ.
pub struct DeleteFaqEntryUseCase<R> {
    repository: R,
}

impl<R> DeleteFaqEntryUseCase<R>
where
    R: FaqRepository,
{
    /// Cria o caso de uso com o repositório fornecido.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Executa a remoção de uma entrada de FAQ existente.
    ///
    /// # Regras de negócio
    ///
    /// 1. O Centro Acadêmico deve existir (`404 academic_center_not_found`).
    /// 2. O Centro Acadêmico deve estar ativo (`422 academic_center_inactive`).
    /// 3. O autor deve possuir cargo de gestão ativo no CA informado (`403 user_not_permitted_in_ca`).
    /// 4. A pergunta frequente deve existir e pertencer ao CA informado (`404 faq_entry_not_found`).
    pub async fn execute(&self, command: DeleteFaqEntryCommand) -> Result<(), FaqApplicationError> {
        let access = self
            .repository
            .check_ca_management_access(command.academic_center_id, command.author_id)
            .await?;

        if !access.ca_exists {
            return Err(FaqApplicationError::AcademicCenterNotFound(
                command.academic_center_id,
            ));
        }

        if !access.ca_active {
            return Err(FaqApplicationError::AcademicCenterInactive(
                command.academic_center_id,
            ));
        }

        if !access.is_ca_manager {
            return Err(FaqApplicationError::UserNotPermittedInCa);
        }

        let entry = self.repository.find_by_id(command.faq_entry_id).await?;
        let Some(entry) = entry else {
            return Err(FaqApplicationError::FaqEntryNotFound(command.faq_entry_id));
        };

        if entry.academic_center_id() != command.academic_center_id {
            return Err(FaqApplicationError::FaqEntryNotFound(command.faq_entry_id));
        }

        self.repository.delete(&command.faq_entry_id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::faq::{
        domain::entities::faq_entry::FaqEntry, infrastructure::repository::InMemoryFaqRepository,
    };
    use chrono::Utc;
    use uuid::Uuid;

    const CA_COMPUTACAO: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000010");
    const CA_MATEMATICA: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000020");
    const GESTOR_COMPUTACAO: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000100");
    const GESTOR_MATEMATICA: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000101");
    const ESTUDANTE_COMUM: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000200");

    async fn setup_repository() -> (InMemoryFaqRepository, FaqEntry) {
        let repo = InMemoryFaqRepository::new();
        repo.add_ca(CA_COMPUTACAO, true, Some(GESTOR_COMPUTACAO));
        repo.add_ca(CA_MATEMATICA, true, Some(GESTOR_MATEMATICA));

        let entry = FaqEntry::new(
            Uuid::new_v4(),
            CA_COMPUTACAO,
            "Como solicitar apoio acadêmico?".to_string(),
            "Entre em contato com a gestão do CA.".to_string(),
            Some("Apoio Acadêmico".to_string()),
            GESTOR_COMPUTACAO,
            Utc::now(),
        )
        .expect("criação válida");

        repo.save(&entry).await.unwrap();
        (repo, entry)
    }

    #[tokio::test]
    async fn should_delete_faq_entry_successfully() {
        let (repo, entry) = setup_repository().await;
        let use_case = DeleteFaqEntryUseCase::new(&repo);

        let command = DeleteFaqEntryCommand::new(CA_COMPUTACAO, entry.id(), GESTOR_COMPUTACAO);
        use_case.execute(command).await.expect("deve remover");

        let loaded = repo.find_by_id(entry.id()).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn should_reject_delete_when_ca_does_not_exist() {
        let (repo, entry) = setup_repository().await;
        let use_case = DeleteFaqEntryUseCase::new(&repo);
        let unknown_ca = Uuid::new_v4();

        let command = DeleteFaqEntryCommand::new(unknown_ca, entry.id(), GESTOR_COMPUTACAO);
        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(err, FaqApplicationError::AcademicCenterNotFound(unknown_ca));
    }

    #[tokio::test]
    async fn should_reject_delete_when_ca_is_inactive() {
        let (repo, entry) = setup_repository().await;
        let inactive_ca = Uuid::new_v4();
        repo.add_ca(inactive_ca, false, Some(GESTOR_COMPUTACAO));
        let use_case = DeleteFaqEntryUseCase::new(&repo);

        let command = DeleteFaqEntryCommand::new(inactive_ca, entry.id(), GESTOR_COMPUTACAO);
        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(
            err,
            FaqApplicationError::AcademicCenterInactive(inactive_ca)
        );
    }

    #[tokio::test]
    async fn should_reject_delete_when_user_is_not_ca_manager() {
        let (repo, entry) = setup_repository().await;
        let use_case = DeleteFaqEntryUseCase::new(&repo);

        let command = DeleteFaqEntryCommand::new(CA_COMPUTACAO, entry.id(), ESTUDANTE_COMUM);
        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(err, FaqApplicationError::UserNotPermittedInCa);
    }

    #[tokio::test]
    async fn should_reject_delete_when_entry_does_not_exist() {
        let (repo, _entry) = setup_repository().await;
        let use_case = DeleteFaqEntryUseCase::new(&repo);
        let unknown_entry_id = Uuid::new_v4();

        let command =
            DeleteFaqEntryCommand::new(CA_COMPUTACAO, unknown_entry_id, GESTOR_COMPUTACAO);
        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(err, FaqApplicationError::FaqEntryNotFound(unknown_entry_id));
    }

    #[tokio::test]
    async fn should_reject_delete_when_entry_belongs_to_another_ca() {
        let (repo, entry_computacao) = setup_repository().await;
        let use_case = DeleteFaqEntryUseCase::new(&repo);

        // Gestor de matemática tenta deletar pergunta de computação informando o CA de matemática
        let command =
            DeleteFaqEntryCommand::new(CA_MATEMATICA, entry_computacao.id(), GESTOR_MATEMATICA);
        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(
            err,
            FaqApplicationError::FaqEntryNotFound(entry_computacao.id())
        );
    }
}
