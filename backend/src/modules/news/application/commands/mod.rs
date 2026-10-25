//! Comandos e casos de uso de alteração de estado do módulo de notícias.
//!
//! Implementa os fluxos de transição editorial: submissão para revisão,
//! publicação e arquivamento, aplicando invariantes de autorização e domínio.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::news::{
    application::{
        NewsApplicationError,
        ports::{NewsAuthorizationPort, NewsRepositoryPort},
    },
    domain::{NewsArticle, NewsStatus},
};

/// Comando para publicar um artigo de notícia existente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishNewsArticleCommand {
    /// Identificador único da notícia a ser publicada.
    pub news_id: Uuid,
    /// Identificador do usuário que solicita a publicação.
    pub actor_id: Uuid,
    /// Instante opcional de publicação (se omitido, adota o instante atual).
    pub published_at: Option<DateTime<Utc>>,
}

/// Comando para arquivar um artigo de notícia existente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveNewsArticleCommand {
    /// Identificador único da notícia a ser arquivada.
    pub news_id: Uuid,
    /// Identificador do usuário que solicita o arquivamento.
    pub actor_id: Uuid,
    /// Instante opcional de arquivamento (se omitido, adota o instante atual).
    pub archived_at: Option<DateTime<Utc>>,
}

/// Comando para submeter um rascunho de notícia para homologação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitForReviewNewsArticleCommand {
    /// Identificador único da notícia a ser submetida.
    pub news_id: Uuid,
    /// Identificador do usuário que submete o conteúdo.
    pub actor_id: Uuid,
}

/// Caso de uso responsável por homologar e publicar um artigo de notícia.
///
/// Garante que:
/// 1. A notícia existe.
/// 2. O usuário requisitante possui permissão institucional de gestão no CA.
/// 3. A notícia não se encontra arquivada e transita validamente para `Published`.
/// 4. O timestamp `published_at` é registrado e a atualização ocorre de forma atômica.
pub struct PublishNewsArticleUseCase<R, A>
where
    R: NewsRepositoryPort + ?Sized,
    A: NewsAuthorizationPort + ?Sized,
{
    repository: Arc<R>,
    authorization: Arc<A>,
}

impl<R, A> PublishNewsArticleUseCase<R, A>
where
    R: NewsRepositoryPort + ?Sized,
    A: NewsAuthorizationPort + ?Sized,
{
    /// Constrói uma nova instância do caso de uso com suas dependências.
    pub fn new(repository: Arc<R>, authorization: Arc<A>) -> Self {
        Self {
            repository,
            authorization,
        }
    }

    /// Executa o fluxo de publicação da notícia.
    ///
    /// # Erros
    ///
    /// - [`NewsApplicationError::NotFound`] se a notícia não existir.
    /// - [`NewsApplicationError::Forbidden`] se o usuário não tiver permissão no CA.
    /// - [`NewsApplicationError::AlreadyArchived`] se a notícia já estiver arquivada.
    /// - [`NewsApplicationError::InvalidTransition`] se a transição a partir do estado atual for inválida.
    /// - [`NewsApplicationError::Repository`] em falhas de persistência.
    pub async fn execute(
        &self,
        command: PublishNewsArticleCommand,
    ) -> Result<NewsArticle, NewsApplicationError> {
        let mut article = self
            .repository
            .find_by_id(command.news_id)
            .await?
            .ok_or(NewsApplicationError::NotFound(command.news_id))?;

        if article.status.is_archived() {
            return Err(NewsApplicationError::AlreadyArchived { id: article.id });
        }

        let is_authorized = self
            .authorization
            .can_publish_news(command.actor_id, article.ca_id)
            .await?;

        if !is_authorized {
            return Err(NewsApplicationError::Forbidden {
                user_id: command.actor_id,
                ca_id: article.ca_id,
                operation: "publish",
            });
        }

        let now = command.published_at.unwrap_or_else(Utc::now);
        let expected_status = article.status;

        // Valida as invariantes do domínio
        article.publish(now)?;

        // Aplica a atualização atômica no armazenamento
        let updated_article = self
            .repository
            .update_status_atomic(
                article.id,
                expected_status,
                NewsStatus::Published,
                Some(now),
                None,
            )
            .await?;

        Ok(updated_article)
    }
}

