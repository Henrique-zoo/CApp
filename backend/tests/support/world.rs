//! Contexto de cada cenário e acesso ao banco de testes e ao router real.
//!
//! [`AppWorld`] nasce sem recursos externos. Os hooks associam banco e router
//! antes dos steps e os liberam ao término. Respostas e resultados pertencem
//! a cada cenário, mesmo quando o pool consultivo é compartilhado.

use std::{collections::HashSet, fmt};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use super::{
    database::{SuiteDatabase, TestDatabase},
    results::{TestDatabaseMutation, TestResponse},
};

/// Contexto independente criado pelo Cucumber para cada cenário.
///
/// [`Default`] inicia sem banco ou router, sem resultados e com contador de
/// requisições zerado. O hook `before` associa os recursos conforme a tag de
/// isolamento; o hook `after` os libera. Steps de integração acessam o pool por
/// [`Self::database_pool`] e enviam requisições por [`Self::get`] ou
/// [`Self::get_json`].
///
/// Os campos de resultados SQL são preenchidos pelos steps que executarem
/// mutações. Os helpers HTTP não os atualizam automaticamente.
#[derive(cucumber::World, Default)]
pub(crate) struct AppWorld {
    /// Banco associado pelo hook `before` e liberado pelo hook `after`.
    database: Option<TestDatabase>,
    /// Router real com o estado associado ao pool desse cenário.
    app: Option<Router>,
    /// Resultados das operações concorrentes, na ordem definida pelo step.
    pub(crate) concurrent_database_mutations: Vec<TestDatabaseMutation>,
    /// Resultado da última tentativa de mutação sequencial.
    pub(crate) last_database_mutation: Option<TestDatabaseMutation>,
    /// Última resposta HTTP capturada pelos helpers de requisição.
    pub(crate) last_response: Option<TestResponse>,
    /// Número de tentativas de GET iniciadas após a obtenção do router.
    ///
    /// Inclui chamadas via `get_json` e aumenta antes de construir a requisição;
    /// uma URI inválida ou falha de leitura também pode incrementar o contador.
    pub(crate) request_count: usize,
    /// Gerenciamento em memória e regras do mural de notícias (#77).
    pub(crate) news_fixture: NewsFixture,
}

impl fmt::Debug for AppWorld {
    /// Formata o contexto do cenário sem expor pools ou o container.
    ///
    /// Inclui modo do banco, presença do router, resultados SQL, última resposta
    /// e contador de requisições. O corpo HTTP é incluído no diagnóstico.
    ///
    /// # Erros
    ///
    /// Propaga falhas de escrita retornadas pelo formatador.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppWorld")
            .field(
                "database_kind",
                &self.database.as_ref().map(TestDatabase::kind),
            )
            .field("app_ready", &self.app.is_some())
            .field(
                "concurrent_database_mutations",
                &self.concurrent_database_mutations,
            )
            .field("last_database_mutation", &self.last_database_mutation)
            .field("last_response", &self.last_response)
            .field("request_count", &self.request_count)
            .field("news_fixture", &self.news_fixture)
            .finish()
    }
}

impl AppWorld {
    /// Obtém o pool associado ao cenário pelo hook `before`.
    ///
    /// # Retorno
    ///
    /// Referência ao pool compartilhado de leitura ou ao pool isolado do cenário.
    /// Steps que escrevem devem estar em cenários com `@isolated_database`.
    ///
    /// # Panics
    ///
    /// Se o hook ainda não associou o banco ou a limpeza já o removeu do World.
    pub(crate) fn database_pool(&self) -> &PgPool {
        &self
            .database
            .as_ref()
            .expect("BDD database should be initialized")
            .pool
    }

    /// Associa o banco e o router ao World antes da execução dos steps.
    ///
    /// # Parâmetros
    ///
    /// - `suite`: recursos compartilhados e acesso administrativo ao banco de testes.
    /// - `isolated`: solicita uma cópia gravável da base-modelo quando `true`.
    ///
    /// Delega a preparação a [`SuiteDatabase::prepare_scenario`] e só armazena
    /// os recursos depois que essa operação termina. Não reinicializa os campos
    /// de resultados: cada cenário deve começar com um novo World.
    ///
    /// # Panics
    ///
    /// Em associação duplicada, falha de clonagem ou conexão. Uma base criada
    /// antes de falha de conexão poderá ser recolhida no encerramento da suíte.
    pub(super) async fn attach_database(&mut self, suite: &SuiteDatabase, isolated: bool) {
        assert!(
            self.database.is_none() && self.app.is_none(),
            "a Cucumber world should receive its database only once",
        );

        let (database, app) = suite.prepare_scenario(isolated).await;
        self.database = Some(database);
        self.app = Some(app);
    }

