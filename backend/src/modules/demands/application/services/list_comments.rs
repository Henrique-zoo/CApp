//! Caso de uso: listagem de comentários de uma demanda.
//!
//! Coordena a validação de autorização (apenas autor da demanda ou membros da gestão do CA),
//! isolamento por Centro Acadêmico e recuperação dos comentários em ordem cronológica ascendente.

use crate::modules::demands::{
    application::{
        error::DemandApplicationError,
        ports::{AcademicAffiliationPort, DemandCommentRepository, DemandRepository},
        queries::ListDemandCommentsQuery,
    },
    domain::entities::DemandComment,
};

/// Serviço para listagem dos comentários de uma demanda em ordem cronológica ascendente.
#[derive(Debug, Clone)]
pub struct ListDemandCommentsUseCase<D, C, A> {
    demand_repository: D,
    comment_repository: C,
    affiliation_port: A,
}

impl<D, C, A> ListDemandCommentsUseCase<D, C, A>
where
    D: DemandRepository,
    C: DemandCommentRepository,
    A: AcademicAffiliationPort,
{
    /// Inicializa o caso de uso com os adaptadores de repositório e afiliação.
    pub fn new(demand_repository: D, comment_repository: C, affiliation_port: A) -> Self {
        Self {
            demand_repository,
            comment_repository,
            affiliation_port,
        }
    }

    /// Executa o caso de uso de listagem de comentários de uma demanda.
    ///
    /// # Regras de Negócio e Controle de Acesso
    ///
    /// 1. O Centro Acadêmico deve existir e estar ativo.
    /// 2. O usuário solicitante deve possuir vínculo estudantil ativo na instituição.
    /// 3. A demanda deve existir e pertencer ao Centro Acadêmico informado.
    /// 4. Apenas o autor da demanda ou membros com cargo de gestão ativo no respectivo CA
    ///    possuem permissão para visualizar as interações e comentários da demanda.
    /// 5. Os comentários são retornados em ordem cronológica ascendente (`created_at ASC, id ASC`).
    pub async fn execute(
        &self,
        query: ListDemandCommentsQuery,
    ) -> Result<Vec<DemandComment>, DemandApplicationError> {
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

        // 4. Localiza a demanda associada
        let demand = self
            .demand_repository
            .find_by_id(query.demand_id)
            .await?
            .ok_or(DemandApplicationError::DemandNotFound(query.demand_id))?;

        // Isolamento de CA: a demanda deve pertencer ao CA informado
        if demand.academic_center_id() != query.academic_center_id {
            return Err(DemandApplicationError::DemandBelongsToAnotherCa {
                demand_id: query.demand_id,
                ca_id: query.academic_center_id,
            });
        }

        // 5. Permissão de visualização: apenas autor da demanda ou gestor do CA
        let has_permission = affiliation.is_ca_manager || query.user_id == demand.author_id();
        if !has_permission {
            return Err(DemandApplicationError::UserNotPermittedToComment);
        }

        // 6. Consulta os comentários no repositório (já ordenados cronologicamente)
        let mut comments = self
            .comment_repository
            .list_by_demand_id(query.demand_id)
            .await?;

        // Garante ordenação cronológica ascendente: created_at ASC, id ASC
        comments.sort_by(|a, b| {
            a.created_at()
                .cmp(&b.created_at())
                .then_with(|| a.id().cmp(&b.id()))
        });

        Ok(comments)
    }
}

