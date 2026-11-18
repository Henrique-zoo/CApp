//! Prova de conhecimento de chave pública de uma autoridade.

use crate::modules::elections::domain::value_objects::big_uint::BigUInt;

/// Prova de conhecimento de uma chave secreta.
///
/// A verificação matemática da prova pertence à camada criptográfica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyProof {
    commitment: BigUInt,
    response: BigUInt,
}

impl KeyProof {
    /// Constrói uma prova a partir de seus valores decimais validados.
    pub fn new(commitment: BigUInt, response: BigUInt) -> Self {
        Self {
            commitment,
            response,
        }
    }

    /// Retorna o compromisso da prova.
    pub const fn commitment(&self) -> &BigUInt {
        &self.commitment
    }

    /// Retorna a resposta da prova.
    pub const fn response(&self) -> &BigUInt {
        &self.response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_commitment_and_response() {
        let commitment = BigUInt::try_from_decimal("123").unwrap();
        let response = BigUInt::try_from_decimal("456").unwrap();

        let proof = KeyProof::new(commitment.clone(), response.clone());

        assert_eq!(proof.commitment(), &commitment);
        assert_eq!(proof.response(), &response);
    }

    #[test]
    fn accepts_zero_values_structurally() {
        // Os limites matemáticos dependem do grupo e são verificados
        // pela camada criptográfica, não por este objeto de valor.
        let zero = BigUInt::try_from_decimal("0").unwrap();

        let proof = KeyProof::new(zero.clone(), zero.clone());

        assert_eq!(proof.commitment(), &zero);
        assert_eq!(proof.response(), &zero);
    }
}
