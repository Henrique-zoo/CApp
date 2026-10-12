//! Reexportação dos tipos e contratos da entidade de domínio Centro Acadêmico (`academic_center`).

pub use crate::modules::ca::domain::entities::academic_center::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_access_academic_center_from_domain_reexport() {
        let caer = AcademicCenter::caer_didactic();
        assert_eq!(caer.id, CAER_DIDACTIC_ID);
        assert_eq!(caer.acronym, "CAER");
    }
}
