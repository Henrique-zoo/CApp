//! Repositório de notícias em memória para testes unitários e desenvolvimento.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use uuid::Uuid;

use crate::modules::news::application::ports::NewsRepository;
use crate::modules::news::domain::{NewsArticle, NewsError};

/// Implementação em memória de [`NewsRepository`].
#[derive(Debug, Clone, Default)]
pub struct InMemoryNewsRepository {
    storage: Arc<RwLock<HashMap<Uuid, NewsArticle>>>,
}

impl InMemoryNewsRepository {
    /// Constrói um novo repositório em memória vazio.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Retorna a quantidade de notícias atualmente armazenadas.
    pub fn count(&self) -> usize {
        self.storage.read().map(|map| map.len()).unwrap_or_default()
    }

    /// Limpa todos os registros do repositório em memória.
    pub fn clear(&self) {
        if let Ok(mut map) = self.storage.write() {
            map.clear();
        }
    }
}

#[async_trait]
impl NewsRepository for InMemoryNewsRepository {
    async fn save(&self, article: &NewsArticle) -> Result<(), NewsError> {
        let mut map = self
            .storage
            .write()
            .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
        map.insert(article.id, article.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<NewsArticle>, NewsError> {
        let map = self
            .storage
            .read()
            .map_err(|e| NewsError::RepositoryError(e.to_string()))?;
        Ok(map.get(&id).cloned())
    }

    async fn update(&self, article: &NewsArticle) -> Result<(), NewsError> {
        let mut map = self
            .storage
            .write()
            .map_err(|e| NewsError::RepositoryError(e.to_string()))?;

        if !map.contains_key(&article.id) {
            return Err(NewsError::ArticleNotFound { id: article.id });
        }

        map.insert(article.id, article.clone());
        Ok(())
    }

    async fn find_by_ca_id(&self, ca_id: Uuid) -> Result<Vec<NewsArticle>, NewsError> {
        let map = self
            .storage
            .read()
            .map_err(|e| NewsError::RepositoryError(e.to_string()))?;

        let mut articles: Vec<NewsArticle> = map
            .values()
            .filter(|article| article.ca_id == ca_id)
            .cloned()
            .collect();

        articles.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        Ok(articles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAER_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000001");
    const CACC_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000002");
    const VETERANO_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000010");

    #[tokio::test]
    async fn deve_salvar_e_recuperar_por_id() {
        let repo = InMemoryNewsRepository::new();
        let article =
            NewsArticle::new(CAER_ID, VETERANO_ID, "Notícia Teste", "Corpo da notícia.").unwrap();

        repo.save(&article).await.expect("salva");
        assert_eq!(repo.count(), 1);

        let retrieved = repo
            .find_by_id(article.id)
            .await
            .expect("busca")
            .expect("encontrada");
        assert_eq!(retrieved, article);
    }

    #[tokio::test]
    async fn deve_retornar_none_quando_nao_encontrado() {
        let repo = InMemoryNewsRepository::new();
        let result = repo.find_by_id(Uuid::new_v4()).await.expect("busca");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn deve_atualizar_noticia_existente() {
        let repo = InMemoryNewsRepository::new();
        let mut article =
            NewsArticle::new(CAER_ID, VETERANO_ID, "Original", "Corpo original.").unwrap();
        repo.save(&article).await.unwrap();

        article
            .update_content("Modificado", "Corpo modificado.")
            .unwrap();
        repo.update(&article).await.expect("atualiza");

        let updated = repo.find_by_id(article.id).await.unwrap().unwrap();
        assert_eq!(updated.title, "Modificado");
        assert_eq!(updated.revision, 2);
    }

    #[tokio::test]
    async fn deve_retornar_erro_ao_atualizar_inexistente() {
        let repo = InMemoryNewsRepository::new();
        let article =
            NewsArticle::new(CAER_ID, VETERANO_ID, "Fantasma", "Corpo inexistente.").unwrap();

        let res = repo.update(&article).await;
        assert_eq!(res, Err(NewsError::ArticleNotFound { id: article.id }));
    }

    #[tokio::test]
    async fn deve_filtrar_e_ordenar_por_ca_id() {
        let repo = InMemoryNewsRepository::new();
        let a1 = NewsArticle::new(CAER_ID, VETERANO_ID, "Notícia CAER 1", "Corpo 1.").unwrap();
        let a2 = NewsArticle::new(CACC_ID, VETERANO_ID, "Notícia CACC 1", "Corpo 2.").unwrap();
        let a3 = NewsArticle::new(CAER_ID, VETERANO_ID, "Notícia CAER 2", "Corpo 3.").unwrap();

        repo.save(&a1).await.unwrap();
        repo.save(&a2).await.unwrap();
        repo.save(&a3).await.unwrap();

        let caer_articles = repo.find_by_ca_id(CAER_ID).await.unwrap();
        assert_eq!(caer_articles.len(), 2);
        assert!(caer_articles.iter().all(|a| a.ca_id == CAER_ID));

        let cacc_articles = repo.find_by_ca_id(CACC_ID).await.unwrap();
        assert_eq!(cacc_articles.len(), 1);
        assert_eq!(cacc_articles[0].id, a2.id);
    }
}
