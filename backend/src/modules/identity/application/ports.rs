//! Contratos de integração e persistência usados pela aplicação.
//!
//! Os serviços conhecem estas traits, sem depender de SQLx, Reqwest ou estado HTTP.
//! Os perfis são dados externos opcionais; identidades verificadas não comprovam
//! por si só a elegibilidade institucional. A evidência persistida é consultada
//! separadamente e associada ao UID e ao objeto Microsoft autenticados.

use uuid::Uuid;

use crate::modules::identity::{
    application::error::{
        AuthenticationError, InstitutionRepositoryError, InstitutionalVerificationRepositoryError,
    },
    domain::entities::{
        institution::Institution, institutional_verification::InstitutionalVerification,
    },
};

/// Perfil externo obtido pelo contrato do diretório institucional.
///
/// Não é o perfil interno de um usuário provisionado. `id` deve ser confrontado
/// com a identidade Microsoft vinculada ao Firebase; os outros campos podem estar
/// ausentes. O curso é texto livre informado pelo provedor, sem associação local.
#[derive(Debug, Clone)]
pub struct InstitutionalProfile {
    /// Identificador do usuário no diretório Microsoft.
    pub id: String,
    /// Nome informado pela instituição.
    pub name: Option<String>,
    /// Endereço de e-mail informado pelo diretório.
    pub mail: Option<String>,
    /// Nome de entrada da conta; não é necessariamente um e-mail.
    pub user_principal_name: Option<String>,
    /// Curso informado pela instituição, quando disponível.
    pub course_name: Option<String>,
}

/// Consultas necessárias à verificação institucional.
#[async_trait::async_trait]
pub trait InstitutionalDirectory: Send + Sync {
    /// Identifica o tenant consultado com a credencial Microsoft fornecida.
    ///
    /// # Parâmetros
    ///
    /// - `access_token`: credencial delegada do mesmo usuário consultado por `get_profile`.
    ///
    /// # Retorno
    ///
    /// UUID organizacional usado para localizar a instituição. Não aceita um tenant
    /// informado pelo cliente como evidência de origem.
    ///
    /// # Erros
    ///
    /// Credencial recusada, resposta incompatível ou indisponibilidade do diretório.
    /// A implementação Graph espera exatamente uma organização no envelope `value`.
    async fn get_tenant_id(&self, access_token: &str) -> Result<Uuid, AuthenticationError>;

    /// Obtém o perfil do usuário representado pelo access token Microsoft.
    ///
    /// # Retorno
    ///
    /// Identificador externo e atributos disponíveis. Ausência de nome, e-mail ou curso
    /// não constitui falha de autenticação. A correspondência com o Firebase cabe ao
    /// serviço chamador, não ao diretório.
    ///
    /// # Erros
    ///
    /// Retorna erro se a credencial for recusada, a resposta não corresponder ao contrato
    /// ou a consulta falhar. Nunca deve devolver dados sem confirmar a resposta do provedor.
    async fn get_profile(
        &self,
        access_token: &str,
    ) -> Result<InstitutionalProfile, AuthenticationError>;

    /// Confirma participação no grupo institucional selecionado pelo serviço.
    ///
    /// # Parâmetros
    ///
    /// - `access_token`: credencial do usuário cujo perfil já foi confrontado.
    /// - `student_group_id`: grupo configurado para o tenant da instituição.
    ///
    /// # Retorno
    ///
    /// `true` quando a participação foi confirmada; `false` quando uma consulta
    /// conclusiva não a confirmou. O adaptador Graph usa `checkMemberGroups`, incluindo
    /// participação transitiva. O nome do grupo não é usado como chave.
    ///
    /// # Erros
    ///
    /// Falhas de rede, resposta ou autorização da consulta devem retornar erro, nunca
    /// `false`. O chamador só deve persistir uma resposta conclusiva.
    async fn is_student_member(
        &self,
        access_token: &str,
        student_group_id: Uuid,
    ) -> Result<bool, AuthenticationError>;
}

