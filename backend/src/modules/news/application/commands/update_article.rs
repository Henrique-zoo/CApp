//! Caso de uso: atualização de artigo de notícia.

use chrono::Utc;
use uuid::Uuid;

use crate::modules::news::application::ports::{NewsAuthorizationPort, NewsRepository};
use crate::modules::news::domain::{NewsArticle, NewsError, NewsStatus};

/// Comando para atualizar um artigo de notícia existente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateNewsArticleCommand {
    /// Identificador da notícia a ser atualizada.
    pub article_id: Uuid,
    /// Identificador do usuário que está solicitando a edição.
    pub author_id: Uuid,
    /// Novo título para a notícia.
    pub title: String,
    /// Novo conteúdo textual para a notícia.
    pub body: String,
    /// Se verdadeiro, solicita manter ou colocar em publicação direta após a edição.
    pub publish_directly: bool,
}

/// Manipulador do caso de uso de atualização de artigo de notícia.
pub struct UpdateNewsArticleHandler<R, A> {
    repository: R,
    authorization_port: A,
}

impl<R, A> UpdateNewsArticleHandler<R, A>
where
    R: NewsRepository,
    A: NewsAuthorizationPort,
{
    /// Constrói o manipulador com as dependências fornecidas.
    pub fn new(repository: R, authorization_port: A) -> Self {
        Self {
            repository,
            authorization_port,
        }
    }

    /// Executa o comando de atualização da notícia.
    ///
    /// Localiza a notícia existente, verifica se o usuário tem permissão de gestão
    /// no Centro Acadêmico correspondente, atualiza o conteúdo (incrementando
    /// a revisão caso o texto tenha mudado) e aplica a política de homologação:
    /// se a notícia estava publicada e não houve solicitação/autorização de
    /// publicação direta, o estado reverte para rascunho.
    pub async fn execute(&self, cmd: UpdateNewsArticleCommand) -> Result<NewsArticle, NewsError> {
        let mut article = self
            .repository
            .find_by_id(cmd.article_id)
            .await?
            .ok_or(NewsError::ArticleNotFound { id: cmd.article_id })?;

        let can_manage = self
            .authorization_port
            .can_manage_news(cmd.author_id, article.ca_id)
            .await?;

        if !can_manage {
            return Err(NewsError::UnauthorizedAuthor {
                author_id: cmd.author_id,
                ca_id: article.ca_id,
            });
        }

        if cmd.publish_directly {
            let can_publish = self
                .authorization_port
                .can_publish_directly(cmd.author_id, article.ca_id)
                .await?;

            if !can_publish {
                return Err(NewsError::UnauthorizedPublish {
                    author_id: cmd.author_id,
                    ca_id: article.ca_id,
                });
            }
        }

        let was_published = article.status == NewsStatus::Published;
        let changed = article.update_content(cmd.title, cmd.body)?;

        if changed {
            if was_published && !cmd.publish_directly {
                article.mark_as_draft()?;
            } else if cmd.publish_directly {
                article.publish(Utc::now())?;
            }
        }

        self.repository.update(&article).await?;
        Ok(article)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::news::infrastructure::{
        InMemoryNewsAuthorizationService, InMemoryNewsRepository,
    };

    const CAER_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000001");
    const CACC_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000002");
    const VETERANO_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000010");
    const CALOURO_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000020");

    #[tokio::test]
    async fn deve_atualizar_noticia_com_sucesso_e_incrementar_revisao() {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);

        let initial_article = NewsArticle::new(
            CAER_ID,
            VETERANO_ID,
            "Título Original",
            "Texto original da notícia.",
        )
        .expect("artigo válido");
        let article_id = initial_article.id;
        repo.save(&initial_article).await.expect("salva artigo");

        let handler = UpdateNewsArticleHandler::new(repo.clone(), auth);

        let cmd = UpdateNewsArticleCommand {
            article_id,
            author_id: VETERANO_ID,
            title: "Título Alterado".to_string(),
            body: "Texto alterado com novas informações.".to_string(),
            publish_directly: false,
        };

        let updated = handler.execute(cmd).await.expect("atualiza com sucesso");
        assert_eq!(updated.title, "Título Alterado");
        assert_eq!(updated.body, "Texto alterado com novas informações.");
        assert_eq!(updated.revision, 2);

        let from_repo = repo
            .find_by_id(article_id)
            .await
            .unwrap()
            .expect("artigo no repo");
        assert_eq!(from_repo.revision, 2);
    }

    #[tokio::test]
    async fn deve_reverter_noticia_publicada_para_rascunho_quando_editada_sem_publicacao_direta() {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);

        let mut article = NewsArticle::new(
            CAER_ID,
            VETERANO_ID,
            "Título Publicado",
            "Conteúdo inicialmente publicado.",
        )
        .expect("artigo válido");
        article.publish(Utc::now()).expect("publica");
        let article_id = article.id;
        repo.save(&article).await.expect("salva");

        let handler = UpdateNewsArticleHandler::new(repo.clone(), auth);

        let cmd = UpdateNewsArticleCommand {
            article_id,
            author_id: VETERANO_ID,
            title: "Título Modificado".to_string(),
            body: "Conteúdo com errata.".to_string(),
            publish_directly: false,
        };

        let updated = handler
            .execute(cmd)
            .await
            .expect("deve atualizar com sucesso");
        assert_eq!(updated.status, NewsStatus::Draft);
        assert_eq!(updated.revision, 2);
    }

    #[tokio::test]
    async fn deve_manter_noticia_publicada_quando_autor_tem_permissao_e_solicita_publicacao_direta()
    {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);
        auth.grant_publish_permission(VETERANO_ID, CAER_ID);

        let mut article =
            NewsArticle::new(CAER_ID, VETERANO_ID, "Comunicado", "Texto do comunicado.")
                .expect("artigo válido");
        article.publish(Utc::now()).expect("publica");
        let article_id = article.id;
        repo.save(&article).await.expect("salva");

        let handler = UpdateNewsArticleHandler::new(repo.clone(), auth);

        let cmd = UpdateNewsArticleCommand {
            article_id,
            author_id: VETERANO_ID,
            title: "Comunicado Retificado".to_string(),
            body: "Texto retificado com urgência.".to_string(),
            publish_directly: true,
        };

        let updated = handler
            .execute(cmd)
            .await
            .expect("deve atualizar e republicar");
        assert_eq!(updated.status, NewsStatus::Published);
        assert_eq!(updated.revision, 2);
    }

    #[tokio::test]
    async fn deve_recusar_edicao_de_noticia_inexistente() {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        let handler = UpdateNewsArticleHandler::new(repo, auth);

        let inexistent_id = Uuid::new_v4();
        let cmd = UpdateNewsArticleCommand {
            article_id: inexistent_id,
            author_id: VETERANO_ID,
            title: "Título".to_string(),
            body: "Texto".to_string(),
            publish_directly: false,
        };

        let res = handler.execute(cmd).await;
        assert_eq!(res, Err(NewsError::ArticleNotFound { id: inexistent_id }));
    }

    #[tokio::test]
    async fn deve_recusar_edicao_por_usuario_sem_permissao_no_ca() {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        // Usuário tem permissão no CACC, mas a notícia pertence ao CAER
        auth.grant_manage_permission(CALOURO_ID, CACC_ID);

        let article =
            NewsArticle::new(CAER_ID, VETERANO_ID, "Notícia CAER", "Conteúdo CAER.").unwrap();
        let article_id = article.id;
        repo.save(&article).await.unwrap();

        let handler = UpdateNewsArticleHandler::new(repo, auth);

        let cmd = UpdateNewsArticleCommand {
            article_id,
            author_id: CALOURO_ID,
            title: "Tentativa de Invasão".to_string(),
            body: "Texto indevido.".to_string(),
            publish_directly: false,
        };

        let res = handler.execute(cmd).await;
        assert_eq!(
            res,
            Err(NewsError::UnauthorizedAuthor {
                author_id: CALOURO_ID,
                ca_id: CAER_ID,
            })
        );
    }

    #[tokio::test]
    async fn deve_recusar_edicao_de_noticia_arquivada() {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);

        let mut article =
            NewsArticle::new(CAER_ID, VETERANO_ID, "Notícia Velha", "Conteúdo antigo.").unwrap();
        article.archive(Utc::now()).unwrap();
        let article_id = article.id;
        repo.save(&article).await.unwrap();

        let handler = UpdateNewsArticleHandler::new(repo, auth);

        let cmd = UpdateNewsArticleCommand {
            article_id,
            author_id: VETERANO_ID,
            title: "Novo Título".to_string(),
            body: "Novo Conteúdo.".to_string(),
            publish_directly: false,
        };

        let res = handler.execute(cmd).await;
        assert_eq!(res, Err(NewsError::CannotEditArchived));
    }
}
