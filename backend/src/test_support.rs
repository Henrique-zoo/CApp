//! Acesso restrito pela feature `test-support` para testes externos à biblioteca.
//!
//! Constrói clientes com origens locais e um router de teste que exercita o extractor
//! real. Não altera o router de produção nem os construtores públicos dos clientes.

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use crate::{
    api::state::{AppState, Clients},
    modules::identity::{
        api::extractors::AuthenticatedInstitutionalIdentity,
        infrastructure::{firebase::FirebaseTokenVerifier, microsoft::MicrosoftGraphClient},
    },
};

/// Constrói clientes reais apontando para os servidores HTTP locais da suíte.
///
/// As URLs são fornecidas pelo teste, nunca pela requisição do usuário. A validação
/// criptográfica e o cache Firebase permanecem ativos. O cliente HTTP deve recusar
/// redirecionamentos para não encaminhar credenciais a outras origens.
///
/// Utiliza a configuração determinística rápida de testes ([`crate::config::HttpConfig::default_for_test`]).
///
/// # Erros
///
/// Propaga a rejeição de um identificador de projeto Firebase inválido.
pub fn local_identity_clients(
    project_id: String,
    http: reqwest::Client,
    jwks_url: String,
    graph_url: String,
) -> anyhow::Result<Clients> {
    local_identity_clients_with_config(
        project_id,
        http,
        jwks_url,
        graph_url,
        crate::config::HttpConfig::default_for_test(),
    )
}

/// Constrói clientes reais apontando para os servidores HTTP locais da suíte com
/// parâmetros explícitos de timeout e política de retentativa segura.
///
/// # Erros
///
/// Propaga a rejeição de um identificador de projeto Firebase inválido.
pub fn local_identity_clients_with_config(
    project_id: String,
    http: reqwest::Client,
    jwks_url: String,
    graph_url: String,
    http_config: crate::config::HttpConfig,
) -> anyhow::Result<Clients> {
    Ok(Clients::new(
        FirebaseTokenVerifier::new_with_endpoint_and_config(
            project_id,
            http.clone(),
            jwks_url,
            http_config.clone(),
        )?,
        MicrosoftGraphClient::new_with_base_url_and_config(http, graph_url, http_config),
    ))
}

/// Compõe as rotas reais de identidade com uma rota protegida exclusiva dos testes.
///
/// `GET /protected` devolve UID e instituição obtidos pelo extractor real. O router
/// ainda precisa de `AppState` e não abre conexões durante sua construção. Essa rota
/// não é adicionada por `api::router::create_router` nem pelo servidor de produção.
pub fn identity_router() -> Router<AppState> {
    crate::modules::identity::api::router().route("/protected", get(protected_identity))
}

/// Compõe as rotas reais de identidade e de Centros Acadêmicos para os testes de integração.
pub fn academic_center_router() -> Router<AppState> {
    crate::modules::identity::api::router().merge(crate::modules::ca::api::router())
}

/// Expõe o resultado do extractor para que o teste confira a identidade autenticada.
async fn protected_identity(identity: AuthenticatedInstitutionalIdentity) -> Json<Value> {
    Json(json!({
        "uid": identity.0.firebase_uid,
        "institution": identity.0.institution_id,
    }))
}
