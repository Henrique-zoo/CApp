//! Caso de uso: listagem e navegação de perguntas e respostas frequentes (FAQ).
//!
//! Acesso liberado para estudantes e gestores autenticados.
//! Valida a existência do Centro Acadêmico e sua situação ativa, garantindo
//! isolamento estrito entre Centros Acadêmicos.

use uuid::Uuid;

use crate::modules::faq::{
    application::{
        error::FaqApplicationError,
        ports::{FaqFilter, FaqRepository},
        queries::list_faq_entries::ListFaqEntriesQuery,
    },
    domain::entities::faq_entry::FaqEntry,
};

/// Serviço que coordena a listagem e filtragem de entradas de FAQ.
pub struct ListFaqEntriesUseCase<R> {
    repository: R,
}

impl<R> ListFaqEntriesUseCase<R>
where
    R: FaqRepository,
{
    /// Cria o caso de uso com o repositório fornecido.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Executa a listagem de entradas de FAQ de acordo com os filtros informados.
    ///
    /// # Regras de negócio
    ///
    /// 1. O Centro Acadêmico deve existir (`404 academic_center_not_found`).
    /// 2. O Centro Acadêmico deve estar ativo (`422 academic_center_inactive`).
    /// 3. Retorna lista vazia de forma consistente caso não haja entradas que correspondam aos filtros.
    pub async fn execute(
        &self,
        query: ListFaqEntriesQuery,
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

        let filter = FaqFilter::new(query.category, query.search);
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
            "Qual o prazo para trancamento de matrícula?".to_string(),
            "Consulte o calendário acadêmico oficial no SIGAA.".to_string(),
            Some("Matrícula".to_string()),
            AUTHOR_ID,
            now - chrono::Duration::hours(1),
        )
        .unwrap();

        let e3 = FaqEntry::new(
            Uuid::new_v4(),
            CA_MATEMATICA,
            "Qual o horário da biblioteca do CA de Matemática?".to_string(),
            "Das 08h às 20h nos dias úteis.".to_string(),
            Some("Biblioteca".to_string()),
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
    async fn should_list_all_entries_for_active_ca() {
        let repo = setup_repository().await;
        let use_case = ListFaqEntriesUseCase::new(&repo);

        let query = ListFaqEntriesQuery::new(CA_COMPUTACAO, None, None);
        let entries = use_case.execute(query).await.expect("deve listar FAQ");

        assert_eq!(entries.len(), 2);
        // Ordenação por created_at DESC: e2 (mais recente) primeiro
        assert_eq!(
            entries[0].question(),
            "Qual o prazo para trancamento de matrícula?"
        );
        assert_eq!(entries[1].question(), "Como solicitar apoio acadêmico?");
    }

    #[tokio::test]
    async fn should_filter_by_category() {
        let repo = setup_repository().await;
        let use_case = ListFaqEntriesUseCase::new(&repo);

        let query =
            ListFaqEntriesQuery::new(CA_COMPUTACAO, Some("Apoio Acadêmico".to_string()), None);
        let entries = use_case
            .execute(query)
            .await
            .expect("deve filtrar por categoria");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].question(), "Como solicitar apoio acadêmico?");
    }

    #[tokio::test]
    async fn should_filter_by_search_term_in_question() {
        let repo = setup_repository().await;
        let use_case = ListFaqEntriesUseCase::new(&repo);

        let query = ListFaqEntriesQuery::new(CA_COMPUTACAO, None, Some("matrícula".to_string()));
        let entries = use_case
            .execute(query)
            .await
            .expect("deve filtrar por busca");

        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].question(),
            "Qual o prazo para trancamento de matrícula?"
        );
    }

    #[tokio::test]
    async fn should_filter_by_search_term_in_answer() {
        let repo = setup_repository().await;
        let use_case = ListFaqEntriesUseCase::new(&repo);

        let query = ListFaqEntriesQuery::new(CA_COMPUTACAO, None, Some("SIGAA".to_string()));
        let entries = use_case
            .execute(query)
            .await
            .expect("deve filtrar por resposta");

        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].answer(),
            "Consulte o calendário acadêmico oficial no SIGAA."
        );
    }

    #[tokio::test]
    async fn should_return_empty_list_when_no_matches_found() {
        let repo = setup_repository().await;
        let use_case = ListFaqEntriesUseCase::new(&repo);

        let query = ListFaqEntriesQuery::new(
            CA_COMPUTACAO,
            None,
            Some("termo sem correspondência".to_string()),
        );
        let entries = use_case
            .execute(query)
            .await
            .expect("deve retornar lista vazia");

        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn should_enforce_strict_isolation_between_cas() {
        let repo = setup_repository().await;
        let use_case = ListFaqEntriesUseCase::new(&repo);

        // Busca por biblioteca no CA de Computação (que existe apenas no CA de Matemática)
        let query = ListFaqEntriesQuery::new(CA_COMPUTACAO, None, Some("biblioteca".to_string()));
        let entries = use_case
            .execute(query)
            .await
            .expect("deve consultar sem erro");

        assert!(
            entries.is_empty(),
            "não deve exibir perguntas do CA de Matemática no CA de Computação"
        );
    }

    #[tokio::test]
    async fn should_reject_when_ca_not_found() {
        let repo = setup_repository().await;
        let use_case = ListFaqEntriesUseCase::new(&repo);
        let unknown_ca = Uuid::new_v4();

        let query = ListFaqEntriesQuery::new(unknown_ca, None, None);
        let err = use_case.execute(query).await.unwrap_err();

        assert_eq!(err, FaqApplicationError::AcademicCenterNotFound(unknown_ca));
    }

    #[tokio::test]
    async fn should_reject_when_ca_is_inactive() {
        let repo = InMemoryFaqRepository::new();
        let inactive_ca = Uuid::new_v4();
        repo.add_ca(inactive_ca, false, None);

        let use_case = ListFaqEntriesUseCase::new(&repo);
        let query = ListFaqEntriesQuery::new(inactive_ca, None, None);
        let err = use_case.execute(query).await.unwrap_err();

        assert_eq!(
            err,
            FaqApplicationError::AcademicCenterInactive(inactive_ca)
        );
    }
}
