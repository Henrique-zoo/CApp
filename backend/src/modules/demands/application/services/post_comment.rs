//! Caso de uso: publicação de comentários e interações em demandas.
//!
//! Coordena a validação de autorização (apenas autor da demanda ou membros da gestão do CA),
//! identificação explícita do papel do autor (`student` vs. `ca_management`),
//! validação de conteúdo e persistência no repositório.

use chrono::Utc;
use uuid::Uuid;

use crate::modules::demands::{
    application::{
        commands::PostDemandCommentCommand,
        error::DemandApplicationError,
        ports::{AcademicAffiliationPort, DemandCommentRepository, DemandRepository},
    },
    domain::entities::{DemandComment, DemandCommentAuthorRole},
};

/// Serviço para publicação de comentários em demandas acadêmicas e de espaço físico.
#[derive(Debug, Clone)]
pub struct PostDemandCommentUseCase<D, C, A> {
    demand_repository: D,
    comment_repository: C,
    affiliation_port: A,
}

impl<D, C, A> PostDemandCommentUseCase<D, C, A>
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

    /// Executa o caso de uso de publicação de comentário em uma demanda.
    ///
    /// # Regras de Negócio e Controle de Acesso
    ///
    /// 1. O Centro Acadêmico deve existir e estar ativo.
    /// 2. O usuário autor deve possuir vínculo estudantil ativo na instituição.
    /// 3. A demanda deve existir e estar vinculada ao Centro Acadêmico indicado.
    /// 4. Permissão de interação:
    ///    - Apenas o autor da demanda ou membros com cargo de gestão ativo no respectivo CA podem comentar.
    ///    - Se for membro da gestão do CA, o comentário é registrado com papel [`DemandCommentAuthorRole::CaManagement`].
    ///    - Se for o estudante autor da demanda, é registrado com papel [`DemandCommentAuthorRole::Student`].
    ///    - Qualquer outro usuário tem a publicação rejeitada com [`DemandApplicationError::UserNotPermittedToComment`].
    /// 5. O conteúdo não pode ser vazio (nem apenas espaços) e não pode exceder o limite de 2000 caracteres.
    pub async fn execute(
        &self,
        command: PostDemandCommentCommand,
    ) -> Result<DemandComment, DemandApplicationError> {
        let affiliation = self
            .affiliation_port
            .get_affiliation(command.author_id, command.academic_center_id)
            .await?;

        // 1. O Centro Acadêmico deve existir
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

        // 4. Localiza a demanda associada
        let demand = self
            .demand_repository
            .find_by_id(command.demand_id)
            .await?
            .ok_or(DemandApplicationError::DemandNotFound(command.demand_id))?;

        // Isolamento de CA: a demanda deve pertencer ao CA informado
        if demand.academic_center_id() != command.academic_center_id {
            return Err(DemandApplicationError::DemandBelongsToAnotherCa {
                demand_id: command.demand_id,
                ca_id: command.academic_center_id,
            });
        }

        // 5. Determina o papel do autor e verifica permissão
        let author_role = if affiliation.is_ca_manager {
            DemandCommentAuthorRole::CaManagement
        } else if command.author_id == demand.author_id() {
            DemandCommentAuthorRole::Student
        } else {
            return Err(DemandApplicationError::UserNotPermittedToComment);
        };

        // 6. Instancia a entidade de comentário e valida invariantes
        let comment = DemandComment::new(
            Uuid::new_v4(),
            command.demand_id,
            command.author_id,
            author_role,
            command.content,
            Utc::now(),
        )?;

        // 7. Persiste o comentário no repositório
        self.comment_repository.create(&comment).await?;

        Ok(comment)
    }
}

