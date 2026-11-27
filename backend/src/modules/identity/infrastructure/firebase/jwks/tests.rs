//! Regressões do mecanismo compartilhado de JWKS, com HTTP local e chaves fictícias.
//!
//! Exercitam cache, concorrência, rotação, cancelamento e falhas sem Firebase
//! externo. As regras de claims permanecem nas suítes dos adaptadores.

use super::*;
use serde_json::{Value, json};
use tokio::time::timeout;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const HTTP_POLICY: CachePolicy = CachePolicy {
    fallback_ttl: Duration::ZERO,
    maximum_ttl: None,
};

const TEST_JWKS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/jwt/jwks.json"
));

fn new_test_keys(server: &MockServer) -> JwksCache {
    JwksCache::new(Client::new(), format!("{}/keys", server.uri()), HTTP_POLICY)
}

async fn mock_jwks(server: &MockServer, max_age: u64, expected_calls: u64) {
    Mock::given(method("GET"))
        .and(path("/keys"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Cache-Control", format!("public, max-age={max_age}"))
                .set_delay(Duration::from_millis(25))
                .set_body_raw(TEST_JWKS, "application/json"),
        )
        .expect(expected_calls)
        .mount(server)
        .await;
}

/// Contém downloads provocados por kid desconhecido durante o intervalo mínimo.
#[tokio::test]
async fn unknown_kid_during_fresh_cache_does_not_trigger_another_download() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_keys(&server);

    verifier.key_for("test-kid").await.unwrap();

    assert!(matches!(
        verifier.key_for("nonexistent-kid").await,
        Err(JwksError::UnknownKey)
    ));
}

/// TTL zero atende apenas a busca atual e mantém contenção nas seguintes.
#[tokio::test]
async fn zero_ttl_applies_backoff_and_does_not_requery_per_request() {
    let server = MockServer::start().await;

    mock_jwks(&server, 0, 1).await;

    let verifier = new_test_keys(&server);

    verifier.key_for("test-kid").await.unwrap();

    assert!(matches!(
        verifier.key_for("test-kid").await,
        Err(JwksError::Unavailable)
    ));
}

/// Expirar uma chave antes do próximo refresh impede reutilizá-la na contenção.
#[tokio::test]
async fn expired_known_keys_are_not_used_during_backoff() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let keys = new_test_keys(&server);

    keys.key_for("test-kid").await.unwrap();

    keys.cache.write().await.valid_until = Instant::now();

    assert!(matches!(
        keys.key_for("test-kid").await,
        Err(JwksError::Unavailable)
    ));

    server.verify().await;
}

/// Falha HTTP do JWKS é indisponibilidade e ativa backoff.
#[tokio::test]
async fn server_failure_is_provider_unavailable() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/keys"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;

    let verifier = new_test_keys(&server);

    assert!(matches!(
        verifier.key_for("test-kid").await,
        Err(JwksError::Unavailable)
    ));
    // A consulta imediata seguinte não deve causar outra requisição.
    assert!(matches!(
        verifier.key_for("test-kid").await,
        Err(JwksError::Unavailable)
    ));
}

/// Leitores de uma chave válida não aguardam o mutex de atualização.
#[tokio::test]
async fn cached_read_does_not_wait_for_refresh_mutex() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_keys(&server);

    verifier.key_for("test-kid").await.unwrap();

    // Simula que outra tarefa está atualizando o JWKS: o mutex de atualização
    // está adquirido, mas o RwLock NÃO deve impedir leitores de chaves válidas.

    let _guard = verifier.refresh_lock.lock().await;
    let found = timeout(Duration::from_millis(500), verifier.key_for("test-kid")).await;

    assert!(
        found
            .expect("cache válido não deveria aguardar o refresh")
            .is_ok()
    );
}

