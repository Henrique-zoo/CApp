//! Identificadores tipados do domínio eleitoral.
//!
//! Cada tipo evita misturar identificadores de conceitos diferentes, mesmo
//! quando todos são representados internamente por UUID.
//!
//! A criação não gera UUIDs: a origem e a geração do identificador pertencem
//! ao chamador. O domínio apenas verifica as invariantes de cada tipo.

use uuid::Uuid;

use crate::modules::elections::domain::error::InvalidIdentifier;

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
non_null_identifier!(SlateId);
non_null_identifier!(AccountId);

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
    fn account_id_rejects_nil_uuid() {
        assert!(AccountId::try_from_uuid(Uuid::nil()).is_err());
    }

    #[test]
    fn authority_id_rejects_nil_uuid() {
        assert!(AuthorityId::try_from_uuid(Uuid::nil()).is_err());
    }

    #[test]
    fn slate_id_rejects_the_uuid_reserved_for_the_null_vote() {
        assert!(SlateId::try_from_uuid(Uuid::nil()).is_err());
    }

    #[test]
    fn different_identifier_types_are_distinct_types() {
        let uuid = Uuid::new_v4();

        let election_id = ElectionId::try_from_uuid(uuid).unwrap();
        let authority_id = AuthorityId::try_from_uuid(uuid).unwrap();

        assert_eq!(election_id.into_uuid(), authority_id.into_uuid());
    }
}
