//! Identificadores tipados do domínio eleitoral.
//!
//! Cada tipo evita misturar identificadores de conceitos diferentes, mesmo
//! quando todos são representados internamente por UUID.
//!
//! A criação não gera UUIDs: a origem e a geração do identificador pertencem
//! ao chamador. O domínio apenas verifica as invariantes de cada tipo.

use uuid::Uuid;

use crate::modules::elections::domain::errors::InvalidIdentifier;

/// Implementa um identificador UUID que não pode ser nulo.
macro_rules! non_null_identifier {
    ($name:ident) => {
        #[doc = concat!("Identificador tipado de domínio para ", stringify!($name), ".")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(Uuid);

        impl $name {
            /// Constrói o identificador se o UUID não for nulo.
            pub fn try_from_uuid(value: Uuid) -> Result<Self, InvalidIdentifier> {
                if value.is_nil() {
                    return Err(InvalidIdentifier::new(stringify!($name)));
                }

                Ok(Self(value))
            }

            /// Retorna o UUID subjacente sem transferir a propriedade.
            pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            /// Consome o identificador e retorna o UUID subjacente.
            pub const fn into_uuid(self) -> Uuid {
                self.0
            }
        }
    };
}

non_null_identifier!(ElectionId);
non_null_identifier!(AcademicCenterId);
non_null_identifier!(AuthorityId);
non_null_identifier!(VoterId);
non_null_identifier!(SubmissionId);
non_null_identifier!(ActorId);
non_null_identifier!(SlateId);

/// Identificador de uma opção eleitoral.
///
/// O UUID zero é reservado exclusivamente para representar a opção Nulo.
/// Os demais valores identificam opções de chapa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OptionId(Uuid);

impl OptionId {
    /// UUID reservado pelo protocolo para a opção Nulo.
    pub const NULL_OPTION_UUID: Uuid = Uuid::nil();

    /// Constrói um identificador de opção, incluindo o UUID reservado para Nulo.
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Retorna `true` quando o identificador representa a opção Nulo.
    pub const fn is_null_option(&self) -> bool {
        self.0.is_nil()
    }

    /// Retorna o UUID subjacente sem transferir a propriedade.
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Consome o identificador e retorna o UUID subjacente.
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn election_id_rejects_nil_uuid() {
        let result = ElectionId::try_from_uuid(Uuid::nil());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "ElectionId");
    }

    #[test]
    fn election_id_accepts_non_nil_uuid() {
        let uuid = Uuid::new_v4();
        let id = ElectionId::try_from_uuid(uuid).unwrap();

        assert_eq!(*id.as_uuid(), uuid);
        assert_eq!(id.into_uuid(), uuid);
    }

    #[test]
    fn academic_center_id_rejects_nil_uuid() {
        assert!(AcademicCenterId::try_from_uuid(Uuid::nil()).is_err());
    }

    #[test]
    fn authority_id_rejects_nil_uuid() {
        assert!(AuthorityId::try_from_uuid(Uuid::nil()).is_err());
    }

    #[test]
    fn voter_id_rejects_nil_uuid() {
        assert!(VoterId::try_from_uuid(Uuid::nil()).is_err());
    }

    #[test]
    fn submission_id_rejects_nil_uuid() {
        assert!(SubmissionId::try_from_uuid(Uuid::nil()).is_err());
    }

    #[test]
    fn actor_id_rejects_nil_uuid() {
        assert!(ActorId::try_from_uuid(Uuid::nil()).is_err());
    }

    #[test]
    fn option_id_accepts_nil_uuid_as_null_option() {
        let id = OptionId::from_uuid(Uuid::nil());

        assert!(id.is_null_option());
        assert_eq!(*id.as_uuid(), OptionId::NULL_OPTION_UUID);
    }

    #[test]
    fn option_id_accepts_non_nil_uuid_as_slate_option() {
        let uuid = Uuid::new_v4();
        let id = OptionId::from_uuid(uuid);

        assert!(!id.is_null_option());
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn different_identifier_types_are_distinct_types() {
        let uuid = Uuid::new_v4();

        let election_id = ElectionId::try_from_uuid(uuid).unwrap();
        let authority_id = AuthorityId::try_from_uuid(uuid).unwrap();

        assert_eq!(election_id.into_uuid(), authority_id.into_uuid());
    }
}