/// Função utilitária para execução direta do caso de uso de listagem de comentários.
pub async fn list_demand_comments<D, C, A>(
    demand_repository: &D,
    comment_repository: &C,
    affiliation_port: &A,
    query: ListDemandCommentsQuery,
) -> Result<Vec<DemandComment>, DemandApplicationError>
where
    D: DemandRepository,
    C: DemandCommentRepository,
    A: AcademicAffiliationPort,
{
    let use_case =
        ListDemandCommentsUseCase::new(demand_repository, comment_repository, affiliation_port);
    use_case.execute(query).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::modules::demands::{
        application::ports::AcademicAffiliation,
        domain::entities::{Demand, DemandCommentAuthorRole, DemandForum},
        infrastructure::{
            InMemoryAcademicAffiliationPort, InMemoryDemandCommentRepository,
            InMemoryDemandRepository,
        },
    };

    fn setup() -> (
        InMemoryDemandRepository,
        InMemoryDemandCommentRepository,
        InMemoryAcademicAffiliationPort,
        Uuid,
        Uuid,
        Uuid,
    ) {
        let demand_repo = InMemoryDemandRepository::new();
        let comment_repo = InMemoryDemandCommentRepository::new();
        let affiliation_port = InMemoryAcademicAffiliationPort::new();

        let ca_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();

        // Configura autor com vínculo ativo
        affiliation_port.set_affiliation(
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

        // Configura gestor do CA com vínculo ativo
        affiliation_port.set_affiliation(
            manager_id,
            ca_id,
            AcademicAffiliation::new(
                true,
                true,
                true,
                Some("Ciência da Computação".to_string()),
                vec!["Ciência da Computação".to_string()],
                true,
            ),
        );

        (
            demand_repo,
            comment_repo,
            affiliation_port,
            ca_id,
            author_id,
            manager_id,
        )
    }

    async fn create_sample_demand(
        repo: &InMemoryDemandRepository,
        ca_id: Uuid,
        author_id: Uuid,
    ) -> Demand {
        let demand = Demand::new(
            Uuid::new_v4(),
            ca_id,
            author_id,
            DemandForum::PhysicalSpace,
            Some("Reparo da fechadura".to_string()),
            "A fechadura da porta do CA quebrou".to_string(),
            None,
            None,
            Utc::now(),
        )
        .unwrap();
        repo.create(&demand).await.unwrap();
        demand
    }

    #[tokio::test]
    async fn should_list_comments_chronologically_for_author() {
        let (demand_repo, comment_repo, port, ca_id, author_id, manager_id) = setup();
        let demand = create_sample_demand(&demand_repo, ca_id, author_id).await;

        let base_time = Utc::now();

        // Comentário 1: autor em t0
        let c1 = DemandComment::new(
            Uuid::new_v4(),
            demand.id(),
            author_id,
            DemandCommentAuthorRole::Student,
            "Primeiro esclarecimento do autor".to_string(),
            base_time,
        )
        .unwrap();

        // Comentário 2: gestor em t1
        let c2 = DemandComment::new(
            Uuid::new_v4(),
            demand.id(),
            manager_id,
            DemandCommentAuthorRole::CaManagement,
            "Resposta da gestão em análise".to_string(),
            base_time + Duration::minutes(10),
        )
        .unwrap();

        // Comentário 3: autor em t2
        let c3 = DemandComment::new(
            Uuid::new_v4(),
            demand.id(),
            author_id,
            DemandCommentAuthorRole::Student,
            "Agradecimento do autor".to_string(),
            base_time + Duration::minutes(20),
        )
        .unwrap();

        // Inseridos fora de ordem na persistência
        comment_repo.create(&c2).await.unwrap();
        comment_repo.create(&c3).await.unwrap();
        comment_repo.create(&c1).await.unwrap();

        let query = ListDemandCommentsQuery::new(ca_id, demand.id(), author_id);
        let comments = list_demand_comments(&demand_repo, &comment_repo, &port, query)
            .await
            .expect("deve listar comentários para o autor");

        assert_eq!(comments.len(), 3);
        assert_eq!(comments[0].id(), c1.id());
        assert_eq!(comments[0].author_role(), DemandCommentAuthorRole::Student);
        assert_eq!(comments[1].id(), c2.id());
        assert_eq!(
            comments[1].author_role(),
            DemandCommentAuthorRole::CaManagement
        );
        assert_eq!(comments[2].id(), c3.id());
        assert_eq!(comments[2].author_role(), DemandCommentAuthorRole::Student);
    }

    #[tokio::test]
    async fn should_list_comments_for_ca_manager() {
        let (demand_repo, comment_repo, port, ca_id, author_id, manager_id) = setup();
        let demand = create_sample_demand(&demand_repo, ca_id, author_id).await;

        let c1 = DemandComment::new(
            Uuid::new_v4(),
            demand.id(),
            author_id,
            DemandCommentAuthorRole::Student,
            "Comentário do autor".to_string(),
            Utc::now(),
        )
        .unwrap();
        comment_repo.create(&c1).await.unwrap();

        let query = ListDemandCommentsQuery::new(ca_id, demand.id(), manager_id);
        let comments = list_demand_comments(&demand_repo, &comment_repo, &port, query)
            .await
            .expect("gestor do CA pode visualizar interações");

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].id(), c1.id());
    }

    #[tokio::test]
    async fn should_reject_listing_for_unauthorized_third_party_with_403() {
        let (demand_repo, comment_repo, port, ca_id, author_id, _) = setup();
        let demand = create_sample_demand(&demand_repo, ca_id, author_id).await;

        let third_party_id = Uuid::new_v4();
        port.set_affiliation(
            third_party_id,
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

        let query = ListDemandCommentsQuery::new(ca_id, demand.id(), third_party_id);
        let err = list_demand_comments(&demand_repo, &comment_repo, &port, query)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::UserNotPermittedToComment
        ));
    }

    #[tokio::test]
    async fn should_reject_listing_when_ca_does_not_exist() {
        let (demand_repo, comment_repo, port, _, author_id, _) = setup();
        let non_existent_ca = Uuid::new_v4();

        let query = ListDemandCommentsQuery::new(non_existent_ca, Uuid::new_v4(), author_id);
        let err = list_demand_comments(&demand_repo, &comment_repo, &port, query)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::AcademicCenterNotFound(id) if id == non_existent_ca
        ));
    }

    #[tokio::test]
    async fn should_reject_listing_when_demand_does_not_exist() {
        let (demand_repo, comment_repo, port, ca_id, author_id, _) = setup();
        let non_existent_demand = Uuid::new_v4();

        let query = ListDemandCommentsQuery::new(ca_id, non_existent_demand, author_id);
        let err = list_demand_comments(&demand_repo, &comment_repo, &port, query)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::DemandNotFound(id) if id == non_existent_demand
        ));
    }

    #[tokio::test]
    async fn should_reject_listing_when_demand_belongs_to_another_ca() {
        let (demand_repo, comment_repo, port, ca_1_id, author_id, _) = setup();
        let ca_2_id = Uuid::new_v4();

        let demand = create_sample_demand(&demand_repo, ca_2_id, author_id).await;

        let query = ListDemandCommentsQuery::new(ca_1_id, demand.id(), author_id);
        let err = list_demand_comments(&demand_repo, &comment_repo, &port, query)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::DemandBelongsToAnotherCa { demand_id, ca_id }
                if demand_id == demand.id() && ca_id == ca_1_id
        ));
    }
}
