//! Caso de uso: busca textual de perguntas e respostas frequentes (FAQ).
//!
//! Acesso liberado para estudantes e gestores autenticados.
//! Valida a existência do Centro Acadêmico e sua situação ativa, garantindo
//! isolamento estrito entre Centros Acadêmicos.

use uuid::Uuid;

use crate::modules::faq::{
    application::{
        error::FaqApplicationError,
        ports::{FaqFilter, FaqRepository},
        queries::search_faq_entries::SearchFaqEntriesQuery,
    },
    domain::entities::faq_entry::FaqEntry,
};

/// Serviço que coordena a busca de perguntas frequentes por palavras-chave.
pub struct SearchFaqEntriesUseCase<R> {
    repository: R,
}

impl<R> SearchFaqEntriesUseCase<R>
where
    R: FaqRepository,
{
    /// Cria o caso de uso com o repositório fornecido.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Executa a busca textual no FAQ de acordo com o termo e categoria opcional.
    ///
    /// # Regras de negócio
    ///
    /// 1. O Centro Acadêmico deve existir (`404 academic_center_not_found`).
    /// 2. O Centro Acadêmico deve estar ativo (`422 academic_center_inactive`).
    /// 3. Retorna lista vazia caso nenhum registro coincida com o termo buscado.
    pub async fn execute(
        &self,
        query: SearchFaqEntriesQuery,
    ) -> Result<Vec<FaqEntry>, FaqApplicationError> {
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

        let filter = FaqFilter::new(query.category, Some(query.search_term));
        let entries = self
            .repository
            .find_by_academic_center_id_with_filter(query.academic_center_id, &filter)
            .await?;

        Ok(entries)
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

    async fn setup_repository() -> InMemoryFaqRepository {
        let repo = InMemoryFaqRepository::new();
        repo.add_ca(CA_COMPUTACAO, true, Some(AUTHOR_ID));
        repo.add_ca(CA_MATEMATICA, true, None);

        let now = Utc::now();
        let e1 = FaqEntry::new(
            Uuid::new_v4(),
            CA_COMPUTACAO,
            "Como solicitar apoio acadêmico?".to_string(),
            "Entre em contato com a gestão do CA.".to_string(),
            Some("Apoio Acadêmico".to_string()),
            AUTHOR_ID,
            now - chrono::Duration::hours(2),
        )
        .unwrap();

        let e2 = FaqEntry::new(
            Uuid::new_v4(),
            CA_COMPUTACAO,
            "Dúvidas sobre matrícula em disciplinas?".to_string(),
            "Acesse o SIGAA e confira os prazos oficiais.".to_string(),
            Some("Matrícula".to_string()),
            AUTHOR_ID,
            now - chrono::Duration::hours(1),
        )
        .unwrap();

        let e3 = FaqEntry::new(
            Uuid::new_v4(),
            CA_MATEMATICA,
            "Instruções para matrícula do curso de Matemática".to_string(),
            "Verifique as disciplinas ofertadas pelo departamento.".to_string(),
            Some("Matrícula".to_string()),
            AUTHOR_ID,
            now,
        )
        .unwrap();

        repo.save(&e1).await.unwrap();
        repo.save(&e2).await.unwrap();
        repo.save(&e3).await.unwrap();

        repo
    }

    #[tokio::test]
    async fn should_search_by_term_matching_question() {
        let repo = setup_repository().await;
        let use_case = SearchFaqEntriesUseCase::new(&repo);

        let query = SearchFaqEntriesQuery::new(CA_COMPUTACAO, "matrícula".to_string(), None);
        let results = use_case
            .execute(query)
            .await
            .expect("busca deve ter sucesso");

        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].question(),
            "Dúvidas sobre matrícula em disciplinas?"
        );
    }

    #[tokio::test]
    async fn should_search_by_term_matching_answer() {
        let repo = setup_repository().await;
        let use_case = SearchFaqEntriesUseCase::new(&repo);

        let query = SearchFaqEntriesQuery::new(CA_COMPUTACAO, "SIGAA".to_string(), None);
        let results = use_case
            .execute(query)
            .await
            .expect("busca deve ter sucesso");

        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].question(),
            "Dúvidas sobre matrícula em disciplinas?"
        );
    }

    #[tokio::test]
    async fn should_search_case_insensitively() {
        let repo = setup_repository().await;
        let use_case = SearchFaqEntriesUseCase::new(&repo);

        let query = SearchFaqEntriesQuery::new(CA_COMPUTACAO, "APOIO ACADÊMICO".to_string(), None);
        let results = use_case
            .execute(query)
            .await
            .expect("busca deve ter sucesso");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].question(), "Como solicitar apoio acadêmico?");
    }

    #[tokio::test]
    async fn should_return_empty_list_when_no_match() {
        let repo = setup_repository().await;
        let use_case = SearchFaqEntriesUseCase::new(&repo);

        let query =
            SearchFaqEntriesQuery::new(CA_COMPUTACAO, "termo inexistente".to_string(), None);
        let results = use_case
            .execute(query)
            .await
            .expect("busca deve retornar lista vazia");

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn should_not_return_matches_from_another_ca() {
        let repo = setup_repository().await;
        let use_case = SearchFaqEntriesUseCase::new(&repo);

        // Termo 'departamento' existe apenas no CA de Matemática
        let query = SearchFaqEntriesQuery::new(CA_COMPUTACAO, "departamento".to_string(), None);
        let results = use_case
            .execute(query)
            .await
            .expect("busca deve isolar CAs");

        assert!(
            results.is_empty(),
            "não deve vazar perguntas do CA de Matemática"
        );
    }

    #[tokio::test]
    async fn should_reject_search_when_ca_not_found() {
        let repo = setup_repository().await;
        let use_case = SearchFaqEntriesUseCase::new(&repo);
        let unknown = Uuid::new_v4();

        let query = SearchFaqEntriesQuery::new(unknown, "apoio".to_string(), None);
        let err = use_case.execute(query).await.unwrap_err();

        assert_eq!(err, FaqApplicationError::AcademicCenterNotFound(unknown));
    }

    #[tokio::test]
    async fn should_reject_search_when_ca_is_inactive() {
        let repo = InMemoryFaqRepository::new();
        let inactive = Uuid::new_v4();
        repo.add_ca(inactive, false, None);

        let use_case = SearchFaqEntriesUseCase::new(&repo);
        let query = SearchFaqEntriesQuery::new(inactive, "apoio".to_string(), None);
        let err = use_case.execute(query).await.unwrap_err();

        assert_eq!(err, FaqApplicationError::AcademicCenterInactive(inactive));
    }
}