/// Caso de uso responsável por arquivar um artigo de notícia existente.
///
/// Garante que:
/// 1. A notícia existe.
/// 2. A notícia não se encontra já arquivada (arquivamento repetido rejeitado).
/// 3. O usuário solicitante possui permissão institucional de gestão no CA.
/// 4. O timestamp `archived_at` é registrado e a atualização ocorre de forma atômica.
pub struct ArchiveNewsArticleUseCase<R, A>
where
    R: NewsRepositoryPort + ?Sized,
    A: NewsAuthorizationPort + ?Sized,
{
    repository: Arc<R>,
    authorization: Arc<A>,
}

impl<R, A> ArchiveNewsArticleUseCase<R, A>
where
    R: NewsRepositoryPort + ?Sized,
    A: NewsAuthorizationPort + ?Sized,
{
    /// Constrói uma nova instância do caso de uso com suas dependências.
    pub fn new(repository: Arc<R>, authorization: Arc<A>) -> Self {
        Self {
            repository,
            authorization,
        }
    }

    /// Executa o fluxo de arquivamento da notícia.
    ///
    /// # Erros
    ///
    /// - [`NewsApplicationError::NotFound`] se a notícia não existir.
    /// - [`NewsApplicationError::AlreadyArchived`] se a notícia já estiver arquivada.
    /// - [`NewsApplicationError::Forbidden`] se o usuário não tiver permissão no CA.
    /// - [`NewsApplicationError::InvalidTransition`] se a transição for recusada pelo domínio.
    /// - [`NewsApplicationError::Repository`] em falhas de persistência.
    pub async fn execute(
        &self,
        command: ArchiveNewsArticleCommand,
    ) -> Result<NewsArticle, NewsApplicationError> {
        let mut article = self
            .repository
            .find_by_id(command.news_id)
            .await?
            .ok_or(NewsApplicationError::NotFound(command.news_id))?;

        if article.status.is_archived() {
            return Err(NewsApplicationError::AlreadyArchived { id: article.id });
        }

        let is_authorized = self
            .authorization
            .can_archive_news(command.actor_id, article.ca_id)
            .await?;

        if !is_authorized {
            return Err(NewsApplicationError::Forbidden {
                user_id: command.actor_id,
                ca_id: article.ca_id,
                operation: "archive",
            });
        }

        let now = command.archived_at.unwrap_or_else(Utc::now);
        let expected_status = article.status;

        // Valida as invariantes do domínio
        article.archive(now)?;

        // Aplica a atualização atômica no armazenamento
        let updated_article = self
            .repository
            .update_status_atomic(
                article.id,
                expected_status,
                NewsStatus::Archived,
                None,
                Some(now),
            )
            .await?;

        Ok(updated_article)
    }
}

/// Caso de uso responsável por submeter uma proposta de notícia para homologação.
///
/// Garante que:
/// 1. A notícia existe.
/// 2. O usuário requisitante é o autor ou possui permissão de gestão no CA.
/// 3. A notícia transita validamente de `Draft` ou `Rejected` para `PendingReview`.
/// 4. O status é atualizado atomicamente no repositório.
pub struct SubmitForReviewNewsArticleUseCase<R, A>
where
    R: NewsRepositoryPort + ?Sized,
    A: NewsAuthorizationPort + ?Sized,
{
    repository: Arc<R>,
    authorization: Arc<A>,
}

