//! Reexportação dos tipos e contratos da entidade de domínio Centro Acadêmico.

pub use crate::modules::ca::domain::*;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    const CAER_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0001);
    const CACC_DIDACTIC_ID: Uuid = Uuid::from_u128(0x018f_0000_0000_7000_8000_0000_0000_0002);

    #[test]
    fn should_access_academic_center_from_domain_reexport() {
        let now = Utc::now();
        let caer = AcademicCenter::new(
            CAER_DIDACTIC_ID,
            None,
            "Centro Acadêmico de Engenharia de Redes".to_string(),
            "CAER".to_string(),
            "Descrição do CAER".to_string(),
            AcademicCenterStatus::Active,
            now,
            now,
        )
        .expect("should instantiate caer");

        assert_eq!(caer.acronym, "CAER");
        assert_eq!(caer.id, CAER_DIDACTIC_ID);

        let cacc = AcademicCenter::new(
            CACC_DIDACTIC_ID,
            None,
            "Centro Acadêmico de Ciência da Computação".to_string(),
            "CACC".to_string(),
            "Descrição do CACC".to_string(),
            AcademicCenterStatus::Active,
            now,
            now,
        )
        .expect("should instantiate cacc");

        assert_eq!(cacc.acronym, "CACC");
        assert_eq!(cacc.id, CACC_DIDACTIC_ID);
    }

    #[test]
    fn should_validate_academic_center_from_domain_reexport() {
        let now = Utc::now();
        let result = AcademicCenter::new(
            Uuid::new_v4(),
            None,
            "".to_string(),
            "CA".to_string(),
            "Desc".to_string(),
            AcademicCenterStatus::Draft,
            now,
            now,
        );

        assert_eq!(result.unwrap_err(), AcademicCenterError::EmptyName);
    }

    #[test]
    fn should_handle_domain_status_from_reexport() {
        assert_eq!(
            "active".parse::<AcademicCenterStatus>().unwrap(),
            AcademicCenterStatus::Active
        );
        assert_eq!(AcademicCenterStatus::Draft.as_str(), "draft");
        assert_eq!(AcademicCenterStatus::Draft.to_string(), "draft");
    }

    #[test]
    fn should_handle_domain_error_display_from_reexport() {
        let err = AcademicCenterError::EmptyAcronym;
        assert_eq!(
            err.to_string(),
            "a sigla do Centro Acadêmico não pode ser vazia"
        );
    }
}