    /// Retira do World os recursos usados pelo cenário e libera seu banco.
    ///
    /// Descarta o router antes de delegar a limpeza a
    /// [`SuiteDatabase::release_scenario`]. Tolera um World sem banco e pode ser
    /// chamado novamente após a limpeza. Mantém os resultados das operações para
    /// diagnóstico, mas o World deixa de aceitar consultas e requisições.
    ///
    /// # Panics
    ///
    /// Se a remoção da base isolada falhar. O recurso já terá sido retirado do World;
    /// a busca de bases remanescentes poderá tentar a remoção no final da suíte.
    pub(super) async fn detach_database(&mut self, suite: &SuiteDatabase) {
        self.app.take();

        let Some(database) = self.database.take() else {
            return;
        };

        suite.release_scenario(database).await;
    }

    /// Executa GET no router real e preserva status e corpo da resposta.
    ///
    /// # Parâmetros
    ///
    /// - `path`: URI aceita por [`Request::builder`], incluindo query string.
    ///
    /// Usa [`ServiceExt::oneshot`] com corpo vazio, sem abrir um socket HTTP.
    /// Incrementa [`Self::request_count`] antes de construir a requisição e substitui
    /// [`Self::last_response`] somente após ler e converter o corpo por completo.
    /// Status 4xx e 5xx também são capturados, sem causar panic por si só.
    ///
    /// O helper destina-se a respostas textuais de teste: exige UTF-8 e coleta o
    /// corpo inteiro em memória, com limite de `usize::MAX` bytes. Não há timeout
    /// adicional além daqueles implementados pelo próprio router.
    ///
    /// # Panics
    ///
    /// Se o router não estiver associado, a URI for inválida ou ocorrer falha ao
    /// processar a requisição, coletar o corpo ou convertê-lo para UTF-8. Quando
    /// há falha antes de armazenar a nova resposta, a resposta anterior permanece.
    pub(crate) async fn get(&mut self, path: &str) {
        let app = self
            .app
            .as_ref()
            .expect("backend API should be initialized before requests")
            .clone();
        self.request_count += 1;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(path)
                    .body(Body::empty())
                    .expect("request should be valid"),
            )
            .await
            .expect("request should be handled by backend router");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");

        self.last_response = Some(TestResponse {
            status,
            body: String::from_utf8(body.to_vec()).expect("response body should be UTF-8"),
        });
    }

    /// Executa GET e desserializa o corpo de uma resposta HTTP 200.
    ///
    /// # Parâmetros
    ///
    /// - `path`: URI enviada a [`Self::get`].
    /// - `T`: tipo de destino que implementa [`DeserializeOwned`].
    ///
    /// # Retorno
    ///
    /// Corpo convertido para `T`. A resposta textual continua em
    /// [`Self::last_response`], inclusive se a validação de status ou JSON falhar.
    /// O contador aumenta uma única vez por meio de `get`. Não verifica o cabeçalho
    /// `Content-Type`; a validação usa o status e a desserialização do corpo.
    ///
    /// # Panics
    ///
    /// Nas falhas de [`Self::get`], em status diferente de 200 ou quando o corpo
    /// não puder ser desserializado para `T`.
    pub(crate) async fn get_json<T>(&mut self, path: &str) -> T
    where
        T: DeserializeOwned,
    {
        self.get(path).await;

        let response = self
            .last_response
            .as_ref()
            .expect("a response should have been captured");

        assert_eq!(response.status, StatusCode::OK);

        serde_json::from_str(&response.body).expect("response should match the expected JSON shape")
    }
}

/// Situação editorial de um artigo de notícia no mural do Centro Acadêmico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NewsStatusState {
    /// Registrada como solicitação / rascunho, pendente de validação por um administrador.
    Solicitacao,
    /// Homologada e publicada no mural do Centro Acadêmico.
    Publicada,
    /// Rejeitada pela moderação ou administração do Centro Acadêmico.
    Rejeitada,
}

