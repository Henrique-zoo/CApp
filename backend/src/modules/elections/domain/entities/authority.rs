//! Entidade que representa uma autoridade participante de uma eleição.

use crate::modules::elections::domain::value_objects::{
    base64_url::Base64Url,
    big_uint::BigUInt,
    identifiers::{AuthorityId, SlateId},
    key_proof::KeyProof,
};

/// Papel desempenhado por uma autoridade no protocolo eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorityRole {
    /// Autoridade responsável pelos registros assinados pelo servidor.
    Server,

    /// Autoridade da comissão eleitoral.
    Commission,

    /// Autoridade responsável pela chave de uma chapa.
    Slate,
}

/// Autoridade registrada na configuração de uma eleição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authority {
    id: AuthorityId,
    role: AuthorityRole,
    slate_id: Option<SlateId>,
    signing_key: Base64Url,
    public_value: BigUInt,
    key_proof: KeyProof,
}

/// Erros encontrados ao construir uma autoridade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidAuthority {
    /// O papel Slate exige um identificador de chapa.
    #[error("Uma autoridade de chapa exige um slate_id")]
    MissingSlateId,

    /// Autoridades Server e Commission não podem apontar para uma chapa.
    #[error("Somente uma autoridade de chapa pode possuir slate_id")]
    UnexpectedSlateId,

    /// O valor público 1 é proibido pelo perfil eleitoral.
    #[error("O valor público da autoridade não pode ser 1")]
    InvalidPublicValue,

    /// O compromisso da prova de conhecimento não pode ser 1.
    #[error("O compromisso da prova de conhecimento não pode ser 1")]
    InvalidKeyProofCommitment,
}

impl Authority {
    /// Constrói uma autoridade verificando suas invariantes individuais.
    pub fn try_new(
        id: AuthorityId,
        role: AuthorityRole,
        slate_id: Option<SlateId>,
        signing_key: Base64Url,
        public_value: BigUInt,
        key_proof: KeyProof,
    ) -> Result<Self, InvalidAuthority> {
        match (role, slate_id) {
            (AuthorityRole::Slate, None) => {
                return Err(InvalidAuthority::MissingSlateId);
            }
            (AuthorityRole::Server | AuthorityRole::Commission, Some(_)) => {
                return Err(InvalidAuthority::UnexpectedSlateId);
            }
            _ => {}
        }

        if public_value.as_str() == "1" {
            return Err(InvalidAuthority::InvalidPublicValue);
        }

        if key_proof.commitment().as_str() == "1" {
            return Err(InvalidAuthority::InvalidKeyProofCommitment);
        }

        Ok(Self {
            id,
            role,
            slate_id,
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
        self.slate_id
    }

    /// Retorna a chave pública de assinatura codificada.
    pub fn signing_key(&self) -> &Base64Url {
        &self.signing_key
    }

    /// Retorna o valor público do protocolo.
    pub const fn public_value(&self) -> &BigUInt {
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
    use crate::modules::elections::domain::value_objects::identifiers::SlateId;
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use uuid::Uuid;

    fn encoded_key(value: &str) -> String {
        URL_SAFE_NO_PAD.encode(value.as_bytes())
    }

    fn authority_id() -> AuthorityId {
        AuthorityId::try_from_uuid(Uuid::new_v4()).unwrap()
    }

    fn slate_id() -> SlateId {
        SlateId::try_from_uuid(Uuid::new_v4()).unwrap()
    }

    fn signing_key() -> Base64Url {
        Base64Url::try_new(encoded_key("public-signing-key")).unwrap()
    }

    fn public_value() -> BigUInt {
        BigUInt::try_from_decimal("123").unwrap()
    }

    fn key_proof() -> KeyProof {
        KeyProof::new(
            BigUInt::try_from_decimal("456").unwrap(),
            BigUInt::try_from_decimal("789").unwrap(),
        )
    }

    #[test]
    fn creates_server_authority_without_slate() {
        let authority = Authority::try_new(
            authority_id(),
            AuthorityRole::Server,
            None,
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
            None,
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
            AuthorityRole::Slate,
            Some(slate_id),
            signing_key(),
            public_value(),
            key_proof(),
        )
        .unwrap();

        assert_eq!(authority.role(), AuthorityRole::Slate);
        assert_eq!(authority.slate_id(), Some(slate_id));
    }

    #[test]
    fn rejects_slate_authority_without_slate_id() {
        assert_eq!(
            Authority::try_new(
                authority_id(),
                AuthorityRole::Slate,
                None,
                signing_key(),
                public_value(),
                key_proof(),
            ),
            Err(InvalidAuthority::MissingSlateId)
        );
    }

    #[test]
    fn rejects_server_authority_with_slate_id() {
        assert_eq!(
            Authority::try_new(
                authority_id(),
                AuthorityRole::Server,
                Some(slate_id()),
                signing_key(),
                public_value(),
                key_proof(),
            ),
            Err(InvalidAuthority::UnexpectedSlateId)
        );
    }

    #[test]
    fn rejects_commission_authority_with_slate_id() {
        assert_eq!(
            Authority::try_new(
                authority_id(),
                AuthorityRole::Commission,
                Some(slate_id()),
                signing_key(),
                public_value(),
                key_proof(),
            ),
            Err(InvalidAuthority::UnexpectedSlateId)
        );
    }

    #[test]
    fn rejects_key_proof_commitment_equal_to_one() {
        let invalid_proof = KeyProof::new(
            BigUInt::try_from_decimal("1").unwrap(),
            BigUInt::try_from_decimal("789").unwrap(),
        );

        assert_eq!(
            Authority::try_new(
                authority_id(),
                AuthorityRole::Server,
                None,
                signing_key(),
                public_value(),
                invalid_proof,
            ),
            Err(InvalidAuthority::InvalidKeyProofCommitment)
        );
    }

    #[test]
    fn rejects_public_value_equal_to_one() {
        assert_eq!(
            Authority::try_new(
                authority_id(),
                AuthorityRole::Server,
                None,
                signing_key(),
                BigUInt::try_from_decimal("1").unwrap(),
                key_proof(),
            ),
            Err(InvalidAuthority::InvalidPublicValue)
        );
    }
}