/// Identidade obtida de um Firebase ID token com assinatura e claims válidas.
///
/// Não comprova elegibilidade institucional nem ausência de revogação da sessão.
#[derive(Debug, Clone)]
pub struct VerifiedIdentity {
    /// UID do usuário no projeto Firebase configurado.
    pub firebase_uid: String,
    /// Identificador da conta Microsoft vinculada, quando presente no token.
    ///
    /// É diferente do UID Firebase e deve ser confrontado com a identidade
    /// retornada pelo Microsoft Graph.
    pub microsoft_object_id: Option<String>,
}

/// Validação técnica de uma credencial de identidade.
#[async_trait::async_trait]
pub trait IdentityTokenVerifier: Send + Sync {
    /// Verifica a credencial Firebase e devolve a identidade autenticada.
    ///
    /// # Parâmetros
    ///
    /// - `token`: Firebase ID token, sem o prefixo HTTP `Bearer`.
    ///
    /// # Retorno
    ///
    /// UID e eventual identificador Microsoft vinculado. A ausência do vínculo pode
    /// ser representada por `None`; a exigência de um UUID Microsoft pertence ao caso
    /// de uso institucional. Não cria sessão ou usuário interno.
    ///
    /// # Erros
    ///
    /// Credencial inválida ou expirada, indisponibilidade das chaves ou falha interna.
    /// O adaptador atual valida assinatura e claims, sem consultar revogação da sessão
    /// ou situação atual da conta Firebase.
    async fn verify(&self, token: &str) -> Result<VerifiedIdentity, AuthenticationError>;
}

/// Consulta do cadastro confiável de instituições.
///
/// A configuração vem da persistência e não do corpo de login. A consulta não
/// executa chamadas ao Graph nem transforma status de catálogo em autorização.
#[async_trait::async_trait]
pub trait InstitutionRepository {
    /// Localiza uma instituição pelo UUID do tenant confirmado pelo diretório.
    ///
    /// # Retorno
    ///
    /// `Some` com o cadastro completo ou `None` quando o tenant não está cadastrado.
    /// O contrato pressupõe unicidade do tenant. Não filtra `CatalogStatus`.
    ///
    /// # Erros
    ///
    /// Retorna falha técnica para erro de consulta ou de mapeamento do registro.
    async fn find_by_microsoft_tenant_id(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<Institution>, InstitutionRepositoryError>;
}

/// Persistência das evidências de elegibilidade institucional.
#[async_trait::async_trait]
pub trait InstitutionalVerificationRepository: Send + Sync {
    /// Busca a evidência positiva reutilizável para um UID Firebase.
    ///
    /// # Parâmetros
    ///
    /// - `firebase_uid`: identificador obtido de um token já validado no projeto usado
    ///   pelo backend; não deve ser aceito diretamente do corpo da requisição.
    ///
    /// # Retorno
    ///
    /// `Some` somente se a participação foi confirmada, o prazo é posterior ao instante
    /// da consulta e tenant/grupo ainda correspondem à configuração institucional.
    /// Retorna `None` para ausência, resposta negativa, expiração ou configuração
    /// alterada. Não renova o prazo e não consulta o Graph. O serviço ainda deve
    /// comparar o objeto Microsoft do registro com o token atual.
    ///
    /// # Erros
    ///
    /// Falhas de consulta ou desserialização; não devem ser confundidas com `None`.
    async fn find_valid_by_firebase_uid(
        &self,
        firebase_uid: &str,
    ) -> Result<Option<InstitutionalVerification>, InstitutionalVerificationRepositoryError>;

    /// Registra uma resposta conclusiva, positiva ou negativa, por UID Firebase.
    ///
    /// # Parâmetros
    ///
    /// - `verification`: evidência construída após confronto das identidades e consulta
    ///   ao grupo; deve ter `expires_at > checked_at` e instituição existente.
    ///
    /// # Retorno
    ///
    /// `true` quando houve inserção ou substituição; `false` quando existe registro com
    /// `checked_at` igual ou posterior. O registro é substituído por inteiro; não é um
    /// histórico de verificações. A ordenação usa o início da consulta e pressupõe
    /// relógios sincronizados entre instâncias. Erros de provedor não geram registros.
    ///
    /// # Erros
    ///
    /// Falha de persistência, instituição inexistente ou violação de restrição.
    /// O contrato não inclui limpeza de registros expirados nem criação de usuário.
    async fn save(
        &self,
        verification: &InstitutionalVerification,
    ) -> Result<bool, InstitutionalVerificationRepositoryError>;
}
