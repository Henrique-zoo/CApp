//! Entidade de artigo de notícia.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::status::NewsStatus;
use crate::modules::news::domain::NewsError;
use crate::modules::news::domain::rules::{validate_body, validate_title};

/// Artigo de notícia publicado ou em edição por um Centro Acadêmico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticle {
    /// Identificador único e estável da notícia.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico proprietário da notícia.
    pub ca_id: Uuid,
    /// Identificador do usuário autor da notícia.
    pub author_id: Uuid,
    /// Título apresentado na interface.
    pub title: String,
    /// Texto principal do conteúdo.
    pub body: String,
    /// Estado atual da notícia no ciclo editorial.
    pub status: NewsStatus,
    /// Número da revisão deste registro, distinguindo versões do seu conteúdo.
    pub revision: u32,
    /// Instante em que o registro foi criado.
    pub created_at: DateTime<Utc>,
    /// Instante em que o conteúdo foi publicado, se aplicável.
    pub published_at: Option<DateTime<Utc>>,
    /// Instante em que o registro foi arquivado, se aplicável.
    pub archived_at: Option<DateTime<Utc>>,
}

impl NewsArticle {
    /// Cria uma nova notícia em rascunho (revisão 1) após validar título e corpo.
    pub fn new(
        ca_id: Uuid,
        author_id: Uuid,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Result<Self, NewsError> {
        let title = title.into();
        let body = body.into();

        validate_title(&title)?;
        validate_body(&body)?;

        Ok(Self {
            id: Uuid::new_v4(),
            ca_id,
            author_id,
            title: title.trim().to_string(),
            body: body.trim().to_string(),
            status: NewsStatus::Draft,
            revision: 1,
            created_at: Utc::now(),
            published_at: None,
            archived_at: None,
        })
    }

    /// Reconstrói uma instância de notícia a partir de dados já persistidos.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstruct(
        id: Uuid,
        ca_id: Uuid,
        author_id: Uuid,
        title: String,
        body: String,
        status: NewsStatus,
        revision: u32,
        created_at: DateTime<Utc>,
        published_at: Option<DateTime<Utc>>,
        archived_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            ca_id,
            author_id,
            title,
            body,
            status,
            revision,
            created_at,
            published_at,
            archived_at,
        }
    }

    /// Atualiza o conteúdo textual da notícia.
    ///
    /// Se houver alteração em relação aos valores atuais, incrementa a revisão
    /// e retorna `Ok(true)`. Se o conteúdo for idêntico, retorna `Ok(false)`.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsError::CannotEditArchived`] se a notícia estiver arquivada,
    /// ou erros de validação se o novo título ou corpo forem inválidos.
    pub fn update_content(
        &mut self,
        new_title: impl Into<String>,
        new_body: impl Into<String>,
    ) -> Result<bool, NewsError> {
        if self.status == NewsStatus::Archived {
            return Err(NewsError::CannotEditArchived);
        }

        let new_title = new_title.into();
        let new_body = new_body.into();

        validate_title(&new_title)?;
        validate_body(&new_body)?;

        let trimmed_title = new_title.trim();
        let trimmed_body = new_body.trim();

        if self.title == trimmed_title && self.body == trimmed_body {
            return Ok(false);
        }

        self.title = trimmed_title.to_string();
        self.body = trimmed_body.to_string();
        self.revision = self.revision.saturating_add(1);

        Ok(true)
    }

    /// Submete a notícia para homologação.
    pub fn submit_for_review(&mut self) -> Result<(), NewsError> {
        if self.status == NewsStatus::Archived {
            return Err(NewsError::CannotEditArchived);
        }
        self.status = NewsStatus::PendingReview;
        Ok(())
    }

    /// Marca a notícia como publicada, registrando o instante de publicação.
    pub fn publish(&mut self, published_at: DateTime<Utc>) -> Result<(), NewsError> {
        if self.status == NewsStatus::Archived {
            return Err(NewsError::CannotEditArchived);
        }
        self.status = NewsStatus::Published;
        self.published_at = Some(published_at);
        Ok(())
    }

    /// Registra a rejeição da notícia na homologação.
    pub fn reject(&mut self) -> Result<(), NewsError> {
        if self.status == NewsStatus::Archived {
            return Err(NewsError::CannotEditArchived);
        }
        self.status = NewsStatus::Rejected;
        Ok(())
    }

    /// Retorna a notícia ao estado de rascunho.
    pub fn mark_as_draft(&mut self) -> Result<(), NewsError> {
        if self.status == NewsStatus::Archived {
            return Err(NewsError::CannotEditArchived);
        }
        self.status = NewsStatus::Draft;
        Ok(())
    }

    /// Arquiva a notícia, preservando o registro histórico.
    pub fn archive(&mut self, archived_at: DateTime<Utc>) -> Result<(), NewsError> {
        self.status = NewsStatus::Archived;
        self.archived_at = Some(archived_at);
        Ok(())
    }

    /// Informa se a notícia está arquivada.
    pub fn is_archived(&self) -> bool {
        self.status == NewsStatus::Archived
    }

    /// Informa se a notícia está publicada.
    pub fn is_published(&self) -> bool {
        self.status == NewsStatus::Published
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // UUIDs didáticos homologados conforme capp_context.md
    const CAER_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000001");
    const VETERANO_ID: Uuid = uuid::uuid!("018f0000-0000-7000-8000-000000000010");

    #[test]
    fn deve_criar_noticia_com_sucesso_em_rascunho() {
        let article = NewsArticle::new(
            CAER_ID,
            VETERANO_ID,
            "Semana de Integração de Redes",
            "Atividades de recepção e apresentação do curso para calouros.",
        )
        .expect("deve criar artigo válido");

        assert_eq!(article.ca_id, CAER_ID);
        assert_eq!(article.author_id, VETERANO_ID);
        assert_eq!(article.title, "Semana de Integração de Redes");
        assert_eq!(
            article.body,
            "Atividades de recepção e apresentação do curso para calouros."
        );
        assert_eq!(article.status, NewsStatus::Draft);
        assert_eq!(article.revision, 1);
        assert!(article.published_at.is_none());
        assert!(article.archived_at.is_none());
        assert!(!article.is_archived());
        assert!(!article.is_published());
    }

    #[test]
    fn deve_recusar_titulo_vazio_ou_apenas_espacos() {
        let res = NewsArticle::new(CAER_ID, VETERANO_ID, "   ", "Corpo da notícia válido.");
        assert_eq!(res, Err(NewsError::EmptyTitle));
    }

    #[test]
    fn deve_recusar_titulo_muito_curto() {
        let res = NewsArticle::new(CAER_ID, VETERANO_ID, "Oi", "Corpo da notícia válido.");
        assert_eq!(res, Err(NewsError::TitleTooShort { min: 3, actual: 2 }));
    }

    #[test]
    fn deve_recusar_titulo_muito_longo() {
        let long_title = "a".repeat(201);
        let res = NewsArticle::new(CAER_ID, VETERANO_ID, long_title, "Corpo válido.");
        assert_eq!(
            res,
            Err(NewsError::TitleTooLong {
                max: 200,
                actual: 201
            })
        );
    }

    #[test]
    fn deve_recusar_corpo_vazio() {
        let res = NewsArticle::new(CAER_ID, VETERANO_ID, "Título válido", "   ");
        assert_eq!(res, Err(NewsError::EmptyBody));
    }

    #[test]
    fn deve_atualizar_conteudo_e_incrementar_revisao() {
        let mut article = NewsArticle::new(
            CAER_ID,
            VETERANO_ID,
            "Título Inicial",
            "Conteúdo inicial da notícia.",
        )
        .expect("deve criar artigo");

        assert_eq!(article.revision, 1);

        let alterou = article
            .update_content("Título Atualizado", "Novo conteúdo detalhado.")
            .expect("deve atualizar com sucesso");

        assert!(alterou);
        assert_eq!(article.title, "Título Atualizado");
        assert_eq!(article.body, "Novo conteúdo detalhado.");
        assert_eq!(article.revision, 2);

        let alterou_novamente = article
            .update_content("Título Atualizado 2", "Novo conteúdo detalhado.")
            .expect("deve atualizar com sucesso");

        assert!(alterou_novamente);
        assert_eq!(article.revision, 3);
    }

    #[test]
    fn deve_ignorar_atualizacao_se_conteudo_for_identico() {
        let mut article = NewsArticle::new(
            CAER_ID,
            VETERANO_ID,
            "Título Inalterado",
            "Conteúdo inalterado.",
        )
        .expect("deve criar artigo");

        let alterou = article
            .update_content("  Título Inalterado  ", "  Conteúdo inalterado.  ")
            .expect("deve processar");

        assert!(!alterou);
        assert_eq!(article.revision, 1);
    }

    #[test]
    fn deve_impedir_edicao_de_noticia_arquivada() {
        let mut article = NewsArticle::new(
            CAER_ID,
            VETERANO_ID,
            "Notícia a Arquivar",
            "Conteúdo a arquivar.",
        )
        .expect("deve criar artigo");

        article.archive(Utc::now()).expect("deve arquivar");
        assert!(article.is_archived());

        let res = article.update_content("Novo Título", "Novo Conteúdo");
        assert_eq!(res, Err(NewsError::CannotEditArchived));
    }

    #[test]
    fn deve_gerenciar_transicoes_de_status_corretamente() {
        let mut article =
            NewsArticle::new(CAER_ID, VETERANO_ID, "Ciclo Editorial", "Texto do ciclo.")
                .expect("deve criar");

        assert_eq!(article.status, NewsStatus::Draft);

        article
            .submit_for_review()
            .expect("deve submeter para homologação");
        assert_eq!(article.status, NewsStatus::PendingReview);

        article.reject().expect("deve rejeitar");
        assert_eq!(article.status, NewsStatus::Rejected);

        article.mark_as_draft().expect("deve voltar para rascunho");
        assert_eq!(article.status, NewsStatus::Draft);

        let now = Utc::now();
        article.publish(now).expect("deve publicar");
        assert_eq!(article.status, NewsStatus::Published);
        assert_eq!(article.published_at, Some(now));
        assert!(article.is_published());
    }
}
