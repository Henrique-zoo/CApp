//! Caso de uso: criação de demandas nos fóruns institucional e de espaço físico.
//!
//! Coordena a validação de autorização, regras específicas de cada fórum,
//! construção da entidade de domínio e persistência transacional.

use chrono::Utc;
use uuid::Uuid;

use crate::modules::demands::{
    application::{
        commands::CreateDemandCommand,
        error::DemandApplicationError,
        ports::{AcademicAffiliationPort, DemandRepository},
    },
    domain::entities::{Demand, DemandError, DemandForum},
};

/// Serviço para criação de demandas acadêmicas e de espaço físico.
#[derive(Debug, Clone)]
pub struct CreateDemandUseCase<R, A> {
    demand_repository: R,
    affiliation_port: A,
}

impl<R, A> CreateDemandUseCase<R, A>
where
    R: DemandRepository,
    A: AcademicAffiliationPort,
{
    /// Inicializa o caso de uso com suas portas dependentes.
    pub fn new(demand_repository: R, affiliation_port: A) -> Self {
        Self {
            demand_repository,
            affiliation_port,
        }
    }

    /// Executa o caso de uso de criação de demandas.
    ///
    /// # Regras de Negócio e Autorização
    ///
    /// 1. O Centro Acadêmico deve existir e estar ativo.
    /// 2. O autor deve possuir vínculo estudantil ativo na instituição.
    /// 3. Fórum do Espaço Físico:
    ///    - Aberto a todos os estudantes ativos da instituição no contexto do CA.
    ///    - É estritamente proibido associar a um curso específico.
    ///    - Inicia com status `Open`.
    /// 4. Fórum Institucional:
    ///    - Exige a indicação de um curso válido.
    ///    - O curso indicado deve ser oficialmente representado pelo Centro Acadêmico.
    ///    - O acesso é restrito aos estudantes matriculados naquele curso e aos membros da gestão do CA.
    /// 5. Todas as invariantes estruturais (limites de caracteres, não vazio) são validadas pela entidade [`Demand`].
    /// 6. A persistência da demanda e do registro inicial no histórico de status é realizada de forma atômica.
    pub async fn execute(
        &self,
        command: CreateDemandCommand,
    ) -> Result<Demand, DemandApplicationError> {
        let affiliation = self
            .affiliation_port
            .get_affiliation(command.author_id, command.academic_center_id)
            .await?;

        // 1. O Centro Acadêmico alvo deve existir
        if !affiliation.academic_center_exists {
            return Err(DemandApplicationError::AcademicCenterNotFound(
                command.academic_center_id,
            ));
        }

        // 2. O Centro Acadêmico deve estar ativo
        if !affiliation.academic_center_active {
            return Err(DemandApplicationError::AcademicCenterInactive(
                command.academic_center_id,
            ));
        }

        // 3. O autor autenticado deve possuir vínculo estudantil ativo
        if !affiliation.has_active_student_membership {
            return Err(DemandApplicationError::StudentMembershipRequired);
        }

        // 4. Regras específicas de cada fórum
        match command.forum {
            DemandForum::PhysicalSpace => {
                // Demandas de espaço físico não admitem curso
                if command.course_name.is_some() {
                    return Err(DemandApplicationError::InvalidDemand(
                        DemandError::CourseNotAllowedForPhysicalSpace,
                    ));
                }
                // Aberto a todos os usuários com vínculo estudantil ativo no app
            }
            DemandForum::Institutional => {
                // Demandas institucionais exigem associação válida a um curso
                let course = match command.course_name.as_deref() {
                    Some(c) if !c.trim().is_empty() => c.trim(),
                    _ => return Err(DemandApplicationError::CourseNameRequired),
                };

                // O CA deve representar o curso alvo
                if !affiliation.ca_represents_course(course) {
                    return Err(DemandApplicationError::CourseNotRepresentedByCa {
                        ca_id: command.academic_center_id,
                        course_name: course.to_string(),
                    });
                }

                // O autor deve pertencer ao curso ou integrar a gestão do CA
                let is_student_of_course = match affiliation.student_course.as_deref() {
                    Some(sc) => sc.trim().eq_ignore_ascii_case(course),
                    None => false,
                };

                if !affiliation.is_ca_manager && !is_student_of_course {
                    return Err(DemandApplicationError::UserNotPermittedInCa);
                }
            }
        }

        // 5. Instancia a entidade de domínio e valida suas invariantes
        let demand = Demand::new(
            Uuid::new_v4(),
            command.academic_center_id,
            command.author_id,
            command.forum,
            command.title,
            command.description,
            command.category,
            command.course_name,
            Utc::now(),
        )?;

        // 6. Persistência atômica no repositório
        self.demand_repository.create(&demand).await?;

        Ok(demand)
    }
}

