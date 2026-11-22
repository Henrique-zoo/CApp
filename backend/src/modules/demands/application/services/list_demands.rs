//! Caso de uso: listagem e filtragem de demandas nos fóruns institucional e de espaço físico.
//!
//! Coordena regras de visibilidade, isolamento por Centro Acadêmico, autorização
//! estudantil/gestão e filtros determinísticos (fórum, status, demandas próprias e paginação).

use crate::modules::demands::{
    application::{
        error::DemandApplicationError,
        ports::{AcademicAffiliationPort, DemandFilter, DemandRepository},
        queries::ListDemandsQuery,
    },
    domain::entities::{Demand, DemandForum},
};

/// Serviço para listagem e filtragem de demandas de um Centro Acadêmico.
#[derive(Debug, Clone)]
pub struct ListDemandsUseCase<R, A> {
    demand_repository: R,
    affiliation_port: A,
}

impl<R, A> ListDemandsUseCase<R, A>
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

    /// Executa o caso de uso de listagem e filtragem de demandas.
    ///
    /// # Regras de Negócio e Controle de Acesso
    ///
    /// 1. O Centro Acadêmico deve existir e estar ativo.
    /// 2. O usuário autenticado deve possuir vínculo estudantil ativo na instituição.
    /// 3. Visibilidade do Fórum Institucional:
    ///    - Restrito aos discentes matriculados nos cursos representados pelo CA e aos membros da gestão.
    ///    - Se um usuário sem acesso institucional solicitar expressamente `forum = institutional`,
    ///      a requisição é rejeitada com [`DemandApplicationError::UserNotPermittedInCa`].
    ///    - Se o usuário sem acesso institucional consultar a listagem geral sem filtro de fórum (`forum = None`),
    ///      são retornadas exclusivamente as demandas do fórum de espaço físico, preservando o sigilo institucional.
    /// 4. Visibilidade do Fórum de Espaço Físico:
    ///    - Acessível a qualquer estudante da instituição com vínculo ativo, inclusive de outros cursos.
    /// 5. Filtro de demandas próprias (`mine = true`):
    ///    - Restringe os resultados às demandas registradas pelo próprio usuário autenticado.
    /// 6. Isolamento estrito entre Centros Acadêmicos:
    ///    - Apenas demandas vinculadas ao `academic_center_id` da consulta são retornadas.
    pub async fn execute(
        &self,
        query: ListDemandsQuery,
    ) -> Result<Vec<Demand>, DemandApplicationError> {
        let affiliation = self
            .affiliation_port
            .get_affiliation(query.user_id, query.academic_center_id)
            .await?;

        // 1. O Centro Acadêmico deve existir
        if !affiliation.academic_center_exists {
            return Err(DemandApplicationError::AcademicCenterNotFound(
                query.academic_center_id,
            ));
        }

        // 2. O Centro Acadêmico deve estar ativo
        if !affiliation.academic_center_active {
            return Err(DemandApplicationError::AcademicCenterInactive(
                query.academic_center_id,
            ));
        }

        // 3. O usuário autenticado deve possuir vínculo estudantil ativo
        if !affiliation.has_active_student_membership {
            return Err(DemandApplicationError::StudentMembershipRequired);
        }

        // 4. Verificação de permissão para o fórum institucional
        let has_institutional_access = affiliation.is_ca_manager
            || affiliation
                .student_course
                .as_deref()
                .is_some_and(|course| affiliation.ca_represents_course(course));

        let effective_forum = match query.forum {
            Some(DemandForum::Institutional) => {
                if !has_institutional_access {
                    return Err(DemandApplicationError::UserNotPermittedInCa);
                }
                Some(DemandForum::Institutional)
            }
            Some(DemandForum::PhysicalSpace) => Some(DemandForum::PhysicalSpace),
            None => {
                if has_institutional_access {
                    None
                } else {
                    Some(DemandForum::PhysicalSpace)
                }
            }
        };

        // 5. Filtro de autor (demandas próprias)
        let author_filter = if query.mine {
            Some(query.user_id)
        } else {
            None
        };

        // 6. Monta os critérios e consulta o repositório
        let filter = DemandFilter {
            academic_center_id: query.academic_center_id,
            forum: effective_forum,
            status: query.status,
            author_id: author_filter,
            limit: query.limit,
            offset: query.offset,
        };

        self.demand_repository
            .list(&filter)
            .await
            .map_err(Into::into)
    }
}

