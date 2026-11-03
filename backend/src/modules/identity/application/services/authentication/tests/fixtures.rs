//! Dados-padrão reaproveitados pelos testes da aplicação.
use crate::modules::identity::domain::CatalogStatus;

use super::*;

pub const FIREBASE_TOKEN: &str = "firebase-token-de-teste";
pub const MICROSOFT_TOKEN: &str = "microsoft-token-de-teste";
pub const FIREBASE_UID: &str = "firebase-user-123";
pub const MICROSOFT_OBJECT_ID: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
const TENANT_ID: &str = "bbbbbbbb-cccc-dddd-eeee-ffffffffffff";
const STUDENT_GROUP_ID: &str = "cccccccc-dddd-eeee-ffff-000000000000";
const INSTITUTION_ID: &str = "dddddddd-eeee-ffff-0000-111111111111";

pub fn microsoft_object_id() -> Uuid {
    Uuid::parse_str(MICROSOFT_OBJECT_ID).unwrap()
}
pub fn tenant_id() -> Uuid {
    Uuid::parse_str(TENANT_ID).unwrap()
}
pub fn student_group_id() -> Uuid {
    Uuid::parse_str(STUDENT_GROUP_ID).unwrap()
}
pub fn institution_id() -> Uuid {
    Uuid::parse_str(INSTITUTION_ID).unwrap()
}

pub fn valid_identity() -> VerifiedIdentity {
    VerifiedIdentity {
        firebase_uid: FIREBASE_UID.into(),
        microsoft_object_id: Some(MICROSOFT_OBJECT_ID.into()),
    }
}

pub fn evidence(confirmed: bool, validity: Duration) -> InstitutionalVerification {
    let checked_at = Utc::now();
    InstitutionalVerification {
        firebase_uid: FIREBASE_UID.into(),
        microsoft_object_id: microsoft_object_id(),
        institution_id: institution_id(),
        microsoft_tenant_id: tenant_id(),
        microsoft_student_group_id: student_group_id(),
        student_membership_confirmed: confirmed,
        checked_at,
        expires_at: checked_at + validity,
    }
}

/// Cria uma instituição fictícia com os identificadores usados pelos testes.
pub fn institution() -> Institution {
    Institution {
        id: institution_id(),
        name: "Intituição A".to_string(),
        acronym: None,
        microsoft_tenant_id: tenant_id(),
        microsoft_student_group_id: student_group_id(),
        status: CatalogStatus::default(),
    }
}

/// Cria um perfil fictício com o vínculo indicado e preserva o ID, mesmo se inválido.
///
/// Somente estudantes possuem a informação de curso na enum de vínculo.
pub fn profile_with_id(id: &str, is_student: bool) -> InstitutionalProfile {
    InstitutionalProfile {
        id: id.to_owned(),
        name: Some("Estudante de Teste".to_owned()),
        mail: Some("estudante@instituicao.example".to_owned()),
        affiliation: if is_student {
            InstitutionalAffiliation::Student {
                course_name: Some("Curso de Teste".to_owned()),
            }
        } else {
            InstitutionalAffiliation::NonStudent
        },
    }
}
