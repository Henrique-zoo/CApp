//! Caso de uso: consulta detalhada de demandas com histórico.
//!
//! Coordena regras de visibilidade, controle de acesso aos fóruns institucional
//! e de espaço físico, e isolamento estrito por Centro Acadêmico.

use crate::modules::demands::{
    application::{
        error::DemandApplicationError,
        ports::{AcademicAffiliationPort, DemandRepository},
        queries::GetDemandQuery,
    },
    domain::entities::{Demand, DemandForum},
};

/// Serviço para recuperação dos detalhes e histórico de uma demanda.
#[derive(Debug, Clone)]
pub struct GetDemandUseCase<R, A> {
    demand_repository: R,
    affiliation_port: A,
}

impl<R, A> GetDemandUseCase<R, A>
where
    R: DemandRepository,
    A: AcademicAffiliationPort,
{
    /// Inicializa o caso de uso com os adaptadores de repositório e afiliação.
    pub fn new(demand_repository: R, affiliation_port: A) -> Self {
        Self {
            demand_repository,
            affiliation_port,
        }
    }

    /// Executa a consulta de detalhes da demanda.
    ///
    /// # Regras de Negócio e Acesso
    ///
    /// 1. O Centro Acadêmico deve existir e estar ativo.
    /// 2. O usuário autenticado deve possuir vínculo estudantil ativo.
    /// 3. A demanda solicitada deve existir e pertencer estritamente ao `academic_center_id` informado
    ///    ([`DemandApplicationError::DemandNotFound`]), garantindo isolamento total entre CAs.
    /// 4. Visibilidade do Fórum de Espaço Físico:
    ///    - Aberto a qualquer estudante com vínculo ativo no app.
    /// 5. Visibilidade do Fórum Institucional:
    ///    - Restrito aos discentes do curso representado pelo CA e aos membros da gestão
    ///      ([`DemandApplicationError::UserNotPermittedInCa`]).
    pub async fn execute(&self, query: GetDemandQuery) -> Result<Demand, DemandApplicationError> {
        let affiliation = self
            .affiliation_port
            .get_affiliation(query.user_id, query.academic_center_id)
            .await?;

        if !affiliation.academic_center_exists {
            return Err(DemandApplicationError::AcademicCenterNotFound(
                query.academic_center_id,
            ));
        }

        if !affiliation.academic_center_active {
            return Err(DemandApplicationError::AcademicCenterInactive(
                query.academic_center_id,
            ));
        }

        if !affiliation.has_active_student_membership {
            return Err(DemandApplicationError::StudentMembershipRequired);
        }

        let demand = self
            .demand_repository
            .find_by_id(query.demand_id)
            .await?
            .ok_or(DemandApplicationError::DemandNotFound(query.demand_id))?;

        if demand.academic_center_id() != query.academic_center_id {
            return Err(DemandApplicationError::DemandNotFound(query.demand_id));
        }

        if demand.forum() == DemandForum::Institutional {
            let has_access = affiliation.is_ca_manager
                || affiliation
                    .student_course
                    .as_deref()
                    .is_some_and(|course| affiliation.ca_represents_course(course));

            if !has_access {
                return Err(DemandApplicationError::UserNotPermittedInCa);
            }
        }

        Ok(demand)
    }
}