impl<R, A> SubmitForReviewNewsArticleUseCase<R, A>
where
    R: NewsRepositoryPort + ?Sized,
    A: NewsAuthorizationPort + ?Sized,
{
    /// Constrói uma nova instância do caso de uso com suas dependências.
    pub fn new(repository: Arc<R>, authorization: Arc<A>) -> Self {
        Self {
            repository,
            authorization,
        }
    }

    /// Executa o fluxo de submissão da notícia para revisão.
    ///
    /// # Erros
    ///
    /// - [`NewsApplicationError::NotFound`] se a notícia não existir.
    /// - [`NewsApplicationError::AlreadyArchived`] se o artigo estiver arquivado.
    /// - [`NewsApplicationError::Forbidden`] se o usuário não for o autor nem tiver cargo de gestão.
    /// - [`NewsApplicationError::InvalidTransition`] se a transição for rejeitada pelo domínio.
    /// - [`NewsApplicationError::Repository`] em falhas de persistência.
    pub async fn execute(
        &self,
        command: SubmitForReviewNewsArticleCommand,
    ) -> Result<NewsArticle, NewsApplicationError> {
        let mut article = self
            .repository
            .find_by_id(command.news_id)
            .await?
            .ok_or(NewsApplicationError::NotFound(command.news_id))?;

        if article.status.is_archived() {
            return Err(NewsApplicationError::AlreadyArchived { id: article.id });
        }

        let is_authorized = self
            .authorization
            .can_submit_news(command.actor_id, article.ca_id, article.author_id)
            .await?;

        if !is_authorized {
            return Err(NewsApplicationError::Forbidden {
                user_id: command.actor_id,
                ca_id: article.ca_id,
                operation: "submit_for_review",
            });
        }

        let expected_status = article.status;

        // Valida as invariantes do domínio
        article.submit_for_review()?;

        // Aplica a atualização atômica no armazenamento
        let updated_article = self
            .repository
            .update_status_atomic(
                article.id,
                expected_status,
                NewsStatus::PendingReview,
                None,
                None,
            )
            .await?;

        Ok(updated_article)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::news::infrastructure::{
        InMemoryNewsRepository, StaticRoleNewsAuthorization,
    };

    fn criar_cenario_padrao() -> (
        Arc<InMemoryNewsRepository>,
        Arc<StaticRoleNewsAuthorization>,
        Uuid,
        Uuid,
        Uuid,
        NewsArticle,
    ) {
        let ca_id = Uuid::parse_str("018f0000-0000-7000-8000-000000000001").unwrap();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();

        let article = NewsArticle::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            "Reforma da Sala de Estudos".to_string(),
            "A diretoria concluiu as reformas na sala de convivência.".to_string(),
            Utc::now(),
        )
        .unwrap();

        let mut auth = StaticRoleNewsAuthorization::new();
        auth.grant_manage(manager_id, ca_id);

        let repo = Arc::new(InMemoryNewsRepository::with_articles([article.clone()]));
        let auth_arc = Arc::new(auth);

        (repo, auth_arc, ca_id, author_id, manager_id, article)
    }

    #[tokio::test]
    async fn deve_publicar_noticia_com_gestor_autorizado() {
        let (repo, auth, _ca_id, _author_id, manager_id, article) = criar_cenario_padrao();
        let use_case = PublishNewsArticleUseCase::new(repo.clone(), auth);

        let resultado = use_case
            .execute(PublishNewsArticleCommand {
                news_id: article.id,
                actor_id: manager_id,
                published_at: None,
            })
            .await;

        assert!(resultado.is_ok());
        let publicado = resultado.unwrap();
        assert_eq!(publicado.status, NewsStatus::Published);
        assert!(publicado.published_at.is_some());

        // Confirma persistência
        let no_repo = repo.find_by_id(article.id).await.unwrap().unwrap();
        assert_eq!(no_repo.status, NewsStatus::Published);
    }

    #[tokio::test]
    async fn deve_recusar_publicacao_sem_permissao_de_gestao() {
        let (repo, auth, ca_id, _author_id, _manager_id, article) = criar_cenario_padrao();
        let use_case = PublishNewsArticleUseCase::new(repo, auth);
        let usuario_comum = Uuid::new_v4();

        let resultado = use_case
            .execute(PublishNewsArticleCommand {
                news_id: article.id,
                actor_id: usuario_comum,
                published_at: None,
            })
            .await;

        match resultado.unwrap_err() {
            NewsApplicationError::Forbidden {
                user_id,
                ca_id: err_ca,
                operation,
            } => {
                assert_eq!(user_id, usuario_comum);
                assert_eq!(err_ca, ca_id);
                assert_eq!(operation, "publish");
            }
            outro => panic!("Esperava Forbidden, mas recebeu: {outro:?}"),
        }
    }

    #[tokio::test]
    async fn deve_recusar_publicacao_de_noticia_inexistente() {
        let (repo, auth, _ca_id, _author_id, manager_id, _article) = criar_cenario_padrao();
        let use_case = PublishNewsArticleUseCase::new(repo, auth);
        let id_inexistente = Uuid::new_v4();

        let resultado = use_case
            .execute(PublishNewsArticleCommand {
                news_id: id_inexistente,
                actor_id: manager_id,
                published_at: None,
            })
            .await;

        assert!(matches!(
            resultado.unwrap_err(),
            NewsApplicationError::NotFound(id) if id == id_inexistente
        ));
    }

    #[tokio::test]
    async fn deve_recusar_publicacao_de_noticia_ja_arquivada() {
        let (repo, auth, _ca_id, _author_id, manager_id, mut article) = criar_cenario_padrao();
        article.archive(Utc::now()).unwrap();
        repo.update(&article).await.unwrap();

        let use_case = PublishNewsArticleUseCase::new(repo, auth);

        let resultado = use_case
            .execute(PublishNewsArticleCommand {
                news_id: article.id,
                actor_id: manager_id,
                published_at: None,
            })
            .await;

        assert!(matches!(
            resultado.unwrap_err(),
            NewsApplicationError::AlreadyArchived { id } if id == article.id
        ));
    }

    #[tokio::test]
    async fn deve_arquivar_noticia_com_gestor_autorizado() {
        let (repo, auth, _ca_id, _author_id, manager_id, mut article) = criar_cenario_padrao();
        article.publish(Utc::now()).unwrap();
        repo.update(&article).await.unwrap();

        let use_case = ArchiveNewsArticleUseCase::new(repo.clone(), auth);

        let resultado = use_case
            .execute(ArchiveNewsArticleCommand {
                news_id: article.id,
                actor_id: manager_id,
                archived_at: None,
            })
            .await;

        assert!(resultado.is_ok());
        let arquivado = resultado.unwrap();
        assert_eq!(arquivado.status, NewsStatus::Archived);
        assert!(arquivado.archived_at.is_some());

        let no_repo = repo.find_by_id(article.id).await.unwrap().unwrap();
        assert_eq!(no_repo.status, NewsStatus::Archived);
    }

    #[tokio::test]
    async fn deve_recusar_arquivar_noticia_ja_arquivada() {
        let (repo, auth, _ca_id, _author_id, manager_id, mut article) = criar_cenario_padrao();
        article.archive(Utc::now()).unwrap();
        repo.update(&article).await.unwrap();

        let use_case = ArchiveNewsArticleUseCase::new(repo, auth);

        let resultado = use_case
            .execute(ArchiveNewsArticleCommand {
                news_id: article.id,
                actor_id: manager_id,
                archived_at: None,
            })
            .await;

        assert!(matches!(
            resultado.unwrap_err(),
            NewsApplicationError::AlreadyArchived { id } if id == article.id
        ));
    }

    #[tokio::test]
    async fn deve_recusar_arquivamento_sem_permissao() {
        let (repo, auth, ca_id, _author_id, _manager_id, article) = criar_cenario_padrao();
        let use_case = ArchiveNewsArticleUseCase::new(repo, auth);
        let sem_cargo = Uuid::new_v4();

        let resultado = use_case
            .execute(ArchiveNewsArticleCommand {
                news_id: article.id,
                actor_id: sem_cargo,
                archived_at: None,
            })
            .await;

        assert!(matches!(
            resultado.unwrap_err(),
            NewsApplicationError::Forbidden { user_id, ca_id: err_ca, operation }
                if user_id == sem_cargo && err_ca == ca_id && operation == "archive"
        ));
    }

    #[tokio::test]
    async fn deve_submeter_para_revisao_pelo_proprio_autor() {
        let (repo, auth, _ca_id, author_id, _manager_id, article) = criar_cenario_padrao();
        let use_case = SubmitForReviewNewsArticleUseCase::new(repo.clone(), auth);

        let resultado = use_case
            .execute(SubmitForReviewNewsArticleCommand {
                news_id: article.id,
                actor_id: author_id,
            })
            .await;

        assert!(resultado.is_ok());
        let pendente = resultado.unwrap();
        assert_eq!(pendente.status, NewsStatus::PendingReview);
    }

    #[tokio::test]
    async fn deve_recusar_submissao_por_terceiro_sem_cargo() {
        let (repo, auth, _ca_id, _author_id, _manager_id, article) = criar_cenario_padrao();
        let use_case = SubmitForReviewNewsArticleUseCase::new(repo, auth);
        let terceiro = Uuid::new_v4();

        let resultado = use_case
            .execute(SubmitForReviewNewsArticleCommand {
                news_id: article.id,
                actor_id: terceiro,
            })
            .await;

        assert!(matches!(
            resultado.unwrap_err(),
            NewsApplicationError::Forbidden { user_id, operation, .. }
                if user_id == terceiro && operation == "submit_for_review"
        ));
    }
}