/// Representação em memória de um artigo de notícia no contexto de testes BDD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewsArticleState {
    /// Identificador estável do artigo de notícia.
    pub(crate) id: Uuid,
    /// Centro Acadêmico ao qual a notícia pertence.
    pub(crate) ca_name: String,
    /// Identificador ou e-mail do autor responsável.
    pub(crate) author: String,
    /// Título da notícia divulgado no mural.
    pub(crate) title: String,
    /// Conteúdo textual do artigo.
    pub(crate) body: String,
    /// Situação editorial atual.
    pub(crate) status: NewsStatusState,
    /// Instante em que o registro inicial foi criado.
    pub(crate) created_at: DateTime<Utc>,
    /// Instante em que o artigo foi efetivamente publicado, se aplicável.
    pub(crate) published_at: Option<DateTime<Utc>>,
}

/// Fixture e gerenciamento de estado em memória para os cenários de notícias (#77).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewsFixture {
    /// Identificador do usuário em foco no cenário (ex.: e-mail institucional).
    pub(crate) current_user: String,
    /// Centro Acadêmico do qual o usuário faz parte no cenário.
    pub(crate) user_ca: Option<String>,
    /// Centros Acadêmicos nos quais o usuário possui permissão de edição.
    pub(crate) edit_permissions: HashSet<String>,
    /// Centros Acadêmicos nos quais o usuário possui permissão de publicação direta.
    pub(crate) direct_publish_permissions: HashSet<String>,
    /// Artigos de notícias persistidos em memória.
    pub(crate) articles: Vec<NewsArticleState>,
    /// Indica se a última operação de criação, edição ou publicação foi aceita.
    pub(crate) last_operation_succeeded: Option<bool>,
    /// Identificador da última notícia criada ou editada.
    pub(crate) last_created_article_id: Option<Uuid>,
    /// Notícia alvo selecionada para ações subsequentes no cenário.
    pub(crate) target_article_id: Option<Uuid>,
    /// Listagem de notícias publicadas formada na última consulta.
    pub(crate) published_listing: Option<Vec<NewsArticleState>>,
}

impl Default for NewsFixture {
    fn default() -> Self {
        Self {
            current_user: "estudante.unb@aluno.unb.br".to_string(),
            user_ca: None,
            edit_permissions: HashSet::new(),
            direct_publish_permissions: HashSet::new(),
            articles: Vec::new(),
            last_operation_succeeded: None,
            last_created_article_id: None,
            target_article_id: None,
            published_listing: None,
        }
    }
}

/// Valida se uma cadeia textual representa título ou conteúdo aceitável.
///
/// Rejeita strings vazias, contendo apenas espaços ou marcadores de template
/// do Gherkin (ex.: `<titulo>` ou `<conteudo>`).
pub(crate) fn is_valid_news_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.starts_with('<') && trimmed.ends_with('>') {
        return false;
    }
    true
}

impl NewsFixture {
    /// Associa o usuário atual ao Centro Acadêmico especificado.
    pub(crate) fn set_user_ca(&mut self, ca_name: impl Into<String>) {
        self.user_ca = Some(ca_name.into());
    }

    /// Retorna o Centro Acadêmico do usuário ou o padrão didático.
    pub(crate) fn get_effective_ca(&self) -> String {
        self.user_ca
            .clone()
            .unwrap_or_else(|| "CA de Computação".to_string())
    }

    /// Configura a permissão de edição do usuário para determinado CA.
    pub(crate) fn set_can_edit(&mut self, ca_name: impl Into<String>, allowed: bool) {
        let ca = ca_name.into();
        if allowed {
            self.edit_permissions.insert(ca);
        } else {
            self.edit_permissions.remove(&ca);
        }
    }

    /// Configura a permissão de publicação direta do usuário para determinado CA.
    pub(crate) fn set_can_publish_directly(&mut self, ca_name: impl Into<String>, allowed: bool) {
        let ca = ca_name.into();
        if allowed {
            self.direct_publish_permissions.insert(ca);
        } else {
            self.direct_publish_permissions.remove(&ca);
        }
    }

    /// Cria uma nova notícia no Centro Acadêmico como solicitação.
    pub(crate) fn create_news(&mut self, ca_name: &str, title: &str, body: &str) {
        if !is_valid_news_text(title) || !is_valid_news_text(body) {
            self.last_operation_succeeded = Some(false);
            return;
        }

        let id = Uuid::new_v4();
        let article = NewsArticleState {
            id,
            ca_name: ca_name.to_string(),
            author: self.current_user.clone(),
            title: title.to_string(),
            body: body.to_string(),
            status: NewsStatusState::Solicitacao,
            created_at: Utc::now(),
            published_at: None,
        };

        self.articles.push(article);
        self.last_created_article_id = Some(id);
        self.target_article_id = Some(id);
        self.last_operation_succeeded = Some(true);
    }

