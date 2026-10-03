//! Verificação de prontidão usando o World e o router reais do backend.
//!
//! A preparação é responsabilidade do hook `before`. Estes steps confirmam
//! acesso SQL, realizam requisições em memória e verificam o status preservado
//! no World; não iniciam um servidor HTTP nem usam o banco de desenvolvimento.

use cucumber::{given, then, when};

use crate::support::AppWorld;

/// Confirma que o pool do cenário consegue executar `SELECT 1`.
///
/// # Parâmetros
///
/// - `world`: contexto previamente preparado pelo hook `before`.
///
/// # Panics
///
/// Se o banco não estiver associado ao World, a consulta falhar ou o resultado
/// for diferente de `1`.
#[given("que o PostgreSQL está disponível")]
async fn postgres_is_available(world: &mut AppWorld) {
    let result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(world.database_pool())
        .await
        .expect("o PostgreSQL do cenário deve aceitar consultas");

    assert_eq!(result, 1);
}

/// Consulta a rota informada e preserva a resposta HTTP no World.
///
/// O caminho pode conter query string. Delega a execução em memória a
/// [`AppWorld::get`]; um status de erro HTTP é capturado para a asserção seguinte.
///
/// # Panics
///
/// Nas condições de falha de preparação, URI e leitura da resposta descritas
/// em [`AppWorld::get`].
#[when(expr = "consulto a rota {string}")]
async fn get_route(world: &mut AppWorld, path: String) {
    world.get(&path).await;
}

/// Compara o status da última resposta com o código esperado pelo cenário.
///
/// # Parâmetros
///
/// - `world`: contexto que deve conter uma resposta anterior.
/// - `expected`: código HTTP numérico informado no passo Gherkin.
///
/// # Panics
///
/// Se nenhuma resposta tiver sido capturada ou o status diferir de `expected`.
/// Nesse último caso, inclui o corpo da resposta no diagnóstico da asserção.
#[then(expr = "o status da resposta deve ser {int}")]
fn response_status_is(world: &mut AppWorld, expected: u16) {
    let response = world
        .last_response
        .as_ref()
        .expect("uma requisição deve ter sido realizada antes da verificação");

    assert_eq!(
        response.status.as_u16(),
        expected,
        "corpo da resposta: {}",
        response.body,
    );
}