/// Função de conveniência para execução direta da consulta de detalhes de demanda.
pub async fn get_demand<R, A>(
    demand_repository: R,
    affiliation_port: A,
    query: GetDemandQuery,
) -> Result<Demand, DemandApplicationError>
where
    R: DemandRepository,
    A: AcademicAffiliationPort,
{
    GetDemandUseCase::new(demand_repository, affiliation_port)
        .execute(query)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::demands::{
        application::ports::AcademicAffiliation,
        infrastructure::{InMemoryAcademicAffiliationPort, InMemoryDemandRepository},
    };
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_ca_id() -> Uuid {
        Uuid::new_v4()
    }

    fn sample_user_id() -> Uuid {
        Uuid::new_v4()
    }

    fn create_test_demand(ca_id: Uuid, author_id: Uuid, forum: DemandForum) -> Demand {
        Demand::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            forum,
            Some("Título".to_string()),
            "Descrição detalhada".to_string(),
            None,
            if forum == DemandForum::Institutional {
                Some("Ciência da Computação".to_string())
            } else {
                None
            },
            Utc::now(),
        )
        .expect("demanda válida")
    }

    #[tokio::test]
    async fn should_allow_any_active_student_to_view_physical_space_demand_details() {
        let ca_id = sample_ca_id();
        let other_student_id = sample_user_id();
        let author_id = sample_user_id();

        let repo = InMemoryDemandRepository::new();
        let demand = create_test_demand(ca_id, author_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        // Estudante de outro curso (Engenharia de Redes), não é do curso do CA nem gestor
        affiliation_port.set_affiliation(
            other_student_id,
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

        let query = GetDemandQuery::new(ca_id, demand_id, other_student_id);
        let retrieved = get_demand(&repo, &affiliation_port, query)
            .await
            .expect("deve permitir consulta de espaço físico");

        assert_eq!(retrieved.id(), demand_id);
    }

    #[tokio::test]
    async fn should_allow_student_of_course_to_view_institutional_demand_details() {
        let ca_id = sample_ca_id();
        let course_student_id = sample_user_id();
        let author_id = sample_user_id();

        let repo = InMemoryDemandRepository::new();
        let demand = create_test_demand(ca_id, author_id, DemandForum::Institutional);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            course_student_id,
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

        let query = GetDemandQuery::new(ca_id, demand_id, course_student_id);
        let retrieved = get_demand(&repo, &affiliation_port, query)
            .await
            .expect("deve permitir consulta institucional");

        assert_eq!(retrieved.id(), demand_id);
    }

    #[tokio::test]
    async fn should_allow_ca_manager_to_view_institutional_demand_details() {
        let ca_id = sample_ca_id();
        let manager_id = sample_user_id();
        let author_id = sample_user_id();

        let repo = InMemoryDemandRepository::new();
        let demand = create_test_demand(ca_id, author_id, DemandForum::Institutional);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        // Gestor pode ser até de outro curso, mas é gestor
        affiliation_port.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(true, true, true, None, vec![], true),
        );

        let query = GetDemandQuery::new(ca_id, demand_id, manager_id);
        let retrieved = get_demand(&repo, &affiliation_port, query)
            .await
            .expect("deve permitir consulta pelo gestor");

        assert_eq!(retrieved.id(), demand_id);
    }

    #[tokio::test]
    async fn should_reject_student_of_another_course_from_viewing_institutional_demand() {
        let ca_id = sample_ca_id();
        let outsider_id = sample_user_id();
        let author_id = sample_user_id();

        let repo = InMemoryDemandRepository::new();
        let demand = create_test_demand(ca_id, author_id, DemandForum::Institutional);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            outsider_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Letras".to_string()),
                vec!["Ciência da Computação".to_string()],
                false,
            ),
        );

        let query = GetDemandQuery::new(ca_id, demand_id, outsider_id);
        let result = get_demand(&repo, &affiliation_port, query).await;
        match result {
            Err(DemandApplicationError::UserNotPermittedInCa) => (),
            other => panic!("esperava UserNotPermittedInCa, obteve {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_reject_querying_demand_from_another_ca() {
        let ca_1 = sample_ca_id();
        let ca_2 = sample_ca_id();
        let user_id = sample_user_id();

        let repo = InMemoryDemandRepository::new();
        // Demanda cadastrada no CA 2
        let demand = create_test_demand(ca_2, user_id, DemandForum::PhysicalSpace);
        let demand_id = demand.id();
        repo.create(&demand).await.unwrap();

        let affiliation_port = InMemoryAcademicAffiliationPort::new();
        affiliation_port.set_affiliation(
            user_id,
            ca_1,
            AcademicAffiliation::new(true, true, true, None, vec![], false),
        );

        // Usuário consulta no contexto do CA 1
        let query = GetDemandQuery::new(ca_1, demand_id, user_id);
        let result = get_demand(&repo, &affiliation_port, query).await;
        match result {
            Err(DemandApplicationError::DemandNotFound(id)) => assert_eq!(id, demand_id),
            other => panic!("esperava DemandNotFound, obteve {other:?}"),
        }
    }
}
