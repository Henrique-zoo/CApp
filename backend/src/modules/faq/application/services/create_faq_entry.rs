//! Caso de uso: cadastro de pergunta e resposta frequente (FAQ).
//!
//! Apenas membros com cargo de gestão ativo no respectivo Centro Acadêmico
//! possuem autorização para cadastrar perguntas e respostas.

use chrono::Utc;
use uuid::Uuid;

use crate::modules::faq::{
    application::{
        commands::create_faq_entry::CreateFaqEntryCommand, error::FaqApplicationError,
        ports::FaqRepository,
    },
    domain::entities::faq_entry::FaqEntry,
};

/// Serviço que coordena o caso de uso de cadastro de entradas de FAQ.
pub struct CreateFaqEntryUseCase<R> {
    repository: R,
}

impl<R> CreateFaqEntryUseCase<R>
where
    R: FaqRepository,
{
    /// Cria o caso de uso com o repositório fornecido.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Executa o cadastro de uma nova entrada de FAQ.
    ///
    /// # Regras de negócio
    ///
    /// 1. O Centro Acadêmico deve existir (`404 academic_center_not_found`).
    /// 2. O Centro Acadêmico deve estar ativo (`422 academic_center_inactive`).
    /// 3. O autor deve possuir cargo de gestão ativo no CA informado (`403 user_not_permitted_in_ca`).
    /// 4. O conteúdo da pergunta, resposta e categoria deve respeitar as invariantes do domínio.
    pub async fn execute(
        &self,
        command: CreateFaqEntryCommand,
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

        let entry = FaqEntry::new(
            Uuid::new_v4(),
            command.academic_center_id,
            command.question,
            command.answer,
            command.category,
            command.author_id,
            Utc::now(),
        )?;

        self.repository.save(&entry).await?;

        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::faq::{
        domain::entities::faq_entry::FaqError, infrastructure::repository::InMemoryFaqRepository,
    };

    const CA_COMPUTACAO: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000010");
    const CA_MATEMATICA: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000020");
    const GESTOR_COMPUTACAO: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000100");
    const ESTUDANTE_COMUM: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000200");

    fn setup_repository() -> InMemoryFaqRepository {
        let repo = InMemoryFaqRepository::new();
        // CA de Computação ativo com gestor
        repo.add_ca(CA_COMPUTACAO, true, Some(GESTOR_COMPUTACAO));
        // CA de Matemática ativo sem esse gestor
        repo.add_ca(CA_MATEMATICA, true, None);
        repo
    }

    #[tokio::test]
    async fn should_create_faq_entry_successfully_when_user_is_ca_manager() {
        let repo = setup_repository();
        let use_case = CreateFaqEntryUseCase::new(&repo);

        let command = CreateFaqEntryCommand::new(
            CA_COMPUTACAO,
            GESTOR_COMPUTACAO,
            "Como solicitar apoio acadêmico?".to_string(),
            "Entre em contato com a gestão do CA.".to_string(),
            Some("Apoio Acadêmico".to_string()),
        );

        let result = use_case
            .execute(command)
            .await
            .expect("cadastro deve ter sucesso");
        assert_eq!(result.academic_center_id(), CA_COMPUTACAO);
        assert_eq!(result.question(), "Como solicitar apoio acadêmico?");
        assert_eq!(result.answer(), "Entre em contato com a gestão do CA.");
        assert_eq!(result.category(), Some("Apoio Acadêmico"));
        assert_eq!(result.created_by(), GESTOR_COMPUTACAO);

        let saved = repo
            .find_by_id(result.id())
            .await
            .expect("busca sem erro")
            .expect("deve encontrar entrada salva");
        assert_eq!(saved.question(), "Como solicitar apoio acadêmico?");
    }

    #[tokio::test]
    async fn should_reject_when_ca_does_not_exist() {
        let repo = setup_repository();
        let use_case = CreateFaqEntryUseCase::new(&repo);
        let unknown_ca = Uuid::new_v4();

        let command = CreateFaqEntryCommand::new(
            unknown_ca,
            GESTOR_COMPUTACAO,
            "Pergunta?".to_string(),
            "Resposta.".to_string(),
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(err, FaqApplicationError::AcademicCenterNotFound(unknown_ca));
    }

    #[tokio::test]
    async fn should_reject_when_ca_is_inactive() {
        let repo = InMemoryFaqRepository::new();
        let inactive_ca = Uuid::new_v4();
        repo.add_ca(inactive_ca, false, Some(GESTOR_COMPUTACAO));

        let use_case = CreateFaqEntryUseCase::new(&repo);

        let command = CreateFaqEntryCommand::new(
            inactive_ca,
            GESTOR_COMPUTACAO,
            "Pergunta?".to_string(),
            "Resposta.".to_string(),
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(
            err,
            FaqApplicationError::AcademicCenterInactive(inactive_ca)
        );
    }

    #[tokio::test]
    async fn should_reject_when_user_is_not_ca_manager() {
        let repo = setup_repository();
        let use_case = CreateFaqEntryUseCase::new(&repo);

        let command = CreateFaqEntryCommand::new(
            CA_COMPUTACAO,
            ESTUDANTE_COMUM,
            "Pergunta?".to_string(),
            "Resposta.".to_string(),
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(err, FaqApplicationError::UserNotPermittedInCa);
    }

    #[tokio::test]
    async fn should_enforce_strict_isolation_between_cas() {
        let repo = setup_repository();
        let use_case = CreateFaqEntryUseCase::new(&repo);

        // GESTOR_COMPUTACAO tenta cadastrar FAQ no CA_MATEMATICA
        let command = CreateFaqEntryCommand::new(
            CA_MATEMATICA,
            GESTOR_COMPUTACAO,
            "Pergunta no CA errado?".to_string(),
            "Resposta.".to_string(),
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(err, FaqApplicationError::UserNotPermittedInCa);
    }

    #[tokio::test]
    async fn should_reject_invalid_domain_fields() {
        let repo = setup_repository();
        let use_case = CreateFaqEntryUseCase::new(&repo);

        let command = CreateFaqEntryCommand::new(
            CA_COMPUTACAO,
            GESTOR_COMPUTACAO,
            "   ".to_string(),
            "Resposta.".to_string(),
            None,
        );

        let err = use_case.execute(command).await.unwrap_err();
        assert_eq!(
            err,
            FaqApplicationError::InvalidEntry(FaqError::EmptyQuestion)
        );
    }
}
