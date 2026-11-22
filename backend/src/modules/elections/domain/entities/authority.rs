//! Entidade que representa uma autoridade participante de uma eleição.

use crypto_bigint::U2048;

use crate::modules::elections::domain::{
    error::InvalidAuthority,
    identifiers::{AuthorityId, SlateId},
};

/// Papel desempenhado por uma autoridade no protocolo eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorityRole {
    /// Autoridade ElGamal do backend; distinta da assinatura RSA dos registros.
    Server,

    /// Autoridade da comissão eleitoral.
    Commission,

    /// Autoridade responsável pela chave de uma chapa.
    Slate(SlateId),
}

/// Prova de conhecimento do segredo associado ao valor público ElGamal.
///
/// A autoridade fornece o compromisso e a resposta para verificação
/// criptográfica antes da publicação do manifesto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyProof {
    commitment: U2048,
    response: U2048,
}

impl KeyProof {
    /// Reúne o compromisso e a resposta numéricos para verificação.
    pub fn new(commitment: U2048, response: U2048) -> Self {
        Self {
            commitment,
            response,
        }
    }

    /// Retorna o compromisso da prova.
    pub const fn commitment(&self) -> &U2048 {
        &self.commitment
    }

    /// Retorna a resposta da prova.
    pub const fn response(&self) -> &U2048 {
        &self.response
    }
}

/// Autoridade registrada na configuração de uma eleição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authority {
    id: AuthorityId,
    role: AuthorityRole,
    signing_key: Vec<u8>,
    public_value: U2048,
    key_proof: KeyProof,
}

impl Authority {
    /// Constrói uma autoridade verificando suas invariantes individuais.
    pub fn try_new(
        id: AuthorityId,
        role: AuthorityRole,
        signing_key: Vec<u8>,
        public_value: U2048,
        key_proof: KeyProof,
    ) -> Result<Self, InvalidAuthority> {
        if public_value <= U2048::ONE {
            return Err(InvalidAuthority::InvalidPublicValue);
        }

        if key_proof.commitment() <= &U2048::ONE {
            return Err(InvalidAuthority::InvalidKeyProofCommitment);
        }

        Ok(Self {
            id,
            role,
            signing_key,
            public_value,
            key_proof,
        })
    }

    /// Retorna o identificador da autoridade.
    pub const fn id(&self) -> AuthorityId {
        self.id
    }

    /// Retorna o papel da autoridade.
    pub const fn role(&self) -> AuthorityRole {
        self.role
    }

    /// Retorna o identificador da chapa, quando aplicável.
    pub const fn slate_id(&self) -> Option<SlateId> {
        match self.role {
            AuthorityRole::Slate(id) => Some(id),
            AuthorityRole::Server | AuthorityRole::Commission => None,
        }
    }

    /// Retorna os bytes DER da chave pública de assinatura.
    pub fn signing_key(&self) -> &[u8] {
        self.signing_key.as_slice()
    }

    /// Retorna o valor público ElGamal usado para verificar a prova e combinar as chaves.
    pub const fn public_value(&self) -> &U2048 {
        &self.public_value
    }

    /// Retorna a prova de conhecimento da chave.
    pub const fn key_proof(&self) -> &KeyProof {
        &self.key_proof
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uuid::Uuid;

    fn authority_id() -> AuthorityId {
        AuthorityId::try_from_uuid(Uuid::new_v4()).unwrap()
    }

    fn slate_id() -> SlateId {
        SlateId::try_from_uuid(Uuid::new_v4()).unwrap()
    }

    fn signing_key() -> Vec<u8> {
        "public-signing-key".as_bytes().to_vec()
    }

    fn public_value() -> U2048 {
        U2048::from_u32(123)
    }

    fn key_proof() -> KeyProof {
        KeyProof::new(U2048::from_u32(456), U2048::from_u32(789))
    }

    #[test]
    fn creates_server_authority_without_slate() {
        let authority = Authority::try_new(
            authority_id(),
            AuthorityRole::Server,
            signing_key(),
            public_value(),
            key_proof(),
        )
        .unwrap();

        assert_eq!(authority.role(), AuthorityRole::Server);
        assert_eq!(authority.slate_id(), None);
    }

    #[test]
    fn creates_commission_authority_without_slate() {
        let authority = Authority::try_new(
            authority_id(),
            AuthorityRole::Commission,
            signing_key(),
            public_value(),
            key_proof(),
        )
        .unwrap();

        assert_eq!(authority.role(), AuthorityRole::Commission);
        assert_eq!(authority.slate_id(), None);
    }

    #[test]
    fn creates_slate_authority_with_slate_id() {
        let slate_id = slate_id();

        let authority = Authority::try_new(
            authority_id(),
            AuthorityRole::Slate(slate_id),
            signing_key(),
            public_value(),
            key_proof(),
        )
        .unwrap();

        assert_eq!(authority.role(), AuthorityRole::Slate(slate_id));
        assert_eq!(authority.slate_id(), Some(slate_id));
    }

    #[test]
    fn rejects_key_proof_commitment_equal_to_one() {
        let invalid_proof = KeyProof::new(U2048::from_u32(1), U2048::from_u32(789));

        assert_eq!(
            Authority::try_new(
                authority_id(),
                AuthorityRole::Server,
                signing_key(),
                public_value(),
                invalid_proof,
            ),
            Err(InvalidAuthority::InvalidKeyProofCommitment)
        );
    }

    #[test]
    fn rejects_public_values_zero_and_one() {
        for public_value in [U2048::ZERO, U2048::ONE] {
            assert_eq!(
                Authority::try_new(
                    authority_id(),
                    AuthorityRole::Server,
                    signing_key(),
                    public_value,
                    key_proof(),
                ),
                Err(InvalidAuthority::InvalidPublicValue)
            );
        }
    }

    #[test]
    fn stores_commitment_and_response() {
        let commitment = U2048::from_u32(123);
        let response = U2048::from_u32(456);

        let proof = KeyProof::new(commitment, response);

        assert_eq!(proof.commitment(), &commitment);
        assert_eq!(proof.response(), &response);
    }

    #[test]
    fn accepts_zero_values_structurally() {
        // O grupo é considerado na verificação criptográfica da prova.
        let zero = U2048::ZERO;

        let proof = KeyProof::new(zero, zero);

        assert_eq!(proof.commitment(), &zero);
        assert_eq!(proof.response(), &zero);
    }
}
