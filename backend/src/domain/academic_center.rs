//! Reexportação dos tipos e contratos da entidade de domínio Centro Acadêmico.

pub use crate::modules::ca::domain::entities::academic_center::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_access_academic_center_from_domain_reexport() {
        let caer = AcademicCenter::caer_didactic();
        assert_eq!(caer.acronym, "CAER");
        assert_eq!(caer.id, CAER_DIDACTIC_ID);

        let cacc = AcademicCenter::cacc_didactic();
        assert_eq!(cacc.acronym, "CACC");
        assert_eq!(cacc.id, CACC_DIDACTIC_ID);
    }
}
