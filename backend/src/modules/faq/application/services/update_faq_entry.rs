//! Caso de uso: atualização de pergunta e resposta frequente (FAQ).
//!
//! Apenas membros com cargo de gestão ativo no respectivo Centro Acadêmico
//! possuem autorização para atualizar perguntas e respostas da sua própria base.

use chrono::Utc;

use crate::modules::faq::{
    application::{
        commands::update_faq_entry::UpdateFaqEntryCommand, error::FaqApplicationError,
        ports::FaqRepository,
    },
    domain::entities::faq_entry::FaqEntry,
};

/// Serviço que coordena o caso de uso de atualização de entradas de FAQ.
pub struct UpdateFaqEntryUseCase<R> {
    repository: R,
}

impl<R> UpdateFaqEntryUseCase<R>
where
    R: FaqRepository,
{
    /// Cria o caso de uso com o repositório fornecido.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Executa a atualização de uma entrada de FAQ existente.
    ///
    /// # Regras de negócio
    ///
    /// 1. O Centro Acadêmico deve existir (`404 academic_center_not_found`).
    /// 2. O Centro Acadêmico deve estar ativo (`422 academic_center_inactive`).
    /// 3. O autor deve possuir cargo de gestão ativo no CA informado (`403 user_not_permitted_in_ca`).
    /// 4. A pergunta frequente deve existir e pertencer ao CA informado (`404 faq_entry_not_found`).
    /// 5. O novo conteúdo da pergunta, resposta e categoria deve respeitar as invariantes do domínio.
    pub async fn execute(
        &self,
        command: UpdateFaqEntryCommand,
    ) -> Result<FaqEntry, FaqApplicationError> {
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
        let Some(mut entry) = entry else {
            return Err(FaqApplicationError::FaqEntryNotFound(command.faq_entry_id));
        };

        if entry.academic_center_id() != command.academic_center_id {
            return Err(FaqApplicationError::FaqEntryNotFound(command.faq_entry_id));
        }

        let new_question = command
            .question
            .unwrap_or_else(|| entry.question().to_string());
        let new_answer = command.answer.unwrap_or_else(|| entry.answer().to_string());
        let new_category = match command.category {
            Some(cat) => Some(cat),
            None => entry.category().map(String::from),
        };

        entry.update_content(
            new_question,
            new_answer,
            new_category,
            command.author_id,
            Utc::now(),
        )?;

        self.repository.update(&entry).await?;

        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::faq::{
        domain::entities::faq_entry::FaqError, infrastructure::repository::InMemoryFaqRepository,
    };
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
    async fn should_update_faq_entry_successfully() {
        let (repo, entry) = setup_repository().await;
        let use_case = UpdateFaqEntryUseCase::new(&repo);

        let command = UpdateFaqEntryCommand::new(
            CA_COMPUTACAO,
            entry.id(),
            GESTOR_COMPUTACAO,
            Some("Como solicitar apoio pedagógico?".to_string()),
            Some("Envie mensagem pelo canal oficial.".to_string()),
            Some("Pedagógico".to_string()),
        );

        let result = use_case.execute(command).await.expect("deve atualizar");
        assert_eq!(result.id(), entry.id());
        assert_eq!(result.question(), "Como solicitar apoio pedagógico?");
        assert_eq!(result.answer(), "Envie mensagem pelo canal oficial.");
        assert_eq!(result.category(), Some("Pedagógico"));
        assert_eq!(result.updated_by(), GESTOR_COMPUTACAO);

        let persisted = repo.find_by_id(entry.id()).await.unwrap().unwrap();
        assert_eq!(persisted.question(), "Como solicitar apoio pedagógico?");
    }

    #[tokio::test]
    async fn should_update_only_question_keeping_existing_answer_and_category() {
        let (repo, entry) = setup_repository().await;
        let use_case = UpdateFaqEntryUseCase::new(&repo);

        let command = UpdateFaqEntryCommand::new(
            CA_COMPUTACAO,
            entry.id(),
            GESTOR_COMPUTACAO,
            Some("Nova pergunta apenas?".to_string()),
            None,
            None,
        );

        let result = use_case.execute(command).await.expect("deve atualizar");
        assert_eq!(result.question(), "Nova pergunta apenas?");
        assert_eq!(result.answer(), entry.answer());
        assert_eq!(result.category(), entry.category());
    }

    #[tokio::test]
    async fn should_update_only_answer_keeping_existing_question_and_category() {
        let (repo, entry) = setup_repository().await;
        let use_case = UpdateFaqEntryUseCase::new(&repo);

        let command = UpdateFaqEntryCommand::new(
            CA_COMPUTACAO,
            entry.id(),
            GESTOR_COMPUTACAO,
            None,
            Some("Nova resposta apenas.".to_string()),
            None,
        );

        let result = use_case.execute(command).await.expect("deve atualizar");
        assert_eq!(result.question(), entry.question());
        assert_eq!(result.answer(), "Nova resposta apenas.");
        assert_eq!(result.category(), entry.category());
    }

    #[tokio::test]
    async fn should_reject_update_when_ca_does_not_exist() {
        let (repo, entry) = setup_repository().await;
        let use_case = UpdateFaqEntryUseCase::new(&repo);
        let unknown_ca = Uuid::new_v4();

        let command = UpdateFaqEntryCommand::new(
            unknown_ca,
            entry.id(),
            GESTOR_COMPUTACAO,
            Some("Nova pergunta?".to_string()),
            None,
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(err, FaqApplicationError::AcademicCenterNotFound(unknown_ca));
    }

    #[tokio::test]
    async fn should_reject_update_when_ca_is_inactive() {
        let (repo, entry) = setup_repository().await;
        let inactive_ca = Uuid::new_v4();
        repo.add_ca(inactive_ca, false, Some(GESTOR_COMPUTACAO));
        let use_case = UpdateFaqEntryUseCase::new(&repo);

        let command = UpdateFaqEntryCommand::new(
            inactive_ca,
            entry.id(),
            GESTOR_COMPUTACAO,
            Some("Nova pergunta?".to_string()),
            None,
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(
            err,
            FaqApplicationError::AcademicCenterInactive(inactive_ca)
        );
    }

    #[tokio::test]
    async fn should_reject_update_when_user_is_not_ca_manager() {
        let (repo, entry) = setup_repository().await;
        let use_case = UpdateFaqEntryUseCase::new(&repo);

        let command = UpdateFaqEntryCommand::new(
            CA_COMPUTACAO,
            entry.id(),
            ESTUDANTE_COMUM,
            Some("Nova pergunta?".to_string()),
            None,
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(err, FaqApplicationError::UserNotPermittedInCa);
    }

    #[tokio::test]
    async fn should_reject_update_when_entry_does_not_exist() {
        let (repo, _entry) = setup_repository().await;
        let use_case = UpdateFaqEntryUseCase::new(&repo);
        let unknown_entry_id = Uuid::new_v4();

        let command = UpdateFaqEntryCommand::new(
            CA_COMPUTACAO,
            unknown_entry_id,
            GESTOR_COMPUTACAO,
            Some("Nova pergunta?".to_string()),
            None,
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(err, FaqApplicationError::FaqEntryNotFound(unknown_entry_id));
    }

    #[tokio::test]
    async fn should_reject_update_when_entry_belongs_to_another_ca() {
        let (repo, entry_computacao) = setup_repository().await;
        let use_case = UpdateFaqEntryUseCase::new(&repo);

        // Gestor de matemática tenta editar a entrada de computação passando seu CA de matemática
        let command = UpdateFaqEntryCommand::new(
            CA_MATEMATICA,
            entry_computacao.id(),
            GESTOR_MATEMATICA,
            Some("Invasão de FAQ?".to_string()),
            None,
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(
            err,
            FaqApplicationError::FaqEntryNotFound(entry_computacao.id())
        );
    }

    #[tokio::test]
    async fn should_reject_invalid_domain_fields_on_update() {
        let (repo, entry) = setup_repository().await;
        let use_case = UpdateFaqEntryUseCase::new(&repo);

        let command = UpdateFaqEntryCommand::new(
            CA_COMPUTACAO,
            entry.id(),
            GESTOR_COMPUTACAO,
            Some("   ".to_string()),
            None,
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(
            err,
            FaqApplicationError::InvalidEntry(FaqError::EmptyQuestion)
        );
    }
}
