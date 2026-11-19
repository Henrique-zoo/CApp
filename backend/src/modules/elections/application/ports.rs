//! Portas de entrada e saída dos casos de uso eleitorais.
//!
//! Os serviços dependem destas abstrações, nunca de HTTP, SQLx ou Firebase.
//! As implementações concretas pertencem às camadas de infraestrutura.

use chrono::{DateTime, Utc};

use crate::modules::{
    elections::{
        application::error::{
            AuthenticatedElectorError, ClockError, ElectionReaderError, ElectionStoreError,
        },
        domain::{
            entities::{
                ballot::VerifiedBallot,
                election::Election,
                participation::{Participation, ParticipationError, ParticipationKey},
            },
            value_objects::identifiers::{AccountId, ElectionId, VoterId},
        },
    },
    identity::application::services::authentication::AuthenticatedIdentity,
};

/// Identidade interna autenticada resolvida para uma eleição.
///
/// O UID Firebase não é exposto como identificador eleitoral público.
/// O `voter_id` é opaco e estável para a mesma conta dentro da eleição,
/// sem permitir correlação pública entre eleições.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticatedElectorContext {
    user_id: AccountId,
    voter_id: VoterId,
    election_id: ElectionId,
}

impl AuthenticatedElectorContext {
    /// Cria o contexto após a implementação da porta validar a identidade.
    ///
    /// Os identificadores devem ser obtidos de fontes confiáveis.
    /// Nunca devem ser aceitos diretamente do corpo da requisição.
    #[allow(
        dead_code,
        reason = "A implementação concreta de AuthenticatedElector será adicionada em etapa posterior; o construtor permanece restrito ao crate para proteger a criação do contexto autenticado."
    )]
    pub(crate) const fn new(
        user_id: AccountId,
        voter_id: VoterId,
        election_id: ElectionId,
    ) -> Self {
        Self {
            user_id,
            voter_id,
            election_id,
        }
    }

    /// Retorna o identificador interno da conta autenticada.
    pub const fn user_id(&self) -> AccountId {
        self.user_id
    }

    /// Retorna o identificador opaco do eleitor nesta eleição.
    pub const fn voter_id(&self) -> VoterId {
        self.voter_id
    }

    /// Retorna a eleição para a qual a identidade foi resolvida.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }

    /// Combina os identificadores confiáveis usados na unicidade transacional.
    pub const fn participation_key(&self) -> ParticipationKey {
        ParticipationKey::new(self.election_id, self.user_id)
    }

    /// Prepara um novo aceite mantendo conta, eleitor público e eleição juntos.
    ///
    /// Recusa contexto de outra eleição antes de aplicar as regras do domínio.
    /// A aplicação chama este método após reavaliar elegibilidade na transação.
    /// Não grava dados nem confirma o voto; consultas de aceite anterior usam
    /// `BallotStore::find_participation`, sem exigir nova elegibilidade.
    pub fn prepare_participation(
        &self,
        election: &Election,
        ballot: VerifiedBallot,
        accepted_at: DateTime<Utc>,
    ) -> Result<Participation, ParticipationError> {
        if election.id() != self.election_id || ballot.ballot().election_id() != self.election_id {
            return Err(ParticipationError::IdentityMismatch);
        }
        Participation::try_accept(election, self.user_id, self.voter_id, ballot, accepted_at)
    }
}

/// Resolve a identidade autenticada e sua elegibilidade para uma eleição.
///
/// Recebe uma identidade já validada pelo módulo de identidade. Não recebe
/// token Firebase, UID declarado pelo cliente ou credenciais externas.
///
/// Esta consulta não reserva participação nem garante que um voto futuro
/// será aceito. A elegibilidade deve ser reavaliada na operação transacional
/// que efetivamente registrar a submissão.
#[async_trait::async_trait]
pub trait AuthenticatedElector: Send + Sync {
    /// Resolve o contexto eleitoral de uma identidade autenticada.
    async fn resolve(
        &self,
        identity: &AuthenticatedIdentity,
        election_id: ElectionId,
    ) -> Result<AuthenticatedElectorContext, AuthenticatedElectorError>;
}

/// Agregado eleitoral acompanhado da versão de persistência.
///
/// `version` pertence ao armazenamento e não deve ser confundida com
/// `Election::schedule_revision()`, que representa somente o calendário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedElection {
    election: Election,
    version: u64,
}

impl VersionedElection {
    /// Cria uma fotografia versionada recuperada pelo adaptador.
    #[allow(
        dead_code,
        reason = "A implementação de ElectionReader de produção será adicionada em etapa posterior; atualmente o construtor é utilizado pelo teste do caso de uso."
    )]
    pub(crate) const fn new(election: Election, version: u64) -> Self {
        Self { election, version }
    }

    /// Retorna o agregado eleitoral.
    pub const fn election(&self) -> &Election {
        &self.election
    }

    /// Retorna a versão usada no controle de concorrência.
    pub const fn version(&self) -> u64 {
        self.version
    }

    /// Consome a fotografia e devolve o agregado.
    pub fn into_election(self) -> Election {
        self.election
    }
}

/// Consulta eleições sem expor operações de escrita.
///
/// A ausência de uma eleição é representada por `Ok(None)`.
#[async_trait::async_trait]
pub trait ElectionReader: Send + Sync {
    /// Recupera uma eleição e sua versão de persistência.
    ///
    /// `Ok(None)` representa ausência. Falhas técnicas ou dados inválidos
    /// devem ser retornados como erros tipados.
    async fn find_by_id(
        &self,
        election_id: ElectionId,
    ) -> Result<Option<VersionedElection>, ElectionReaderError>;
}

/// Persistência genérica do agregado raiz eleitoral.
///
/// Estende `ElectionReader` para permitir que uma implementação de
/// persistência também recupere o agregado antes de atualizá-lo.
///
/// As atualizações usam controle otimista de concorrência. A implementação
/// deve confirmar a gravação somente se a versão persistida ainda corresponder
/// à versão esperada.
///
/// Esta porta genérica não substitui as operações transacionais especializadas
/// de voto, cadeia de registros, encerramento, cancelamento e apuração definidas
/// no contrato em `packages/api-contracts/elections/interfaces.json`.
#[async_trait::async_trait]
pub trait ElectionStore: ElectionReader {
    /// Persiste uma alteração validada do agregado.
    ///
    /// `expected_version` deve ser a versão obtida na leitura anterior.
    /// A implementação deve realizar comparação e gravação atomicamente,
    /// retornando `Conflict` se a versão já tiver sido alterada.
    ///
    /// Retorna a nova versão persistida.
    async fn save(
        &self,
        election: &Election,
        expected_version: u64,
    ) -> Result<u64, ElectionStoreError>;
}

/// Fonte de tempo confiável do backend.
///
/// Não deve usar o relógio informado pelo cliente.
pub trait Clock: Send + Sync {
    /// Retorna o instante atual em UTC.
    fn now(&self) -> Result<DateTime<Utc>, ClockError>;
}
