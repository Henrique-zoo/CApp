//! Entidade de domínio para artigos de notícias no mural do Centro Acadêmico.
//!
//! Encapsula identidade, conteúdo, controle de revisões e ciclo de vida
//! editorial conforme definido no Modelo Entidade-Relacionamento do CApp.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::NewsStatus;
use crate::modules::news::domain::rules::NewsDomainError;

/// Artigo de notícia publicado ou gerenciado por um Centro Acadêmico.
///
/// Invariantes:
/// - O título e o corpo não podem ser vazios.
/// - Notícias em estado [`NewsStatus::Archived`] são imutáveis e não aceitam edições ou novas transições.
/// - A transição para [`NewsStatus::Published`] registra o instante em `published_at`.
/// - A transição para [`NewsStatus::Archived`] registra o instante em `archived_at`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticle {
    /// Identificador estável do artigo.
    pub id: Uuid,
    /// Identificador do Centro Acadêmico responsável pelo artigo.
    pub ca_id: Uuid,
    /// Identificador do usuário autor do conteúdo original.
    pub author_id: Uuid,
    /// Título do artigo divulgado no mural.
    pub title: String,
    /// Conteúdo textual do artigo.
    pub body: String,
    /// Estado atual do artigo no ciclo editorial.
    pub status: NewsStatus,
    /// Versão sequencial de revisão do conteúdo.
    pub revision: i32,
    /// Instante em que o registro inicial foi criado.
    pub created_at: DateTime<Utc>,
    /// Instante em que o artigo foi efetivamente publicado, se aplicável.
    pub published_at: Option<DateTime<Utc>>,
    /// Instante em que o artigo foi arquivado, se aplicável.
    pub archived_at: Option<DateTime<Utc>>,
}

impl NewsArticle {
    /// Cria uma nova notícia em estado inicial de rascunho ([`NewsStatus::Draft`]).
    ///
    /// # Parâmetros
    ///
    /// - `id`: Identificador único do artigo.
    /// - `ca_id`: Identificador do Centro Acadêmico publicador.
    /// - `author_id`: Identificador do estudante/membro autor.
    /// - `title`: Título do artigo.
    /// - `body`: Conteúdo textual principal.
    /// - `created_at`: Instante de criação do registro.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsDomainError::EmptyTitle`] se o título for composto apenas por espaços,
    /// ou [`NewsDomainError::EmptyBody`] se o corpo for vazio.
    pub fn new(
        id: Uuid,
        ca_id: Uuid,
        author_id: Uuid,
        title: String,
        body: String,
        created_at: DateTime<Utc>,
    ) -> Result<Self, NewsDomainError> {
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Err(NewsDomainError::EmptyTitle);
        }

        let trimmed_body = body.trim();
        if trimmed_body.is_empty() {
            return Err(NewsDomainError::EmptyBody);
        }

