//! Caso de uso: listagem de categorias disponíveis no FAQ de um Centro Acadêmico.
//!
//! Acesso liberado para estudantes e gestores autenticados.
//! Valida a existência do Centro Acadêmico e sua situação ativa.

use uuid::Uuid;

use crate::modules::faq::application::{
    error::FaqApplicationError, ports::FaqRepository,
    queries::list_faq_categories::ListFaqCategoriesQuery,
};

/// Serviço que coordena a listagem de categorias temáticas disponíveis no FAQ.
pub struct ListFaqCategoriesUseCase<R> {
    repository: R,
}

impl<R> ListFaqCategoriesUseCase<R>
where
    R: FaqRepository,
{
    /// Cria o caso de uso com o repositório fornecido.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Executa a listagem das categorias temáticas distintas cadastradas no CA.
    ///
    /// # Regras de negócio
    ///
    /// 1. O Centro Acadêmico deve existir (`404 academic_center_not_found`).
    /// 2. O Centro Acadêmico deve estar ativo (`422 academic_center_inactive`).
    /// 3. Retorna lista vazia caso não haja categorias cadastradas.
    pub async fn execute(
        &self,
        query: ListFaqCategoriesQuery,
    ) -> Result<Vec<String>, FaqApplicationError> {
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

        let categories = self
            .repository
            .list_categories_by_academic_center_id(query.academic_center_id)
            .await?;

        Ok(categories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::faq::{
        domain::entities::faq_entry::FaqEntry, infrastructure::repository::InMemoryFaqRepository,
    };
    use chrono::Utc;

    const CA_COMPUTACAO: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000010");
    const CA_MATEMATICA: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000020");
    const AUTHOR_ID: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000100");

    #[tokio::test]
    async fn should_list_distinct_sorted_categories() {
        let repo = InMemoryFaqRepository::new();
        repo.add_ca(CA_COMPUTACAO, true, Some(AUTHOR_ID));

        let now = Utc::now();
        let e1 = FaqEntry::new(
            Uuid::new_v4(),
            CA_COMPUTACAO,
            "Pergunta 1?".to_string(),
            "Resposta 1.".to_string(),
            Some("Matrícula".to_string()),
            AUTHOR_ID,
            now,
        )
        .unwrap();

        let e2 = FaqEntry::new(
            Uuid::new_v4(),
            CA_COMPUTACAO,
            "Pergunta 2?".to_string(),
            "Resposta 2.".to_string(),
            Some("Apoio Acadêmico".to_string()),
            AUTHOR_ID,
            now,
        )
        .unwrap();

        let e3 = FaqEntry::new(
            Uuid::new_v4(),
            CA_COMPUTACAO,
            "Pergunta 3?".to_string(),
            "Resposta 3.".to_string(),
            Some("Matrícula".to_string()), // Duplicada
            AUTHOR_ID,
            now,
        )
        .unwrap();

        let e4 = FaqEntry::new(
            Uuid::new_v4(),
            CA_COMPUTACAO,
            "Pergunta sem categoria?".to_string(),
            "Resposta 4.".to_string(),
            None,
            AUTHOR_ID,
            now,
        )
        .unwrap();

        repo.save(&e1).await.unwrap();
        repo.save(&e2).await.unwrap();
        repo.save(&e3).await.unwrap();
        repo.save(&e4).await.unwrap();

        let use_case = ListFaqCategoriesUseCase::new(&repo);
        let categories = use_case
            .execute(ListFaqCategoriesQuery::new(CA_COMPUTACAO))
            .await
            .expect("deve listar categorias");

        assert_eq!(categories, vec!["Apoio Acadêmico", "Matrícula"]);
    }

    #[tokio::test]
    async fn should_isolate_categories_between_cas() {
        let repo = InMemoryFaqRepository::new();
        repo.add_ca(CA_COMPUTACAO, true, Some(AUTHOR_ID));
        repo.add_ca(CA_MATEMATICA, true, None);

        let now = Utc::now();
        let e1 = FaqEntry::new(
            Uuid::new_v4(),
            CA_MATEMATICA,
            "Pergunta do CA de Matemática?".to_string(),
            "Resposta.".to_string(),
            Some("Geometria".to_string()),
            AUTHOR_ID,
            now,
        )
        .unwrap();
        repo.save(&e1).await.unwrap();

        let use_case = ListFaqCategoriesUseCase::new(&repo);
        let categories = use_case
            .execute(ListFaqCategoriesQuery::new(CA_COMPUTACAO))
            .await
            .expect("deve listar");

        assert!(
            categories.is_empty(),
            "não deve exibir categorias do outro CA"
        );
    }

    #[tokio::test]
    async fn should_reject_when_ca_not_found() {
        let repo = InMemoryFaqRepository::new();
        let unknown = Uuid::new_v4();

        let use_case = ListFaqCategoriesUseCase::new(&repo);
        let err = use_case
            .execute(ListFaqCategoriesQuery::new(unknown))
            .await
            .unwrap_err();

        assert_eq!(err, FaqApplicationError::AcademicCenterNotFound(unknown));
    }

    #[tokio::test]
    async fn should_reject_when_ca_inactive() {
        let repo = InMemoryFaqRepository::new();
        let inactive = Uuid::new_v4();
        repo.add_ca(inactive, false, None);

        let use_case = ListFaqCategoriesUseCase::new(&repo);
        let err = use_case
            .execute(ListFaqCategoriesQuery::new(inactive))
            .await
            .unwrap_err();

        assert_eq!(err, FaqApplicationError::AcademicCenterInactive(inactive));
    }
}
