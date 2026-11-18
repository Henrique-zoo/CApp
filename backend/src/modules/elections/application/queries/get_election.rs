//! Consulta um agregado eleitoral pelo seu identificador.

use crate::modules::elections::{
    application::{error::GetElectionError, ports::ElectionReader},
    domain::{entities::election::Election, value_objects::identifiers::ElectionId},
};

/// Caso de uso para recuperar uma eleição.
pub struct GetElection<'a> {
    reader: &'a dyn ElectionReader,
}

impl<'a> GetElection<'a> {
    /// Cria a consulta com a porta de leitura eleitoral.
    pub const fn new(reader: &'a dyn ElectionReader) -> Self {
        Self { reader }
    }

    /// Recupera a eleição ou retorna um erro tipado.
    ///
    /// A consulta não altera o agregado nem realiza decisões de autorização.
    /// Essas decisões pertencem à política de acesso da aplicação/API.
    pub async fn execute(&self, election_id: ElectionId) -> Result<Election, GetElectionError> {
        let versioned = self
            .reader
            .find_by_id(election_id)
            .await?
            .ok_or(GetElectionError::NotFound)?;

        Ok(versioned.into_election())
    }
}

#[cfg(test)]
#[path = "get_election/tests.rs"]
mod tests;
