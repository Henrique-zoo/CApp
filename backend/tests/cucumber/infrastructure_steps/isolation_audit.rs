//! Verificação da separação de dados entre cenários com bancos isolados.
//!
//! Cada cenário cria a mesma tabela e grava sua própria linha. Como a criação
//! não usa `IF NOT EXISTS`, compartilhar a base acidentalmente causa falha.
//! A consulta posterior verifica que somente o marcador do cenário está presente.
//! Os cenários devem usar a tag `@isolated_database`.

use cucumber::{then, when};

use crate::support::AppWorld;

/// Cria a tabela de verificação e grava o marcador na base do cenário.
///
/// Confere o prefixo do nome da base antes de executar DDL. O marcador é
/// vinculado como parâmetro SQL. A tabela não é removida pelo step: o hook
/// `after` deve descartar o banco isolado inteiro.
///
/// # Parâmetros
///
/// - `world`: contexto associado a uma base com `@isolated_database`.
/// - `marker`: valor que deverá aparecer somente neste cenário.
///
/// # Panics
///
/// Se o World não tiver banco, o nome não possuir o prefixo das bases isoladas
/// ou qualquer consulta falhar, inclusive por a tabela já existir.
#[when(expr = "gravo o marcador {string} no banco do cenário")]
async fn write_marker(world: &mut AppWorld, marker: String) {
    let pool = world.database_pool();
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(database.starts_with("capp_cucumber_scenario_"));

    // O nome é o mesmo nos dois cenários, sem IF NOT EXISTS.
    // Compartilhar acidentalmente a mesma base fará esta criação falhar.
    sqlx::query("CREATE TABLE bdd_isolation_probe (id INTEGER PRIMARY KEY, valor TEXT NOT NULL)")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO bdd_isolation_probe (id, valor) VALUES (1, $1)")
        .bind(marker)
        .execute(pool)
        .await
        .unwrap();
}

/// Verifica que a tabela contém exatamente o marcador esperado.
///
/// A consulta lê todos os valores ordenados por `id`. A comparação rejeita
/// linhas ausentes, adicionais ou com conteúdo de outro cenário.
///
/// # Panics
///
/// Se o banco não estiver associado, a consulta falhar ou o conjunto de valores
/// diferir de uma única linha contendo `expected`.
#[then(expr = "somente o marcador {string} está presente neste cenário")]
async fn check_marker(world: &mut AppWorld, expected: String) {
    let values: Vec<String> =
        sqlx::query_scalar("SELECT valor FROM bdd_isolation_probe ORDER BY id")
            .fetch_all(world.database_pool())
            .await
            .unwrap();
    assert_eq!(values, vec![expected]);
}
