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

/// Papel do usuário atuante no cenário de prestação de contas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FinancialActorRole {
    /// Membro da gestão com permissão explícita de prestação de contas.
    ManagerWithFinancialPermission,
    /// Membro da gestão ativa sem permissão de prestação de contas.
    ManagerWithoutFinancialPermission,
    /// Estudante regularmente matriculado sem cargo de gestão.
    RegularStudent,
}

/// Natureza da transação contábil manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FinancialTransactionType {
    /// Entrada de recursos financeiros.
    Income,
    /// Saída de recursos financeiros.
    Expense,
}

/// Registro histórico de uma retificação de lançamento financeiro.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FinancialRectification {
    /// Valor anterior à retificação.
    pub(crate) previous_amount: f64,
    /// Novo valor retificado.
    pub(crate) new_amount: f64,
    /// Justificativa formal da retificação.
    pub(crate) reason: String,
    /// Data em que a retificação foi registrada.
    pub(crate) date: String,
    /// Autor responsável pela retificação.
    pub(crate) author: String,
}

/// Lançamento financeiro publicado na prestação de contas do Centro Acadêmico.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FinancialEntry {
    /// Identificador do lançamento (ex.: "DESP-001", "REC-001").
    pub(crate) id: String,
    /// Natureza do lançamento (receita ou despesa).
    pub(crate) transaction_type: FinancialTransactionType,
    /// Título ou descrição concisa da operação.
    pub(crate) title: String,
    /// Valor monetário da operação.
    pub(crate) amount: f64,
    /// Data do lançamento contábil.
    pub(crate) date: String,
    /// Categoria orçamentária do lançamento.
    pub(crate) category: String,
    /// Nome ou caminho do arquivo de comprovante anexado.
    pub(crate) attachment: Option<String>,
    /// Histórico ordenado de retificações aplicadas ao lançamento.
    pub(crate) rectifications: Vec<FinancialRectification>,
}

/// Erros observáveis em tentativas inválidas de movimentação ou alteração contábil.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FinancialAccountabilityError {
    /// Recusa por ausência de permissão financeira explícita.
    Unauthorized,
    /// Recusa por lançamento financeiro não encontrado.
    EntryNotFound,
}

/// Fixture que mantém o estado da prestação de contas durante o cenário BDD.
#[derive(Debug, Clone, Default)]
pub(crate) struct FinancialAccountabilityFixture {
    /// Nome do Centro Acadêmico contextualizado no cenário.
    pub(crate) active_ca: Option<String>,
    /// Papel do usuário atuante no cenário.
    pub(crate) actor_role: Option<FinancialActorRole>,
    /// Lançamentos financeiros registrados e publicados, indexados pelo identificador.
    pub(crate) entries: HashMap<String, FinancialEntry>,
    /// Identificador do último lançamento publicado com sucesso.
    pub(crate) last_published_id: Option<String>,
    /// Erro registrado na última operação, se houver recusa.
    pub(crate) last_error: Option<FinancialAccountabilityError>,
}

impl FinancialAccountabilityFixture {
    /// Calcula a receita total acumulada de todos os lançamentos publicados.
    ///
    /// # Retorno
    ///
    /// Soma monetária de todas as transações do tipo [`FinancialTransactionType::Income`].
    pub(crate) fn total_income(&self) -> f64 {
        self.entries
            .values()
            .filter(|e| e.transaction_type == FinancialTransactionType::Income)
            .map(|e| e.amount)
            .sum()
    }

    /// Calcula a despesa total acumulada de todos os lançamentos publicados.
    ///
    /// # Retorno
    ///
    /// Soma monetária de todas as transações do tipo [`FinancialTransactionType::Expense`].
    pub(crate) fn total_expense(&self) -> f64 {
        self.entries
            .values()
            .filter(|e| e.transaction_type == FinancialTransactionType::Expense)
            .map(|e| e.amount)
            .sum()
    }

    /// Calcula o saldo financeiro consolidado do Centro Acadêmico.
    ///
    /// # Retorno
    ///
    /// Diferença entre o total de receitas e o total de despesas acumuladas.
    pub(crate) fn consolidated_balance(&self) -> f64 {
        self.total_income() - self.total_expense()
    }
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
    /// Estado da prestação de contas e transparência financeira no cenário de teste.
    pub(crate) financial_accountability: FinancialAccountabilityFixture,
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
            .field("financial_accountability", &self.financial_accountability)
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