/// Função utilitária para execução direta do caso de uso de listagem de demandas.
pub async fn list_demands<R, A>(
    demand_repository: &R,
    affiliation_port: &A,
    query: ListDemandsQuery,
) -> Result<Vec<Demand>, DemandApplicationError>
where
    R: DemandRepository,
    A: AcademicAffiliationPort,
{
    let use_case = ListDemandsUseCase::new(demand_repository, affiliation_port);
    use_case.execute(query).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::modules::demands::{
        application::ports::AcademicAffiliation,
        domain::entities::{Demand, DemandForum},
        infrastructure::{InMemoryAcademicAffiliationPort, InMemoryDemandRepository},
    };

    fn setup() -> (
        InMemoryDemandRepository,
        InMemoryAcademicAffiliationPort,
        Uuid,
        Uuid,
        Uuid,
    ) {
        let repo = InMemoryDemandRepository::new();
        let affiliation = InMemoryAcademicAffiliationPort::new();

        let ca_id = Uuid::new_v4();
        let student_cc_id = Uuid::new_v4();
        let student_other_course_id = Uuid::new_v4();

        // CA ativo representando "Ciência da Computação"
        affiliation.set_affiliation(
            student_cc_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Ciência da Computação".to_string()),
                vec![
                    "Centro Acadêmico de Computação".to_string(),
                    "Ciência da Computação".to_string(),
                ],
                false,
            ),
        );

        // Estudante de outro curso com vínculo ativo
        affiliation.set_affiliation(
            student_other_course_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Medicina".to_string()),
                vec![
                    "Centro Acadêmico de Computação".to_string(),
                    "Ciência da Computação".to_string(),
                ],
                false,
            ),
        );

        (
            repo,
            affiliation,
            ca_id,
            student_cc_id,
            student_other_course_id,
        )
    }

    #[tokio::test]
    async fn should_list_physical_space_demands_for_active_student_of_any_course() {
        let (repo, affiliation, ca_id, student_cc_id, student_other_id) = setup();

        let now = Utc::now();
        let demand = Demand::new(
            Uuid::new_v4(),
            ca_id,
            student_cc_id,
            DemandForum::PhysicalSpace,
            Some("Reparo da porta".to_string()),
            "A porta da sala do CA está emperrada".to_string(),
            None,
            None,
            now,
        )
        .unwrap();
        repo.create(&demand).await.unwrap();

        let query = ListDemandsQuery::new(
            ca_id,
            student_other_id,
            Some(DemandForum::PhysicalSpace),
            None,
            false,
            None,
            None,
        );

        let result = list_demands(&repo, &affiliation, query).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title(), Some("Reparo da porta"));
    }

    #[tokio::test]
    async fn should_list_institutional_demands_for_student_of_the_course() {
        let (repo, affiliation, ca_id, student_cc_id, _) = setup();

        let now = Utc::now();
        let demand = Demand::new(
            Uuid::new_v4(),
            ca_id,
            student_cc_id,
            DemandForum::Institutional,
            Some("Ajuste de horário".to_string()),
            "Choque de turmas de cálculo".to_string(),
            None,
            Some("Ciência da Computação".to_string()),
            now,
        )
        .unwrap();
        repo.create(&demand).await.unwrap();

        let query = ListDemandsQuery::new(
            ca_id,
            student_cc_id,
            Some(DemandForum::Institutional),
            None,
            false,
            None,
            None,
        );

        let result = list_demands(&repo, &affiliation, query).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title(), Some("Ajuste de horário"));
    }

    #[tokio::test]
    async fn should_reject_institutional_listing_when_student_belongs_to_another_course() {
        let (repo, affiliation, ca_id, _, student_other_id) = setup();

        let query = ListDemandsQuery::new(
            ca_id,
            student_other_id,
            Some(DemandForum::Institutional),
            None,
            false,
            None,
            None,
        );

        let err = list_demands(&repo, &affiliation, query).await.unwrap_err();
        assert!(matches!(err, DemandApplicationError::UserNotPermittedInCa));
    }

    #[tokio::test]
    async fn should_allow_institutional_listing_for_ca_manager_even_from_another_course() {
        let (repo, affiliation, ca_id, _, _) = setup();
        let manager_id = Uuid::new_v4();

        affiliation.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Física".to_string()),
                vec!["Ciência da Computação".to_string()],
                true, // Gestor do CA
            ),
        );

        let now = Utc::now();
        let demand = Demand::new(
            Uuid::new_v4(),
            ca_id,
            manager_id,
            DemandForum::Institutional,
            Some("Demanda pedagógica".to_string()),
            "Solicitação da gestão à coordenação".to_string(),
            None,
            Some("Ciência da Computação".to_string()),
            now,
        )
        .unwrap();
        repo.create(&demand).await.unwrap();

        let query = ListDemandsQuery::new(
            ca_id,
            manager_id,
            Some(DemandForum::Institutional),
            None,
            false,
            None,
            None,
        );

        let result = list_demands(&repo, &affiliation, query).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title(), Some("Demanda pedagógica"));
    }

    #[tokio::test]
    async fn should_exclude_institutional_demands_when_unauthorized_user_lists_without_forum_filter()
     {
        let (repo, affiliation, ca_id, student_cc_id, student_other_id) = setup();

        let now = Utc::now();
        let demand_physical = Demand::new(
            Uuid::new_v4(),
            ca_id,
            student_cc_id,
            DemandForum::PhysicalSpace,
            Some("Troca de lâmpada".to_string()),
            "Lâmpada queimada na sala do CA".to_string(),
            None,
            None,
            now,
        )
        .unwrap();
        let demand_inst = Demand::new(
            Uuid::new_v4(),
            ca_id,
            student_cc_id,
            DemandForum::Institutional,
            Some("Revisão de ementa".to_string()),
            "Revisão da disciplina de algoritmos".to_string(),
            None,
            Some("Ciência da Computação".to_string()),
            now,
        )
        .unwrap();

        repo.create(&demand_physical).await.unwrap();
        repo.create(&demand_inst).await.unwrap();

        // Consulta sem fórum por estudante de outro curso -> só deve retornar espaço físico
        let query = ListDemandsQuery::new(ca_id, student_other_id, None, None, false, None, None);

        let result = list_demands(&repo, &affiliation, query).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title(), Some("Troca de lâmpada"));
    }

    #[tokio::test]
    async fn should_filter_own_demands_when_mine_is_true() {
        let (repo, affiliation, ca_id, student_a_id, _) = setup();
        let student_b_id = Uuid::new_v4();

        affiliation.set_affiliation(
            student_b_id,
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

        let now = Utc::now();
        let demand_a = Demand::new(
            Uuid::new_v4(),
            ca_id,
            student_a_id,
            DemandForum::PhysicalSpace,
            Some("Reparo da porta".to_string()),
            "Demanda do estudante A".to_string(),
            None,
            None,
            now,
        )
        .unwrap();
        let demand_b = Demand::new(
            Uuid::new_v4(),
            ca_id,
            student_b_id,
            DemandForum::PhysicalSpace,
            Some("Troca da lâmpada".to_string()),
            "Demanda do estudante B".to_string(),
            None,
            None,
            now + Duration::seconds(1),
        )
        .unwrap();

        repo.create(&demand_a).await.unwrap();
        repo.create(&demand_b).await.unwrap();

        // Com mine = true, estudante A só enxerga a sua demanda
        let query_mine = ListDemandsQuery::new(
            ca_id,
            student_a_id,
            Some(DemandForum::PhysicalSpace),
            None,
            true,
            None,
            None,
        );
        let result_mine = list_demands(&repo, &affiliation, query_mine).await.unwrap();
        assert_eq!(result_mine.len(), 1);
        assert_eq!(result_mine[0].title(), Some("Reparo da porta"));

        // Com mine = false, enxerga ambas as demandas visíveis
        let query_all = ListDemandsQuery::new(
            ca_id,
            student_a_id,
            Some(DemandForum::PhysicalSpace),
            None,
            false,
            None,
            None,
        );
        let result_all = list_demands(&repo, &affiliation, query_all).await.unwrap();
        assert_eq!(result_all.len(), 2);
    }

    #[tokio::test]
    async fn should_return_empty_list_when_no_demands_match() {
        let (repo, affiliation, ca_id, student_cc_id, _) = setup();

        let query = ListDemandsQuery::new(
            ca_id,
            student_cc_id,
            Some(DemandForum::PhysicalSpace),
            None,
            false,
            None,
            None,
        );

        let result = list_demands(&repo, &affiliation, query).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn should_isolate_demands_between_academic_centers() {
        let (repo, affiliation, ca_1_id, student_id, _) = setup();
        let ca_2_id = Uuid::new_v4();

        affiliation.set_affiliation(
            student_id,
            ca_2_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Matemática".to_string()),
                vec!["Matemática".to_string()],
                false,
            ),
        );

        let now = Utc::now();
        let demand_ca1 = Demand::new(
            Uuid::new_v4(),
            ca_1_id,
            student_id,
            DemandForum::PhysicalSpace,
            Some("Reparo da porta".to_string()),
            "No CA 1".to_string(),
            None,
            None,
            now,
        )
        .unwrap();
        let demand_ca2 = Demand::new(
            Uuid::new_v4(),
            ca_2_id,
            student_id,
            DemandForum::PhysicalSpace,
            Some("Pintura da sala".to_string()),
            "No CA 2".to_string(),
            None,
            None,
            now,
        )
        .unwrap();

        repo.create(&demand_ca1).await.unwrap();
        repo.create(&demand_ca2).await.unwrap();

        // Consulta CA 1
        let query_ca1 = ListDemandsQuery::new(
            ca_1_id,
            student_id,
            Some(DemandForum::PhysicalSpace),
            None,
            false,
            None,
            None,
        );
        let result_ca1 = list_demands(&repo, &affiliation, query_ca1).await.unwrap();
        assert_eq!(result_ca1.len(), 1);
        assert_eq!(result_ca1[0].title(), Some("Reparo da porta"));

        // Consulta CA 2
        let query_ca2 = ListDemandsQuery::new(
            ca_2_id,
            student_id,
            Some(DemandForum::PhysicalSpace),
            None,
            false,
            None,
            None,
        );
        let result_ca2 = list_demands(&repo, &affiliation, query_ca2).await.unwrap();
        assert_eq!(result_ca2.len(), 1);
        assert_eq!(result_ca2[0].title(), Some("Pintura da sala"));
    }

    #[tokio::test]
    async fn should_reject_listing_when_ca_does_not_exist() {
        let (repo, affiliation, _, student_id, _) = setup();
        let non_existent_ca = Uuid::new_v4();

        let query = ListDemandsQuery::new(
            non_existent_ca,
            student_id,
            Some(DemandForum::PhysicalSpace),
            None,
            false,
            None,
            None,
        );

        let err = list_demands(&repo, &affiliation, query).await.unwrap_err();
        assert!(matches!(
            err,
            DemandApplicationError::AcademicCenterNotFound(id) if id == non_existent_ca
        ));
    }

    #[tokio::test]
    async fn should_reject_listing_when_ca_is_inactive() {
        let (repo, affiliation, ca_id, student_id, _) = setup();

        affiliation.set_affiliation(
            student_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                false, // Inativo
                true,
                Some("Ciência da Computação".to_string()),
                vec!["Ciência da Computação".to_string()],
                false,
            ),
        );

        let query = ListDemandsQuery::new(
            ca_id,
            student_id,
            Some(DemandForum::PhysicalSpace),
            None,
            false,
            None,
            None,
        );

        let err = list_demands(&repo, &affiliation, query).await.unwrap_err();
        assert!(matches!(
            err,
            DemandApplicationError::AcademicCenterInactive(id) if id == ca_id
        ));
    }

    #[tokio::test]
    async fn should_reject_listing_when_student_membership_not_active() {
        let (repo, affiliation, ca_id, student_id, _) = setup();

        affiliation.set_affiliation(
            student_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                false, // Sem vínculo ativo
                Some("Ciência da Computação".to_string()),
                vec!["Ciência da Computação".to_string()],
                false,
            ),
        );

        let query = ListDemandsQuery::new(
            ca_id,
            student_id,
            Some(DemandForum::PhysicalSpace),
            None,
            false,
            None,
            None,
        );

        let err = list_demands(&repo, &affiliation, query).await.unwrap_err();
        assert!(matches!(
            err,
            DemandApplicationError::StudentMembershipRequired
        ));
    }

    #[tokio::test]
    async fn should_paginate_demands_deterministically_with_limit_and_offset() {
        let (repo, affiliation, ca_id, student_id, _) = setup();
        let base_time = Utc::now();

        for i in 1..=5 {
            let demand = Demand::new(
                Uuid::new_v4(),
                ca_id,
                student_id,
                DemandForum::PhysicalSpace,
                Some(format!("Demanda {i}")),
                format!("Descrição {i}"),
                None,
                None,
                base_time + Duration::seconds(i),
            )
            .unwrap();
            repo.create(&demand).await.unwrap();
        }

        // Página 1: limit 2, offset 0 -> deve retornar as mais recentes (5 e 4)
        let page1 = list_demands(
            &repo,
            &affiliation,
            ListDemandsQuery::new(
                ca_id,
                student_id,
                Some(DemandForum::PhysicalSpace),
                None,
                false,
                Some(2),
                Some(0),
            ),
        )
        .await
        .unwrap();

        assert_eq!(page1.len(), 2);
        assert_eq!(page1[0].title(), Some("Demanda 5"));
        assert_eq!(page1[1].title(), Some("Demanda 4"));

        // Página 2: limit 2, offset 2 -> deve retornar (3 e 2)
        let page2 = list_demands(
            &repo,
            &affiliation,
            ListDemandsQuery::new(
                ca_id,
                student_id,
                Some(DemandForum::PhysicalSpace),
                None,
                false,
                Some(2),
                Some(2),
            ),
        )
        .await
        .unwrap();

        assert_eq!(page2.len(), 2);
        assert_eq!(page2[0].title(), Some("Demanda 3"));
        assert_eq!(page2[1].title(), Some("Demanda 2"));
    }
}