/// Função utilitária para execução direta do caso de uso de criação de demanda.
pub async fn create_demand<R, A>(
    demand_repository: &R,
    affiliation_port: &A,
    command: CreateDemandCommand,
) -> Result<Demand, DemandApplicationError>
where
    R: DemandRepository,
    A: AcademicAffiliationPort,
{
    let use_case = CreateDemandUseCase::new(demand_repository, affiliation_port);
    use_case.execute(command).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::demands::{
        application::ports::AcademicAffiliation,
        domain::{entities::DemandStatus, rules::MAX_DESCRIPTION_LENGTH},
        infrastructure::{InMemoryAcademicAffiliationPort, InMemoryDemandRepository},
    };

    fn setup() -> (
        InMemoryDemandRepository,
        InMemoryAcademicAffiliationPort,
        Uuid,
        Uuid,
    ) {
        let repo = InMemoryDemandRepository::new();
        let port = InMemoryAcademicAffiliationPort::new();
        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        (repo, port, ca_id, author_id)
    }

    #[tokio::test]
    async fn should_create_physical_space_demand_successfully_for_any_active_student() {
        let (repo, port, ca_id, author_id) = setup();

        // Estudante de outro curso (Engenharia de Software), sem ser gestor do CA (que é de Computação)
        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Engenharia de Software".to_string()),
                vec!["Ciência da Computação".to_string()],
                false,
            ),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            Some("Reparo da porta".to_string()),
            "A porta da sala do CA está danificada".to_string(),
            Some("Infraestrutura".to_string()),
            None,
        );

        let result = create_demand(&repo, &port, command)
            .await
            .expect("deve criar demanda sobre o espaço físico");

        assert_eq!(result.academic_center_id(), ca_id);
        assert_eq!(result.author_id(), author_id);
        assert_eq!(result.forum(), DemandForum::PhysicalSpace);
        assert_eq!(result.title(), Some("Reparo da porta"));
        assert_eq!(
            result.description(),
            "A porta da sala do CA está danificada"
        );
        assert_eq!(result.category(), Some("Infraestrutura"));
        assert_eq!(result.course_name(), None);
        assert_eq!(result.status(), DemandStatus::Open);
        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn should_create_institutional_demand_successfully_for_student_of_the_course() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Ciência da Computação".to_string()),
                vec!["Ciência da Computação".to_string()],
                false,
            ),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::Institutional,
            Some("Ementa desatualizada".to_string()),
            "A ementa da matéria X está desatualizada".to_string(),
            Some("Acadêmico".to_string()),
            Some("Ciência da Computação".to_string()),
        );

        let result = create_demand(&repo, &port, command)
            .await
            .expect("deve criar demanda institucional para aluno do curso");

        assert_eq!(result.forum(), DemandForum::Institutional);
        assert_eq!(result.course_name(), Some("Ciência da Computação"));
        assert_eq!(result.status(), DemandStatus::Open);
        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn should_create_institutional_demand_successfully_for_ca_manager_even_from_another_course()
     {
        let (repo, port, ca_id, author_id) = setup();

        // O autor é gestor do CA, mesmo tendo curso diferente
        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Outro Curso".to_string()),
                vec!["Ciência da Computação".to_string()],
                true, // Gestor do CA
            ),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::Institutional,
            Some("Demanda da Gestão".to_string()),
            "Acompanhamento da oferta de disciplinas".to_string(),
            None,
            Some("Ciência da Computação".to_string()),
        );

        let result = create_demand(&repo, &port, command)
            .await
            .expect("gestor do CA pode registrar demanda institucional");

        assert_eq!(result.forum(), DemandForum::Institutional);
        assert_eq!(result.course_name(), Some("Ciência da Computação"));
    }

    #[tokio::test]
    async fn should_reject_demand_creation_when_ca_does_not_exist() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(false, false, true, None, vec![], false),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            None,
            "Problema qualquer".to_string(),
            None,
            None,
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::AcademicCenterNotFound(id)) if id == ca_id
        ));
    }

    #[tokio::test]
    async fn should_reject_demand_creation_when_ca_is_inactive() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(true, false, true, None, vec![], false),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            None,
            "Problema qualquer".to_string(),
            None,
            None,
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::AcademicCenterInactive(id)) if id == ca_id
        ));
    }

    #[tokio::test]
    async fn should_reject_demand_creation_when_author_lacks_active_student_membership() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                false, // Sem vínculo estudantil ativo
                None,
                vec!["Engenharia de Redes".to_string()],
                false,
            ),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            None,
            "Problema qualquer".to_string(),
            None,
            None,
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::StudentMembershipRequired)
        ));
    }

    #[tokio::test]
    async fn should_reject_physical_space_demand_with_course_name() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Computação".to_string()),
                vec!["Computação".to_string()],
                false,
            ),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            None,
            "Lâmpada queimada".to_string(),
            None,
            Some("Computação".to_string()),
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::InvalidDemand(
                DemandError::CourseNotAllowedForPhysicalSpace
            ))
        ));
    }

    #[tokio::test]
    async fn should_reject_institutional_demand_without_course_name() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Ciência da Computação".to_string()),
                vec!["Ciência da Computação".to_string()],
                false,
            ),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::Institutional,
            None,
            "Problema com professor".to_string(),
            None,
            None, // Ausência de curso
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::CourseNameRequired)
        ));
    }

    #[tokio::test]
    async fn should_reject_institutional_demand_when_course_is_not_represented_by_ca() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Engenharia Civil".to_string()),
                vec!["Ciência da Computação".to_string()],
                false,
            ),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::Institutional,
            None,
            "Problema curricular".to_string(),
            None,
            Some("Engenharia Civil".to_string()),
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::CourseNotRepresentedByCa { course_name, .. })
                if course_name == "Engenharia Civil"
        ));
    }

    #[tokio::test]
    async fn should_reject_institutional_demand_when_student_belongs_to_another_unauthorized_course()
     {
        let (repo, port, ca_id, author_id) = setup();

        // O CA representa Computação, mas o aluno é de Engenharia de Redes e não é gestor
        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Engenharia de Redes".to_string()),
                vec!["Ciência da Computação".to_string()],
                false,
            ),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::Institutional,
            None,
            "Problema pedagógico".to_string(),
            None,
            Some("Ciência da Computação".to_string()),
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::UserNotPermittedInCa)
        ));
    }

    #[tokio::test]
    async fn should_reject_demand_with_empty_or_whitespace_description() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(true, true, true, None, vec![], false),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            None,
            "   \t \n  ".to_string(),
            None,
            None,
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::InvalidDemand(
                DemandError::EmptyDescription
            ))
        ));
    }

    #[tokio::test]
    async fn should_reject_demand_with_description_exceeding_max_length() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(true, true, true, None, vec![], false),
        );

        let long_description = "x".repeat(MAX_DESCRIPTION_LENGTH + 1);
        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            None,
            long_description,
            None,
            None,
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::InvalidDemand(
                DemandError::DescriptionTooLong { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn should_reject_demand_with_empty_or_whitespace_title() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(true, true, true, None, vec![], false),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            Some("    ".to_string()),
            "Descrição válida".to_string(),
            None,
            None,
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::InvalidDemand(
                DemandError::EmptyTitle
            ))
        ));
    }

    #[tokio::test]
    async fn should_reject_demand_with_empty_or_whitespace_category() {
        let (repo, port, ca_id, author_id) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(true, true, true, None, vec![], false),
        );

        let command = CreateDemandCommand::new(
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            None,
            "Descrição válida".to_string(),
            Some("    ".to_string()),
            None,
        );

        let result = create_demand(&repo, &port, command).await;
        assert!(matches!(
            result,
            Err(DemandApplicationError::InvalidDemand(
                DemandError::EmptyCategory
            ))
        ));
    }
}
