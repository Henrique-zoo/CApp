//! Resultados HTTP e SQL preservados no World para as asserções dos steps.
//!
//! [`TestResponse`] retém o status e o corpo textual, sem cabeçalhos.
//! [`TestDatabaseMutation`] resume sucesso ou falha de uma operação, permitindo
//! verificar uma rejeição esperada em um step posterior.

use axum::http::StatusCode;

/// Status e corpo UTF-8 da última resposta produzida pelo router real.
#[derive(Debug)]
pub(crate) struct TestResponse {
    /// Código HTTP retornado pelo router.
    pub(crate) status: StatusCode,
    /// Corpo completo da resposta convertido para UTF-8.
    pub(crate) body: String,
}

/// Resultado resumido de uma mutação para verificar constraints e concorrência.
///
/// Preserva SQLSTATE e o nome da constraint quando o PostgreSQL os fornece.
/// Não mantém o valor de sucesso nem a mensagem completa do erro. Os steps
/// podem registrar resultados sequenciais ou concorrentes e verificá-los depois.
#[derive(Debug)]
pub(crate) struct TestDatabaseMutation {
    /// Indica se a operação ou transação retornou sucesso.
    pub(crate) succeded: bool,
    /// SQLSTATE informado pelo PostgreSQL, quando disponível.
    pub(crate) code: Option<String>,
    /// Nome da constraint informada pelo PostgreSQL, quando disponível.
    pub(crate) constraint: Option<String>,
}

impl TestDatabaseMutation {
    /// Resume o resultado SQLx sem transformar uma rejeição esperada em panic.
    ///
    /// # Parâmetros
    ///
    /// - `result`: resultado da consulta ou do commit de uma transação, já executados.
    ///   Esta função não executa SQL nem realiza rollback.
    ///
    /// # Retorno
    ///
    /// `succeded = true` para `Ok`, descartando o valor de sucesso. Para `Err`,
    /// registra `false` e copia SQLSTATE e constraint quando disponíveis. Erros de
    /// conexão ou do cliente SQLx podem não fornecer esses dois campos.
    pub(crate) fn from_result<T>(result: Result<T, sqlx::Error>) -> Self {
        match result {
            Ok(_) => Self {
                succeded: true,
                code: None,
                constraint: None,
            },
            Err(error) => {
                let database_error = error.as_database_error();

                Self {
                    succeded: false,
                    code: database_error
                        .and_then(|error| error.code())
                        .map(|code| code.into_owned()),
                    constraint: database_error
                        .and_then(|error| error.constraint())
                        .map(str::to_owned),
                }
            }
        }
    }
}
