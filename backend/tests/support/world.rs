//! Contexto de cada cenário e requisições em memória ao router real.
//!
//! [`AppWorld`] nasce sem recursos externos. Os hooks associam banco e router
//! antes dos steps e os liberam ao término. Respostas e resultados pertencem
//! a cada cenário, mesmo quando o pool consultivo é compartilhado.

use std::{collections::HashMap, fmt};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde::de::DeserializeOwned;
use sqlx::PgPool;
use tower::ServiceExt;

use super::{
    database::{SuiteDatabase, TestDatabase},
    results::{TestDatabaseMutation, TestResponse},
};

/// Papel do usuário atuante no cenário de publicação institucional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstitutionalActorRole {
    /// Membro com permissão da gestão ativa do Centro Acadêmico.
    ActiveAuthorizedManager,
    /// Membro de uma gestão anterior já encerrada.
    PastManager,
    /// Estudante regularmente matriculado sem cargo de gestão.
    RegularStudent,
}

/// Registro de versão de documento institucional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstitutionalDocumentVersion {
    /// Identificador textual da versão (ex.: "1.0", "1.1").
    pub(crate) version: String,
    /// Conteúdo normativo do documento.
    pub(crate) content: String,
    /// Data simulada da atualização ou publicação da versão.
    pub(crate) recorded_at: String,
    /// Autor ou responsável pelo registro da versão.
    pub(crate) author: String,
}

/// Documento institucional com metadados e histórico versionado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstitutionalDocument {
    /// Tipo classificador do documento (ex.: "Estatuto Social", "Regimento Interno").
    pub(crate) doc_type: String,
    /// Título formal do documento institucional.
    pub(crate) title: String,
    /// Versão atualmente vigente para visualização pública.
    pub(crate) current_version: String,
    /// Histórico ordenado de versões do documento.
    pub(crate) versions: Vec<InstitutionalDocumentVersion>,
    /// Data da última alteração do documento vigente.
    pub(crate) updated_at: String,
}

/// Dados oficiais de canais de contato e composição da gestão ativa.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct InstitutionalContactAndManagement {
    /// Endereço eletrônico oficial de contato.
    pub(crate) official_email: String,
    /// Canal de atendimento ao estudante.
    pub(crate) service_channel: String,
    /// Lista de cargos e ocupantes na gestão ativa.
    pub(crate) management_members: Vec<(String, String)>,
}

/// Rejeições e erros de negócio em operações de publicação institucional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InstitutionalPublicationError {
    /// Recusa por falta de permissão administrativa.
    Unauthorized,
    /// Recusa por usuário pertencer a gestão inativa/encerrada.
    InactiveManagement,
    /// Recusa por tentativa de atualizar documento inexistente.
    DocumentNotFound,
}

/// Fixture que mantém o estado da publicação institucional no cenário BDD.
#[derive(Debug, Clone, Default)]
pub(crate) struct InstitutionalPublicationFixture {
    /// Nome do Centro Acadêmico ativo no cenário.
    pub(crate) active_ca: Option<String>,
    /// Papel do ator atual no cenário.
    pub(crate) actor_role: Option<InstitutionalActorRole>,
    /// Acervo de documentos institucionais publicados, indexados pelo tipo.
    pub(crate) documents: HashMap<String, InstitutionalDocument>,
    /// Informações de contato e composição da gestão vigente.
    pub(crate) contact_info: Option<InstitutionalContactAndManagement>,
    /// Indica se as informações publicadas estão disponíveis para a comunidade.
    pub(crate) publicly_available: bool,
    /// Erro registrado na última operação, se houver recusa.
    pub(crate) last_error: Option<InstitutionalPublicationError>,
    /// Última versão de documento publicada com sucesso.
    pub(crate) last_published_version: Option<String>,
}

/// Contexto independente criado pelo Cucumber para cada cenário.
///
/// [`Default`] inicia sem banco ou router, sem resultados e com contador de
/// requisições zerado. O hook `before` associa os recursos conforme a tag de
/// isolamento; o hook `after` os libera. Steps devem acessar o pool por
/// [`Self::database_pool`] e enviar requisições por [`Self::get`] ou
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
    /// Estado da publicação de informações institucionais no cenário de teste.
    pub(crate) institutional_publication: InstitutionalPublicationFixture,
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
            .field("institutional_publication", &self.institutional_publication)
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