    /// Cadastra uma notícia pré-existente no cenário.
    pub(crate) fn add_existing_article(
        &mut self,
        ca_name: &str,
        title: &str,
        body: &str,
        status: NewsStatusState,
        published_at: Option<DateTime<Utc>>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let article = NewsArticleState {
            id,
            ca_name: ca_name.to_string(),
            author: self.current_user.clone(),
            title: title.to_string(),
            body: body.to_string(),
            status,
            created_at: Utc::now(),
            published_at,
        };

        self.articles.push(article);
        self.target_article_id = Some(id);
        id
    }

    /// Atualiza o título e o conteúdo de um artigo existente.
    pub(crate) fn update_article(&mut self, old_title: &str, new_title: &str, new_body: &str) {
        let article_idx = self
            .target_article_id
            .and_then(|id| self.articles.iter().position(|a| a.id == id))
            .or_else(|| self.articles.iter().position(|a| a.title == old_title));

        let Some(idx) = article_idx else {
            self.last_operation_succeeded = Some(false);
            return;
        };

        let ca_name = self.articles[idx].ca_name.clone();
        if !self.edit_permissions.contains(&ca_name) {
            self.last_operation_succeeded = Some(false);
            return;
        }

        if !is_valid_news_text(new_title) || !is_valid_news_text(new_body) {
            self.last_operation_succeeded = Some(false);
            return;
        }

        self.articles[idx].title = new_title.to_string();
        self.articles[idx].body = new_body.to_string();
        self.last_operation_succeeded = Some(true);
        self.target_article_id = Some(self.articles[idx].id);
    }

    /// Publica diretamente um artigo existente.
    pub(crate) fn publish_directly(&mut self, title: Option<&str>) {
        let article_idx = title
            .and_then(|t| self.articles.iter().position(|a| a.title == t))
            .or_else(|| {
                self.target_article_id
                    .and_then(|id| self.articles.iter().position(|a| a.id == id))
            });

        let Some(idx) = article_idx else {
            self.last_operation_succeeded = Some(false);
            return;
        };

        let ca_name = self.articles[idx].ca_name.clone();
        if !self.direct_publish_permissions.contains(&ca_name) {
            self.last_operation_succeeded = Some(false);
            return;
        }

        if !is_valid_news_text(&self.articles[idx].title)
            || !is_valid_news_text(&self.articles[idx].body)
        {
            self.last_operation_succeeded = Some(false);
            return;
        }

        self.articles[idx].status = NewsStatusState::Publicada;
        self.articles[idx].published_at = Some(Utc::now());
        self.last_operation_succeeded = Some(true);
        self.target_article_id = Some(self.articles[idx].id);
    }

    /// Consulta e forma a listagem de notícias publicadas para o CA informado.
    pub(crate) fn form_published_listing(&mut self, ca_name: &str) {
        let published: Vec<NewsArticleState> = self
            .articles
            .iter()
            .filter(|a| a.ca_name == ca_name && a.status == NewsStatusState::Publicada)
            .cloned()
            .collect();
        self.published_listing = Some(published);
    }

    /// Recupera o artigo criado mais recentemente ou o alvo atual.
    pub(crate) fn get_current_article(&self) -> Option<&NewsArticleState> {
        self.target_article_id
            .and_then(|id| self.articles.iter().find(|a| a.id == id))
            .or_else(|| self.articles.last())
    }

    /// Verifica se um artigo específico está presente na listagem publicada formada.
    pub(crate) fn published_listing_contains(&self, article_id: Uuid) -> bool {
        self.published_listing
            .as_ref()
            .is_some_and(|listing| listing.iter().any(|a| a.id == article_id))
    }

    /// Verifica se a notícia está publicada e disponível aos usuários de um CA.
    pub(crate) fn is_published_for_ca(&self, article_id: Uuid, ca_name: &str) -> bool {
        self.articles.iter().any(|a| {
            a.id == article_id && a.ca_name == ca_name && a.status == NewsStatusState::Publicada
        })
    }
}