/// Cache-Control é interpretado sem prolongar validade além da idade da resposta.
#[test]
fn cache_control_directives_and_age_are_respected() {
    for (control, age, expected) in [
        ("public, max-age=300", "20", 280),
        ("MAX-AGE=\"60\"", "0", 60),
        ("max-age=10", "100", 0),
        ("max-age=60, no-store", "0", 0),
        ("no-cache, max-age=60", "0", 0),
        ("max-age=invalid", "0", 0),
        ("max-age=-1", "0", 0),
        ("max-age=30", "invalid", 30),
        ("max-age=10, max-age=20", "0", 10),
        ("public", "0", 0),
    ] {
        let mut headers = HeaderMap::new();

        headers.insert(CACHE_CONTROL, control.parse().unwrap());
        headers.insert(AGE, age.parse().unwrap());

        assert_eq!(
            cache_ttl(&headers, HTTP_POLICY),
            Duration::from_secs(expected),
            "{control}, Age={age}"
        );
    }

    assert_eq!(cache_ttl(&HeaderMap::new(), HTTP_POLICY), Duration::ZERO);
}

/// JSON JWKS inválido ou sem chave RSA de assinatura utilizável é erro do provedor.
#[tokio::test]
async fn invalid_jwks_never_populates_cache() {
    let original: Value = serde_json::from_str(TEST_JWKS).unwrap();
    let mut bodies = vec!["not-json".to_owned(), "{}".into(), "{\"keys\":[]}".into()];

    for (field, value) in [
        ("alg", json!("RS512")),
        ("use", json!("enc")),
        ("kid", json!("")),
        ("n", json!("!")),
    ] {
        let mut body = original.clone();

        body["keys"][0][field] = value;

        bodies.push(body.to_string());
    }

    let mut no_kid = original.clone();

    no_kid["keys"][0].as_object_mut().unwrap().remove("kid");
    bodies.push(no_kid.to_string());
    bodies.push(json!({"keys":[{"kty":"oct", "k":"YWJj", "kid":"test-kid"}]}).to_string());

    let key = original["keys"][0].clone();

    bodies.push(json!({"keys": [key.clone(), key]}).to_string());

    for body in bodies {
        let server = MockServer::start().await;

        Mock::given(path("/keys"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;

        let verifier = new_test_keys(&server);

        assert!(matches!(
            verifier.key_for("test-kid").await,
            Err(JwksError::Unavailable)
        ));

        assert!(verifier.cache.read().await.keys.is_empty());
    }
}

/// Uma chave nova pode ser obtida após o intervalo, e a chave antiga deixa o cache.
#[tokio::test]
async fn expired_cache_refreshes_rotated_keys() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_keys(&server);

    verifier.key_for("test-kid").await.unwrap();

    server.verify().await;

    server.reset().await;

    let mut jwks: Value = serde_json::from_str(TEST_JWKS).unwrap();

    jwks["keys"][0]["kid"] = json!("rotated");

    Mock::given(path("/keys"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("cache-control", "max-age=300")
                .set_body_json(jwks),
        )
        .expect(1)
        .mount(&server)
        .await;

    {
        let mut cache = verifier.cache.write().await;

        cache.valid_until = Instant::now();
        cache.refresh_after = Instant::now();
    }

    assert!(verifier.key_for("rotated").await.is_ok());

    assert!(matches!(
        verifier.key_for("test-kid").await,
        Err(JwksError::UnknownKey)
    ));
}

/// Cancelar uma busca em andamento mantém o backoff e não deixa um lock adquirido.
#[tokio::test]
async fn cancelled_fetch_preserves_backoff_without_deadlock() {
    let server = MockServer::start().await;
    let started = Arc::new(tokio::sync::Notify::new());
    let notify = started.clone();

    Mock::given(path("/keys"))
        .respond_with(move |_: &wiremock::Request| {
            notify.notify_one();
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(30))
                .set_body_raw(TEST_JWKS, "application/json")
        })
        .expect(1)
        .mount(&server)
        .await;

    let verifier = new_test_keys(&server);
    let cloned = verifier.clone();
    let task = tokio::spawn(async move { cloned.key_for("test-kid").await });

    timeout(Duration::from_secs(2), started.notified())
        .await
        .unwrap();

    task.abort();

    assert!(matches!(task.await, Err(error) if error.is_cancelled()));

    let result = timeout(Duration::from_secs(1), verifier.key_for("test-kid"))
        .await
        .unwrap();

    assert!(matches!(result, Err(JwksError::Unavailable)));
}

/// Um refresh malsucedido não impede leitura de uma chave antiga ainda válida.
#[tokio::test]
async fn failed_refresh_preserves_valid_known_keys() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let verifier = new_test_keys(&server);

    verifier.key_for("test-kid").await.unwrap();

    server.verify().await;

    server.reset().await;

    Mock::given(path("/keys"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;

    verifier.cache.write().await.refresh_after = Instant::now();

    assert!(matches!(
        verifier.key_for("rotated").await,
        Err(JwksError::Unavailable)
    ));

    assert!(verifier.key_for("test-kid").await.is_ok());
}

/// Todos os leitores de uma origem fria aguardam a mesma busca bem-sucedida.
#[tokio::test]
async fn concurrent_clones_wait_for_one_initial_fetch() {
    let server = MockServer::start().await;

    mock_jwks(&server, 300, 1).await;

    let keys = new_test_keys(&server);
    let mut tasks = Vec::new();

    for _ in 0..12 {
        let clone = keys.clone();

        tasks.push(tokio::spawn(async move { clone.key_for("test-kid").await }));
    }

    for task in tasks {
        assert!(task.await.unwrap().is_ok());
    }

    server.verify().await;
}

/// Fontes separadas nunca reutilizam chaves da outra instância.
#[tokio::test]
async fn separate_caches_do_not_share_trusted_keys() {
    let first = MockServer::start().await;
    let second = MockServer::start().await;

    mock_jwks(&first, 300, 1).await;

    let mut other: Value = serde_json::from_str(TEST_JWKS).unwrap();

    other["keys"][0]["kid"] = json!("other-kid");

    Mock::given(path("/keys"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("cache-control", "max-age=300")
                .set_body_json(other),
        )
        .expect(1)
        .mount(&second)
        .await;

    let first_keys = new_test_keys(&first);
    let second_keys = new_test_keys(&second);

    first_keys.key_for("test-kid").await.unwrap();

    assert!(matches!(
        second_keys.key_for("test-kid").await,
        Err(JwksError::UnknownKey)
    ));

    second_keys.key_for("other-kid").await.unwrap();

    assert!(matches!(
        first_keys.key_for("other-kid").await,
        Err(JwksError::UnknownKey)
    ));

    first.verify().await;
    second.verify().await;
}

/// O mesmo parser respeita as políticas explícitas de Auth e App Check.
#[test]
fn fallback_and_maximum_ttl_are_explicit_policy() {
    let maximum = Duration::from_secs(6 * 60 * 60);
    let bounded = CachePolicy {
        fallback_ttl: maximum,
        maximum_ttl: Some(maximum),
    };
    let mut headers = HeaderMap::new();

    assert_eq!(cache_ttl(&headers, HTTP_POLICY), Duration::ZERO);

    assert_eq!(cache_ttl(&headers, bounded), maximum);

    headers.insert(CACHE_CONTROL, "max-age=999999".parse().unwrap());

    assert_eq!(cache_ttl(&headers, bounded), maximum);

    headers.insert(AGE, "60".parse().unwrap());

    assert_eq!(
        cache_ttl(&headers, bounded),
        maximum - Duration::from_secs(60)
    );

    headers.insert(CACHE_CONTROL, "max-age=120".parse().unwrap());

    assert_eq!(cache_ttl(&headers, bounded), Duration::from_secs(60));

    headers.append(CACHE_CONTROL, "no-store".parse().unwrap());

    assert_eq!(cache_ttl(&headers, bounded), Duration::ZERO);
}

/// Diretivas em cabeçalhos separados também restringem o prazo de reutilização.
#[test]
fn multiple_cache_control_headers_use_the_shortest_valid_ttl() {
    let mut headers = HeaderMap::new();

    headers.insert(CACHE_CONTROL, "max-age=300".parse().unwrap());
    headers.append(CACHE_CONTROL, "MAX-AGE = \"60\"".parse().unwrap());

    assert_eq!(cache_ttl(&headers, HTTP_POLICY), Duration::from_secs(60));

    headers.append(CACHE_CONTROL, "max-age=invalid".parse().unwrap());

    assert_eq!(cache_ttl(&headers, HTTP_POLICY), Duration::ZERO);
}