        Ok(Self {
            id,
            ca_id,
            author_id,
            title: trimmed_title.to_string(),
            body: trimmed_body.to_string(),
            status: NewsStatus::Draft,
            revision: 1,
            created_at,
            published_at: None,
            archived_at: None,
        })
    }

    /// Reconstrói uma entidade a partir de todos os seus dados persistidos.
    ///
    /// Utilizado pela camada de infraestrutura ao carregar registros do banco de dados.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn reconstruct(
        id: Uuid,
        ca_id: Uuid,
        author_id: Uuid,
        title: String,
        body: String,
        status: NewsStatus,
        revision: i32,
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

    /// Submete a notícia para homologação e revisão da gestão do Centro Acadêmico.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsDomainError::ArticleAlreadyArchived`] se a notícia estiver arquivada,
    /// ou [`NewsDomainError::InvalidStatusTransition`] se o status atual não puder transitar para `PendingReview`.
    pub fn submit_for_review(&mut self) -> Result<(), NewsDomainError> {
        if self.status.is_archived() {
            return Err(NewsDomainError::ArticleAlreadyArchived { id: self.id });
        }

        self.status.validate_transition(NewsStatus::PendingReview)?;
        self.status = NewsStatus::PendingReview;
        Ok(())
    }

    /// Aprova e publica a notícia, tornando-a visível no mural e canais correspondentes.
    ///
    /// Define `published_at` com o instante fornecido.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsDomainError::ArticleAlreadyArchived`] se a notícia estiver arquivada,
    /// ou [`NewsDomainError::InvalidStatusTransition`] se o status atual não autorizar publicação.
    pub fn publish(&mut self, published_at: DateTime<Utc>) -> Result<(), NewsDomainError> {
        if self.status.is_archived() {
            return Err(NewsDomainError::ArticleAlreadyArchived { id: self.id });
        }

        self.status.validate_transition(NewsStatus::Published)?;
        self.status = NewsStatus::Published;
        self.published_at = Some(published_at);
        Ok(())
    }

    /// Arquiva a notícia, retirando-a da listagem corrente e preservando o registro para histórico.
    ///
    /// Define `archived_at` com o instante fornecido.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsDomainError::ArticleAlreadyArchived`] se a notícia já se encontrar arquivada,
    /// ou [`NewsDomainError::InvalidStatusTransition`] se o estado atual não permitir arquivamento.
    pub fn archive(&mut self, archived_at: DateTime<Utc>) -> Result<(), NewsDomainError> {
        if self.status.is_archived() {
            return Err(NewsDomainError::ArticleAlreadyArchived { id: self.id });
        }

        self.status.validate_transition(NewsStatus::Archived)?;
        self.status = NewsStatus::Archived;
        self.archived_at = Some(archived_at);
        Ok(())
    }

    /// Rejeita a proposta de notícia no fluxo de homologação editorial.
    ///
    /// # Erros
    ///
    /// Retorna [`NewsDomainError::ArticleAlreadyArchived`] se já arquivada, ou erro de transição inválida.
    pub fn reject(&mut self) -> Result<(), NewsDomainError> {
        if self.status.is_archived() {
            return Err(NewsDomainError::ArticleAlreadyArchived { id: self.id });
        }

        self.status.validate_transition(NewsStatus::Rejected)?;
        self.status = NewsStatus::Rejected;
        Ok(())
    }

    /// Verifica se a notícia pode receber alterações de conteúdo no momento.
    ///
    /// Notícias arquivadas não podem sofrer qualquer tipo de alteração.
    #[must_use]
    pub fn can_edit(&self) -> bool {
        !self.status.is_archived()
    }

    /// Atualiza o título e o corpo da notícia, incrementando a versão da revisão.
    ///
    /// Se a notícia estava em estado [`NewsStatus::Rejected`], ao ser editada retorna
    /// automaticamente para [`NewsStatus::Draft`] para preparação de nova submissão.
    ///
    /// # Erros
    ///
    /// - [`NewsDomainError::ArchivedArticleCannotBeModified`] se o artigo já estiver arquivado.
    /// - [`NewsDomainError::EmptyTitle`] se o novo título for em branco.
    /// - [`NewsDomainError::EmptyBody`] se o novo corpo for em branco.
    pub fn update_content(
        &mut self,
        new_title: String,
        new_body: String,
    ) -> Result<(), NewsDomainError> {
        if !self.can_edit() {
            return Err(NewsDomainError::ArchivedArticleCannotBeModified);
        }

        let trimmed_title = new_title.trim();
        if trimmed_title.is_empty() {
            return Err(NewsDomainError::EmptyTitle);
        }

        let trimmed_body = new_body.trim();
        if trimmed_body.is_empty() {
            return Err(NewsDomainError::EmptyBody);
        }

        self.title = trimmed_title.to_string();
        self.body = trimmed_body.to_string();
        self.revision += 1;

        if self.status == NewsStatus::Rejected {
            self.status = NewsStatus::Draft;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn criar_artigo_exemplo() -> NewsArticle {
        NewsArticle::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Semana de Recepção aos Calouros".to_string(),
            "O Centro Acadêmico convida a todos para as atividades de recepção.".to_string(),
            Utc::now(),
        )
        .expect("Criação de artigo com dados válidos deve suceder")
    }

    #[test]
    fn deve_criar_noticia_com_status_draft_e_revision_inicial() {
        let artigo = criar_artigo_exemplo();
        assert_eq!(artigo.status, NewsStatus::Draft);
        assert_eq!(artigo.revision, 1);
        assert!(artigo.published_at.is_none());
        assert!(artigo.archived_at.is_none());
        assert!(artigo.can_edit());
    }

    #[test]
    fn deve_recusar_criacao_com_titulo_ou_corpo_vazios() {
        let err_titulo = NewsArticle::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "   ".to_string(),
            "Corpo válido".to_string(),
            Utc::now(),
        );
        assert_eq!(err_titulo.unwrap_err(), NewsDomainError::EmptyTitle);

        let err_corpo = NewsArticle::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Título válido".to_string(),
            "  \n  ".to_string(),
            Utc::now(),
        );
        assert_eq!(err_corpo.unwrap_err(), NewsDomainError::EmptyBody);
    }

    #[test]
    fn deve_submeter_noticia_para_revisao_com_sucesso() {
        let mut artigo = criar_artigo_exemplo();
        assert!(artigo.submit_for_review().is_ok());
        assert_eq!(artigo.status, NewsStatus::PendingReview);
    }

    #[test]
    fn deve_publicar_noticia_e_atualizar_timestamp_published_at() {
        let mut artigo = criar_artigo_exemplo();
        let instante_publicacao = Utc::now();

        assert!(artigo.publish(instante_publicacao).is_ok());
        assert_eq!(artigo.status, NewsStatus::Published);
        assert_eq!(artigo.published_at, Some(instante_publicacao));
        assert!(artigo.archived_at.is_none());
    }

    #[test]
    fn deve_arquivar_noticia_publicada_e_atualizar_timestamp_archived_at() {
        let mut artigo = criar_artigo_exemplo();
        let instante_pub = Utc::now();
        artigo.publish(instante_pub).unwrap();

        let instante_arc = Utc::now();
        assert!(artigo.archive(instante_arc).is_ok());
        assert_eq!(artigo.status, NewsStatus::Archived);
        assert_eq!(artigo.archived_at, Some(instante_arc));
        assert!(!artigo.can_edit());
    }

    #[test]
    fn deve_impedir_arquivamento_de_noticia_ja_arquivada() {
        let mut artigo = criar_artigo_exemplo();
        artigo.archive(Utc::now()).unwrap();

        let resultado = artigo.archive(Utc::now());
        assert_eq!(
            resultado.unwrap_err(),
            NewsDomainError::ArticleAlreadyArchived { id: artigo.id }
        );
    }

    #[test]
    fn deve_impedir_publicacao_de_noticia_ja_arquivada() {
        let mut artigo = criar_artigo_exemplo();
        artigo.archive(Utc::now()).unwrap();

        let resultado = artigo.publish(Utc::now());
        assert_eq!(
            resultado.unwrap_err(),
            NewsDomainError::ArticleAlreadyArchived { id: artigo.id }
        );
    }

    #[test]
    fn deve_impedir_edicao_em_noticia_arquivada() {
        let mut artigo = criar_artigo_exemplo();
        artigo.archive(Utc::now()).unwrap();

        let resultado = artigo.update_content("Novo Título".to_string(), "Novo Corpo".to_string());
        assert_eq!(
            resultado.unwrap_err(),
            NewsDomainError::ArchivedArticleCannotBeModified
        );
    }

    #[test]
    fn deve_permitir_atualizar_conteudo_e_incrementar_revisao() {
        let mut artigo = criar_artigo_exemplo();
        assert_eq!(artigo.revision, 1);

        assert!(
            artigo
                .update_content(
                    "Título Atualizado".to_string(),
                    "Novo corpo revisado da notícia.".to_string(),
                )
                .is_ok()
        );

        assert_eq!(artigo.title, "Título Atualizado");
        assert_eq!(artigo.body, "Novo corpo revisado da notícia.");
        assert_eq!(artigo.revision, 2);
    }

    #[test]
    fn deve_retornar_para_draft_ao_editar_noticia_rejeitada() {
        let mut artigo = criar_artigo_exemplo();
        artigo.submit_for_review().unwrap();
        artigo.reject().unwrap();
        assert_eq!(artigo.status, NewsStatus::Rejected);

        artigo
            .update_content(
                "Ajustes Solicitados".to_string(),
                "Corpo corrigido".to_string(),
            )
            .unwrap();

        assert_eq!(artigo.status, NewsStatus::Draft);
        assert_eq!(artigo.revision, 2);
    }
}
