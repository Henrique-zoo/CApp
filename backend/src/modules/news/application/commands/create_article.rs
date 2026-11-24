//! Caso de uso: criação de artigo de notícia.

use chrono::Utc;
use uuid::Uuid;

use crate::modules::news::application::ports::{NewsAuthorizationPort, NewsRepository};
use crate::modules::news::domain::{NewsArticle, NewsError};

/// Comando para criar um novo artigo de notícia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateNewsArticleCommand {
    /// Identificador do Centro Acadêmico proprietário da notícia.
    pub ca_id: Uuid,
    /// Identificador do usuário autor da notícia.
    pub author_id: Uuid,
    /// Título da notícia.
    pub title: String,
    /// Conteúdo textual da notícia.
    pub body: String,
    /// Se verdadeiro, publica diretamente caso o autor possua a permissão necessária.
    pub publish_directly: bool,
}

/// Manipulador do caso de uso de criação de artigo de notícia.
pub struct CreateNewsArticleHandler<R, A> {
    repository: R,
    authorization_port: A,
}

impl<R, A> CreateNewsArticleHandler<R, A>
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

    /// Executa o comando de criação da notícia.
    ///
    /// Valida se o autor tem permissão de gestão no Centro Acadêmico,
    /// valida as invariantes de título e corpo, aplica publicação direta se
    /// solicitado e autorizado, e persiste a nova notícia.
    pub async fn execute(&self, cmd: CreateNewsArticleCommand) -> Result<NewsArticle, NewsError> {
        let can_manage = self
            .authorization_port
            .can_manage_news(cmd.author_id, cmd.ca_id)
            .await?;

        if !can_manage {
            return Err(NewsError::UnauthorizedAuthor {
                author_id: cmd.author_id,
                ca_id: cmd.ca_id,
            });
        }

        if cmd.publish_directly {
            let can_publish = self
                .authorization_port
                .can_publish_directly(cmd.author_id, cmd.ca_id)
                .await?;

            if !can_publish {
                return Err(NewsError::UnauthorizedPublish {
                    author_id: cmd.author_id,
                    ca_id: cmd.ca_id,
                });
            }
        }

        let mut article = NewsArticle::new(cmd.ca_id, cmd.author_id, cmd.title, cmd.body)?;

        if cmd.publish_directly {
            article.publish(Utc::now())?;
        }

        self.repository.save(&article).await?;
        Ok(article)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::news::domain::NewsStatus;
    use crate::modules::news::infrastructure::{
        InMemoryNewsAuthorizationService, InMemoryNewsRepository,
    };

    const CAER_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000001");
    const VETERANO_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000010");
    const CALOURO_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000020");

    #[tokio::test]
    async fn deve_criar_noticia_em_rascunho_quando_autor_tem_permissao_de_gestao() {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);

        let handler = CreateNewsArticleHandler::new(repo.clone(), auth);

        let cmd = CreateNewsArticleCommand {
            ca_id: CAER_ID,
            author_id: VETERANO_ID,
            title: "Edital de Monitoria".to_string(),
            body: "Inscrições abertas para monitoria no semestre.".to_string(),
            publish_directly: false,
        };

        let article = handler.execute(cmd).await.expect("deve criar com sucesso");
        assert_eq!(article.status, NewsStatus::Draft);
        assert_eq!(article.revision, 1);
        assert_eq!(repo.count(), 1);

        let saved = repo
            .find_by_id(article.id)
            .await
            .expect("busca sem erro")
            .expect("deve encontrar artigo salvo");
        assert_eq!(saved.title, "Edital de Monitoria");
    }

    #[tokio::test]
    async fn deve_criar_e_publicar_diretamente_com_permissao_de_publicacao() {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);
        auth.grant_publish_permission(VETERANO_ID, CAER_ID);

        let handler = CreateNewsArticleHandler::new(repo.clone(), auth);

        let cmd = CreateNewsArticleCommand {
            ca_id: CAER_ID,
            author_id: VETERANO_ID,
            title: "Urgente: Horário de Aulas".to_string(),
            body: "Ajuste na grade horária de Engenharia de Redes.".to_string(),
            publish_directly: true,
        };

        let article = handler
            .execute(cmd)
            .await
            .expect("deve criar e publicar com sucesso");
        assert_eq!(article.status, NewsStatus::Published);
        assert!(article.published_at.is_some());
        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn deve_rejeitar_quando_autor_nao_tem_permissao_de_gestao() {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        // CALOURO_ID não tem permissão concedida

        let handler = CreateNewsArticleHandler::new(repo.clone(), auth);

        let cmd = CreateNewsArticleCommand {
            ca_id: CAER_ID,
            author_id: CALOURO_ID,
            title: "Proposta de Notícia".to_string(),
            body: "Tentativa de publicação sem cargo de gestão.".to_string(),
            publish_directly: false,
        };

        let result = handler.execute(cmd).await;
        assert_eq!(
            result,
            Err(NewsError::UnauthorizedAuthor {
                author_id: CALOURO_ID,
                ca_id: CAER_ID,
            })
        );
        assert_eq!(repo.count(), 0);
    }

    #[tokio::test]
    async fn deve_rejeitar_publicacao_direta_se_autor_nao_tem_permissao_especifica() {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        // Tem permissão de gestão, mas NÃO de publicação direta
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);

        let handler = CreateNewsArticleHandler::new(repo.clone(), auth);

        let cmd = CreateNewsArticleCommand {
            ca_id: CAER_ID,
            author_id: VETERANO_ID,
            title: "Notícia Sem Homologação".to_string(),
            body: "Tentativa de publicação direta sem permissão.".to_string(),
            publish_directly: true,
        };

        let result = handler.execute(cmd).await;
        assert_eq!(
            result,
            Err(NewsError::UnauthorizedPublish {
                author_id: VETERANO_ID,
                ca_id: CAER_ID,
            })
        );
        assert_eq!(repo.count(), 0);
    }

    #[tokio::test]
    async fn deve_rejeitar_quando_dados_do_dominio_forem_invalidos() {
        let repo = InMemoryNewsRepository::new();
        let auth = InMemoryNewsAuthorizationService::new();
        auth.grant_manage_permission(VETERANO_ID, CAER_ID);

        let handler = CreateNewsArticleHandler::new(repo.clone(), auth);

        let cmd = CreateNewsArticleCommand {
            ca_id: CAER_ID,
            author_id: VETERANO_ID,
            title: "   ".to_string(),
            body: "Corpo válido.".to_string(),
            publish_directly: false,
        };

        let result = handler.execute(cmd).await;
        assert_eq!(result, Err(NewsError::EmptyTitle));
        assert_eq!(repo.count(), 0);
    }
}
