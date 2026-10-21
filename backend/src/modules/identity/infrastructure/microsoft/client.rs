//! Consultas ao perfil e aos vínculos do usuário no Microsoft Graph.

use std::time::Duration;

use reqwest::{Client, RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::modules::identity::{
    application::{
        error::AuthenticationError,
        ports::{InstitutionalDirectory, InstitutionalProfile},
    },
    infrastructure::microsoft::dto::{
        GraphCheckMemberGroupsRequest, GraphCheckMemberGroupsResponse, GraphOrganizationsResponse,
        GraphUserResponse,
    },
};

/// Cliente reutilizável do diretório Microsoft.
///
/// O token de acesso pertence à operação e não é armazenado no cliente.
#[derive(Clone)]
pub struct MicrosoftGraphClient {
    /// Cliente HTTP reutilizável; deve ser configurado para recusar redirecionamentos.
    http: Client,
}

impl MicrosoftGraphClient {
    /// Constrói o adaptador a partir de um cliente HTTP já configurado.
    ///
    /// # Parâmetros
    ///
    /// - `http`: cliente reutilizável com redirecionamentos desabilitados, para manter
    ///   as credenciais nas origens esperadas. Os métodos impõem timeout de cinco segundos.
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
        Self { http }
    }

    /// Executa uma requisição interna com Bearer e timeout de cinco segundos.
    ///
    /// Os métodos deste adaptador constroem URLs fixas do Graph. Não registra o token
    /// nem o corpo recebido. A requisição não é repetida automaticamente em caso de erro.
    ///
    /// # Retorno
    ///
    /// Resposta HTTP 200 desserializada no DTO esperado.
    ///
    /// # Erros
    ///
    /// 401 vira `InvalidToken`; qualquer outro status diferente de 200, falha de rede,
    /// timeout ou JSON incompatível vira `ProviderUnavailable`. Em particular, 403 ou
    /// 429 do Graph não são interpretados como ausência de vínculo estudantil.
    async fn send_json<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
        access_token: &str,
    ) -> Result<T, AuthenticationError> {
        let response = request
            .bearer_auth(access_token)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|error| {
                tracing::warn!(
                    timeout = error.is_timeout(),
                    "Falha de comunicação com Microsoft Graph"
                );
                AuthenticationError::ProviderUnavailable
            })?;

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(AuthenticationError::InvalidToken);
        }

        if status != StatusCode::OK {
            tracing::warn!(
                %status,
                "Microsoft Graph não permitiu concluir a consulta"
            );
            return Err(AuthenticationError::ProviderUnavailable);
        }

        response.json().await.map_err(|_| {
            tracing::warn!("Microsoft Graph retornou uma resposta inesperada");
            AuthenticationError::ProviderUnavailable
        })
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
        let request = self
            .http
            .get("https://graph.microsoft.com/v1.0/organization")
            .query(&[("$select", "id")]);
        let [organization] = self
            .send_json::<GraphOrganizationsResponse>(request, access_token)
            .await?
            .value;

        Ok(organization.id)
    }

    /// Consulta `/me` selecionando ID, nome, e-mail, UPN e `jobTitle`.
    ///
    /// Converte os campos para `InstitutionalProfile` sem persistência. `jobTitle`
    /// vazio ou composto apenas por espaços resulta em `course_name = None`; outros
    /// valores são preservados, sem normalização ou associação a um curso local.
    /// A comparação do ID com o vínculo Firebase cabe ao serviço.
    ///
    /// # Erros
    ///
    /// Propaga recusa de credencial e falhas de comunicação ou de resposta do Graph.
    async fn get_profile(
        &self,
        access_token: &str,
    ) -> Result<InstitutionalProfile, AuthenticationError> {
        let request = self
            .http
            .get("https://graph.microsoft.com/v1.0/me")
            .query(&[("$select", "id,displayName,mail,userPrincipalName,jobTitle")]);
        let user = self
            .send_json::<GraphUserResponse>(request, access_token)
            .await?;

        Ok(InstitutionalProfile {
            id: user.id,
            name: user.display_name,
            mail: user.mail,
            user_principal_name: user.user_principal_name,
            course_name: user.job_title.filter(|value| !value.trim().is_empty()),
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

        let request = self
            .http
            .post("https://graph.microsoft.com/v1.0/me/checkMemberGroups")
            .json(&body);

        let response = self
            .send_json::<GraphCheckMemberGroupsResponse>(request, access_token)
            .await?;

        Ok(response.value.contains(&student_group_id))
    }
}
