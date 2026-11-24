//! Consultas ao perfil e aos vínculos do usuário no Microsoft Graph.

use reqwest::{Client, RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::{
    config::{
        HttpConfig, is_permanent_client_error, is_retryable_error, is_retryable_status,
        parse_retry_after,
    },
    modules::identity::{
        application::{
            error::AuthenticationError,
            ports::{InstitutionalAffiliation, InstitutionalDirectory, InstitutionalProfile},
        },
        infrastructure::microsoft::dto::{
            GraphCheckMemberGroupsRequest, GraphCheckMemberGroupsResponse,
            GraphOrganizationsResponse, GraphUserResponse,
        },
    },
};

/// Cliente reutilizável do diretório Microsoft.
///
/// O token de acesso pertence à operação e não é armazenado no cliente.
#[derive(Clone)]
pub struct MicrosoftGraphClient {
    /// Cliente HTTP reutilizável; deve ser configurado para recusar redirecionamentos.
    http: Client,
    /// Origem fixa em produção; substituída por Wiremock apenas nos testes.
    base_url: String,
    /// Configurações de timeout e retentativa segura.
    http_config: HttpConfig,
}

impl MicrosoftGraphClient {
    /// Constrói o adaptador a partir de um cliente HTTP já configurado e timeouts padrão.
    ///
    /// # Parâmetros
    ///
    /// - `http`: cliente reutilizável com redirecionamentos desabilitados, para manter
    ///   as credenciais nas origens esperadas.
    ///
    /// # Retorno
    ///
    /// Adaptador sem tenant, grupo ou credencial de usuário armazenados. O grupo é
    /// recebido em cada consulta. Não verifica permissões nem efetua chamadas na criação.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use backend::modules::identity::infrastructure::microsoft::MicrosoftGraphClient;
    /// let http = reqwest::Client::builder()
    ///     .redirect(reqwest::redirect::Policy::none())
    ///     .build()?;
    /// let _directory = MicrosoftGraphClient::new(http);
    /// # Ok::<(), reqwest::Error>(())
    /// ```
    pub fn new(http: Client) -> Self {
        Self::with_config(http, HttpConfig::default())
    }

    /// Constrói o adaptador com parâmetros explícitos de timeout e retentativa.
    ///
    /// # Parâmetros
    ///
    /// - `http`: cliente reutilizável com redirecionamento desabilitado.
    /// - `http_config`: configuração de timeouts e política de retentativa com backoff.
    pub fn with_config(http: Client, http_config: HttpConfig) -> Self {
        Self {
            http,
            base_url: "https://graph.microsoft.com/v1.0".to_owned(),
            http_config,
        }
    }

    /// Usa uma origem local de teste sem alterar o construtor público.
    #[allow(dead_code)]
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn new_with_base_url(http: Client, base_url: String) -> Self {
        Self::new_with_base_url_and_config(http, base_url, HttpConfig::default())
    }

    /// Usa uma origem local de teste com configuração customizada de timeouts e retentativas.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn new_with_base_url_and_config(
        http: Client,
        base_url: String,
        http_config: HttpConfig,
    ) -> Self {
        Self {
            http,
            base_url,
            http_config,
        }
    }

    /// Executa uma requisição interna com Bearer, timeout e retentativa segura.
    ///
    /// Aplica política de retentativa com backoff exponencial e jitter apenas para
    /// falhas transitórias (erros de rede/timeout ou HTTP 429, 502, 503, 504).
    /// Erros definitivos do cliente (400, 401, 403, 404) nunca são repetidos.
    ///
    /// # Invariantes de Segurança
    ///
    /// - Nunca registra o token nem o corpo recebido em logs ou mensagens de erro.
    /// - Redirecionamentos permanecem desabilitados no cliente HTTP.
    ///
    /// # Retorno
    ///
    /// Resposta HTTP 200 desserializada no DTO esperado.
    ///
    /// # Erros
    ///
    /// 401 vira `InvalidToken`; qualquer outro status diferente de 200, falha de rede,
    /// timeout ou JSON incompatível vira `ProviderUnavailable`.
    async fn send_json<T: DeserializeOwned>(
        &self,
        make_request: impl Fn() -> RequestBuilder,
        access_token: &str,
    ) -> Result<T, AuthenticationError> {
        let mut retries = 0;

        loop {
            let request = make_request()
                .bearer_auth(access_token)
                .timeout(self.http_config.request_timeout);

            let result = request.send().await;

            match result {
                Ok(response) => {
                    let status = response.status();

                    if status == StatusCode::UNAUTHORIZED {
                        return Err(AuthenticationError::InvalidToken);
                    }

                    if is_permanent_client_error(status) {
                        tracing::warn!(
                            %status,
                            "Microsoft Graph retornou erro definitivo do cliente; sem retentativa"
                        );
                        return Err(AuthenticationError::ProviderUnavailable);
                    }

                    if is_retryable_status(status) && retries < self.http_config.max_retries {
                        retries += 1;
                        let retry_after = parse_retry_after(response.headers());
                        let delay = self.http_config.compute_backoff(retries, retry_after);
                        tracing::warn!(
                            %status,
                            retries,
                            delay_ms = delay.as_millis(),
                            "Microsoft Graph retornou falha transitória; aguardando retentativa"
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    if status != StatusCode::OK {
                        tracing::warn!(
                            %status,
                            "Microsoft Graph não permitiu concluir a consulta"
                        );
                        return Err(AuthenticationError::ProviderUnavailable);
                    }

                    return response.json().await.map_err(|_| {
                        tracing::warn!("Microsoft Graph retornou uma resposta inesperada");
                        AuthenticationError::ProviderUnavailable
                    });
                }
                Err(error) => {
                    if is_retryable_error(&error) && retries < self.http_config.max_retries {
                        retries += 1;
                        let delay = self.http_config.compute_backoff(retries, None);
                        tracing::warn!(
                            timeout = error.is_timeout(),
                            retries,
                            delay_ms = delay.as_millis(),
                            "Falha de transporte/timeout com Microsoft Graph; aguardando retentativa"
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    tracing::warn!(
                        timeout = error.is_timeout(),
                        "Falha de comunicação com Microsoft Graph"
                    );
                    return Err(AuthenticationError::ProviderUnavailable);
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl InstitutionalDirectory for MicrosoftGraphClient {
    /// Consulta `GET /organization?$select=id` com a credencial do usuário.
    ///
    /// Exige um único elemento em `value`; a desserialização falha se houver zero ou
    /// mais de uma organização. Retorna seu UUID sem interpretar localmente o access
    /// token Microsoft. Falhas seguem a classificação de `send_json`.
    async fn get_tenant_id(&self, access_token: &str) -> Result<Uuid, AuthenticationError> {
        let base_url = self.base_url.clone();
        let make_request = || {
            self.http
                .get(format!("{base_url}/organization"))
                .query(&[("$select", "id")])
        };
        let [organization] = self
            .send_json::<GraphOrganizationsResponse>(make_request, access_token)
            .await?
            .value;

        Ok(organization.id)
    }

    /// Consulta `/me` selecionando ID, nome, e-mail e `jobTitle`.
    ///
    /// Converte os campos para `InstitutionalProfile` sem persistência. Para estudantes,
    /// `jobTitle` ausente, nulo, vazio ou composto apenas por espaços resulta em acesso
    /// não estudantil; outros valores são preservados, sem associação a um curso local.
    /// Não estudantes recebem `NonStudent`, sem interpretar cargos como cursos.
    /// A comparação do ID com o vínculo Firebase cabe ao serviço.
    ///
    /// # Erros
    ///
    /// Propaga recusa de credencial e falhas de comunicação ou de resposta do Graph.
    async fn get_profile(
        &self,
        access_token: &str,
        is_student: bool,
    ) -> Result<InstitutionalProfile, AuthenticationError> {
        let base_url = self.base_url.clone();
        let make_request = || {
            self.http
                .get(format!("{base_url}/me"))
                .query(&[("$select", "id,displayName,mail,jobTitle")])
        };
        let user = self
            .send_json::<GraphUserResponse>(make_request, access_token)
            .await?;

        Ok(InstitutionalProfile {
            id: user.id,
            name: user.display_name,
            mail: user.mail,
            affiliation: InstitutionalAffiliation::from_directory(is_student, user.job_title),
        })
    }

    /// Consulta `POST /me/checkMemberGroups` para um único UUID de grupo.
    ///
    /// Retorna `true` somente se `value` contiver o ID solicitado. Um array vazio
    /// representa consulta conclusiva sem participação; a verificação do Graph inclui
    /// vínculos transitivos. Não resolve nomes de grupo nem pagina listas de membros.
    ///
    /// # Erros
    ///
    /// Propaga os erros de `send_json`. Não converte falha do provedor em `false`.
    async fn is_student_member(
        &self,
        access_token: &str,
        student_group_id: Uuid,
    ) -> Result<bool, AuthenticationError> {
        let body = GraphCheckMemberGroupsRequest {
            group_ids: [student_group_id],
        };

        let base_url = self.base_url.clone();
        let make_request = || {
            self.http
                .post(format!("{base_url}/me/checkMemberGroups"))
                .json(&body)
        };

        let response = self
            .send_json::<GraphCheckMemberGroupsResponse>(make_request, access_token)
            .await?;

        Ok(response.value.contains(&student_group_id))
    }
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