/// Função utilitária para execução direta do caso de uso de publicação de comentário.
pub async fn post_demand_comment<D, C, A>(
    demand_repository: &D,
    comment_repository: &C,
    affiliation_port: &A,
    command: PostDemandCommentCommand,
) -> Result<DemandComment, DemandApplicationError>
where
    D: DemandRepository,
    C: DemandCommentRepository,
    A: AcademicAffiliationPort,
{
    let use_case =
        PostDemandCommentUseCase::new(demand_repository, comment_repository, affiliation_port);
    use_case.execute(command).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::demands::{
        application::ports::AcademicAffiliation,
        domain::{
            entities::{Demand, DemandError, DemandForum},
            rules::MAX_DEMAND_COMMENT_CONTENT_LENGTH,
        },
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
    async fn should_post_comment_successfully_by_demand_author() {
        let (demand_repo, comment_repo, port, ca_id, author_id, _) = setup();
        let demand = create_sample_demand(&demand_repo, ca_id, author_id).await;

        let command = PostDemandCommentCommand::new(
            ca_id,
            demand.id(),
            author_id,
            "Anexei a foto da fechadura quebrada.".to_string(),
        );

        let comment = post_demand_comment(&demand_repo, &comment_repo, &port, command)
            .await
            .expect("autor deve poder comentar");

        assert_eq!(comment.demand_id(), demand.id());
        assert_eq!(comment.author_id(), author_id);
        assert_eq!(comment.author_role(), DemandCommentAuthorRole::Student);
        assert_eq!(comment.content(), "Anexei a foto da fechadura quebrada.");
        assert_eq!(comment_repo.count(), 1);
    }

    #[tokio::test]
    async fn should_post_official_response_successfully_by_ca_management() {
        let (demand_repo, comment_repo, port, ca_id, author_id, manager_id) = setup();
        let demand = create_sample_demand(&demand_repo, ca_id, author_id).await;

        let command = PostDemandCommentCommand::new(
            ca_id,
            demand.id(),
            manager_id,
            "Chamado aberto junto à prefeitura do campus.".to_string(),
        );

        let comment = post_demand_comment(&demand_repo, &comment_repo, &port, command)
            .await
            .expect("gestor deve poder responder");

        assert_eq!(comment.demand_id(), demand.id());
        assert_eq!(comment.author_id(), manager_id);
        assert_eq!(comment.author_role(), DemandCommentAuthorRole::CaManagement);
        assert_eq!(
            comment.content(),
            "Chamado aberto junto à prefeitura do campus."
        );
        assert_eq!(comment_repo.count(), 1);
    }

    #[tokio::test]
    async fn should_reject_comment_by_unauthorized_third_party_with_403() {
        let (demand_repo, comment_repo, port, ca_id, author_id, _) = setup();
        let demand = create_sample_demand(&demand_repo, ca_id, author_id).await;

        let third_party_id = Uuid::new_v4();
        // Terceiro possui vínculo ativo, mas não é autor nem gestor
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

        let command = PostDemandCommentCommand::new(
            ca_id,
            demand.id(),
            third_party_id,
            "Tentativa de intromissão.".to_string(),
        );

        let err = post_demand_comment(&demand_repo, &comment_repo, &port, command)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::UserNotPermittedToComment
        ));
        assert_eq!(comment_repo.count(), 0);
    }

    #[tokio::test]
    async fn should_reject_comment_with_empty_or_whitespace_content() {
        let (demand_repo, comment_repo, port, ca_id, author_id, _) = setup();
        let demand = create_sample_demand(&demand_repo, ca_id, author_id).await;

        let command =
            PostDemandCommentCommand::new(ca_id, demand.id(), author_id, "   \t \n  ".to_string());

        let err = post_demand_comment(&demand_repo, &comment_repo, &port, command)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::InvalidDemand(DemandError::EmptyCommentContent)
        ));
    }

    #[tokio::test]
    async fn should_reject_comment_exceeding_max_length() {
        let (demand_repo, comment_repo, port, ca_id, author_id, _) = setup();
        let demand = create_sample_demand(&demand_repo, ca_id, author_id).await;

        let long_text = "x".repeat(MAX_DEMAND_COMMENT_CONTENT_LENGTH + 1);
        let command = PostDemandCommentCommand::new(ca_id, demand.id(), author_id, long_text);

        let err = post_demand_comment(&demand_repo, &comment_repo, &port, command)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::InvalidDemand(DemandError::CommentContentTooLong { max })
                if max == MAX_DEMAND_COMMENT_CONTENT_LENGTH
        ));
    }

    #[tokio::test]
    async fn should_reject_comment_when_ca_does_not_exist() {
        let (demand_repo, comment_repo, port, _, author_id, _) = setup();
        let non_existent_ca = Uuid::new_v4();

        let command = PostDemandCommentCommand::new(
            non_existent_ca,
            Uuid::new_v4(),
            author_id,
            "Texto qualquer".to_string(),
        );

        let err = post_demand_comment(&demand_repo, &comment_repo, &port, command)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::AcademicCenterNotFound(id) if id == non_existent_ca
        ));
    }

    #[tokio::test]
    async fn should_reject_comment_when_ca_is_inactive() {
        let (demand_repo, comment_repo, port, ca_id, author_id, _) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(true, false, true, None, vec![], false),
        );

        let command = PostDemandCommentCommand::new(
            ca_id,
            Uuid::new_v4(),
            author_id,
            "Texto qualquer".to_string(),
        );

        let err = post_demand_comment(&demand_repo, &comment_repo, &port, command)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::AcademicCenterInactive(id) if id == ca_id
        ));
    }

    #[tokio::test]
    async fn should_reject_comment_when_author_lacks_student_membership() {
        let (demand_repo, comment_repo, port, ca_id, author_id, _) = setup();

        port.set_affiliation(
            author_id,
            ca_id,
            AcademicAffiliation::new(true, true, false, None, vec![], false),
        );

        let command = PostDemandCommentCommand::new(
            ca_id,
            Uuid::new_v4(),
            author_id,
            "Texto qualquer".to_string(),
        );

        let err = post_demand_comment(&demand_repo, &comment_repo, &port, command)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::StudentMembershipRequired
        ));
    }

    #[tokio::test]
    async fn should_reject_comment_when_demand_does_not_exist() {
        let (demand_repo, comment_repo, port, ca_id, author_id, _) = setup();
        let non_existent_demand = Uuid::new_v4();

        let command = PostDemandCommentCommand::new(
            ca_id,
            non_existent_demand,
            author_id,
            "Texto qualquer".to_string(),
        );

        let err = post_demand_comment(&demand_repo, &comment_repo, &port, command)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::DemandNotFound(id) if id == non_existent_demand
        ));
    }

    #[tokio::test]
    async fn should_reject_comment_when_demand_belongs_to_another_ca() {
        let (demand_repo, comment_repo, port, ca_1_id, author_id, _) = setup();
        let ca_2_id = Uuid::new_v4();

        // Demanda foi criada no CA 2
        let demand = create_sample_demand(&demand_repo, ca_2_id, author_id).await;

        // Autor tenta comentar informando CA 1 na rota
        let command = PostDemandCommentCommand::new(
            ca_1_id,
            demand.id(),
            author_id,
            "Comentário no CA errado".to_string(),
        );

        let err = post_demand_comment(&demand_repo, &comment_repo, &port, command)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DemandApplicationError::DemandBelongsToAnotherCa { demand_id, ca_id }
                if demand_id == demand.id() && ca_id == ca_1_id
        ));
    }
}
