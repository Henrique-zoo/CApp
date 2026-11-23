//! Testes unitários para casos de uso de consultas e enquetes estudantis.

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::modules::ca::{
    application::{
        commands::{
            ClosePollCommand, CreatePollCommand, VotePollCommand, execute_close_poll,
            execute_create_poll, execute_vote_poll,
        },
        error::PollApplicationError,
        ports::{DefaultAcademicCenterAuthorization, PollRepository},
        queries::{GetPollQuery, ListPollsQuery, execute_get_poll, execute_list_polls},
    },
    domain::entities::{AcademicCenter, AcademicCenterStatus, PollStatus},
    infrastructure::repository::{InMemoryAcademicCenterRepository, InMemoryPollRepository},
};

fn create_test_ca(id: Uuid, manager_id: Uuid, status: AcademicCenterStatus) -> AcademicCenter {
    AcademicCenter::reconstitute(
        id,
        Uuid::new_v4(),
        Some(manager_id),
        "Centro Acadêmico de Redes".to_string(),
        Some("CAER".to_string()),
        "Descrição institucional do CAER".to_string(),
        status,
        Utc::now() - Duration::days(10),
        Utc::now() - Duration::days(10),
    )
    .expect("valid academic center")
}

#[tokio::test]
async fn should_create_poll_successfully_by_authorized_manager() {
    let ca_repo = InMemoryAcademicCenterRepository::new();
    let poll_repo = InMemoryPollRepository::new();
    let auth_port = DefaultAcademicCenterAuthorization;

    let ca_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    ca_repo.insert(create_test_ca(
        ca_id,
        manager_id,
        AcademicCenterStatus::Active,
    ));

    let now = Utc::now();
    let cmd = CreatePollCommand {
        academic_center_id: ca_id,
        created_by_id: manager_id,
        title: "Ampliação do horário dos laboratórios".to_string(),
        description: Some("Manter laboratórios abertos até as 22h".to_string()),
        options: vec![
            "Apoio a proposta".to_string(),
            "Não apoio a proposta".to_string(),
        ],
        starts_at: now,
        ends_at: Some(now + Duration::days(7)),
        target_audience: Some("all_students".to_string()),
        vote_modality: Some("identified".to_string()),
    };

    let poll = execute_create_poll(&ca_repo, &poll_repo, &auth_port, cmd)
        .await
        .expect("should create poll");

    assert_eq!(poll.academic_center_id(), ca_id);
    assert_eq!(poll.created_by_id(), manager_id);
    assert_eq!(poll.title(), "Ampliação do horário dos laboratórios");
    assert_eq!(poll.status(), PollStatus::Open);
    assert_eq!(poll.options().len(), 2);
    assert_eq!(poll.options()[0].order(), 1);
    assert_eq!(poll.options()[1].order(), 2);

    // Verifica que foi salvo no repositório
    let persisted = poll_repo
        .find_by_id(poll.id())
        .await
        .expect("query ok")
        .expect("poll exists");
    assert_eq!(persisted.id(), poll.id());
}

#[tokio::test]
async fn should_fail_create_poll_when_user_not_permitted() {
    let ca_repo = InMemoryAcademicCenterRepository::new();
    let poll_repo = InMemoryPollRepository::new();
    let auth_port = DefaultAcademicCenterAuthorization;

    let ca_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    ca_repo.insert(create_test_ca(
        ca_id,
        manager_id,
        AcademicCenterStatus::Active,
    ));

    let cmd = CreatePollCommand {
        academic_center_id: ca_id,
        created_by_id: other_user_id, // Usuário não autorizado
        title: "Consulta não autorizada".to_string(),
        description: None,
        options: vec!["Opção 1".to_string(), "Opção 2".to_string()],
        starts_at: Utc::now(),
        ends_at: None,
        target_audience: None,
        vote_modality: None,
    };

    let err = execute_create_poll(&ca_repo, &poll_repo, &auth_port, cmd.clone())
        .await
        .expect_err("should fail");

    assert!(matches!(
        err,
        PollApplicationError::UserNotPermittedInCa { .. }
    ));
}

#[tokio::test]
async fn should_fail_create_poll_when_ca_inactive_or_not_found() {
    let ca_repo = InMemoryAcademicCenterRepository::new();
    let poll_repo = InMemoryPollRepository::new();
    let auth_port = DefaultAcademicCenterAuthorization;

    let ca_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    ca_repo.insert(create_test_ca(
        ca_id,
        manager_id,
        AcademicCenterStatus::Archived,
    ));

    let cmd = CreatePollCommand {
        academic_center_id: ca_id,
        created_by_id: manager_id,
        title: "Consulta em CA arquivado".to_string(),
        description: None,
        options: vec!["A".to_string(), "B".to_string()],
        starts_at: Utc::now(),
        ends_at: None,
        target_audience: None,
        vote_modality: None,
    };

    let err = execute_create_poll(&ca_repo, &poll_repo, &auth_port, cmd.clone())
        .await
        .expect_err("should fail");
    assert!(matches!(
        err,
        PollApplicationError::AcademicCenterInactive(..)
    ));

    // CA inexistente
    let fake_ca_id = Uuid::new_v4();
    let mut cmd_fake = cmd.clone();
    cmd_fake.academic_center_id = fake_ca_id;
    let err_not_found = execute_create_poll(&ca_repo, &poll_repo, &auth_port, cmd_fake)
        .await
        .expect_err("should fail");
    assert!(matches!(
        err_not_found,
        PollApplicationError::AcademicCenterNotFound(..)
    ));
}

#[tokio::test]
async fn should_fail_create_poll_with_invalid_options_or_dates() {
    let ca_repo = InMemoryAcademicCenterRepository::new();
    let poll_repo = InMemoryPollRepository::new();
    let auth_port = DefaultAcademicCenterAuthorization;

    let ca_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    ca_repo.insert(create_test_ca(
        ca_id,
        manager_id,
        AcademicCenterStatus::Active,
    ));

    // Menos de 2 opções
    let cmd_few = CreatePollCommand {
        academic_center_id: ca_id,
        created_by_id: manager_id,
        title: "Opção única".to_string(),
        description: None,
        options: vec!["Apenas uma".to_string()],
        starts_at: Utc::now(),
        ends_at: None,
        target_audience: None,
        vote_modality: None,
    };
    let err_few = execute_create_poll(&ca_repo, &poll_repo, &auth_port, cmd_few)
        .await
        .expect_err("should fail");
    assert!(matches!(
        err_few,
        PollApplicationError::InvalidPollOptions(..)
    ));

    // Opções duplicadas
    let cmd_dup = CreatePollCommand {
        academic_center_id: ca_id,
        created_by_id: manager_id,
        title: "Opções iguais".to_string(),
        description: None,
        options: vec!["Sim".to_string(), "sim".to_string()],
        starts_at: Utc::now(),
        ends_at: None,
        target_audience: None,
        vote_modality: None,
    };
    let err_dup = execute_create_poll(&ca_repo, &poll_repo, &auth_port, cmd_dup)
        .await
        .expect_err("should fail");
    assert!(matches!(
        err_dup,
        PollApplicationError::InvalidPollOptions(..)
    ));

    // Datas invertidas
    let now = Utc::now();
    let cmd_dates = CreatePollCommand {
        academic_center_id: ca_id,
        created_by_id: manager_id,
        title: "Datas invertidas".to_string(),
        description: None,
        options: vec!["A".to_string(), "B".to_string()],
        starts_at: now,
        ends_at: Some(now - Duration::days(1)),
        target_audience: None,
        vote_modality: None,
    };
    let err_dates = execute_create_poll(&ca_repo, &poll_repo, &auth_port, cmd_dates)
        .await
        .expect_err("should fail");
    assert!(matches!(
        err_dates,
        PollApplicationError::InvalidPollDates(..)
    ));
}

#[tokio::test]
async fn should_vote_successfully_and_prevent_duplicate_vote() {
    let ca_repo = InMemoryAcademicCenterRepository::new();
    let poll_repo = InMemoryPollRepository::new();
    let auth_port = DefaultAcademicCenterAuthorization;

    let ca_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    ca_repo.insert(create_test_ca(
        ca_id,
        manager_id,
        AcademicCenterStatus::Active,
    ));

    let now = Utc::now();
    let poll = execute_create_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        CreatePollCommand {
            academic_center_id: ca_id,
            created_by_id: manager_id,
            title: "Enquete de Teste".to_string(),
            description: None,
            options: vec!["Sim".to_string(), "Não".to_string()],
            starts_at: now - Duration::hours(1),
            ends_at: Some(now + Duration::days(1)),
            target_audience: None,
            vote_modality: None,
        },
    )
    .await
    .unwrap();

    let student_1 = Uuid::new_v4();
    let student_2 = Uuid::new_v4();
    let opt_1 = poll.options()[0].id();
    let opt_2 = poll.options()[1].id();

    // Primeiro voto: estudante 1
    let vote_1 = execute_vote_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        VotePollCommand {
            academic_center_id: ca_id,
            poll_id: poll.id(),
            option_id: opt_1,
            user_id: student_1,
            voted_at: now,
        },
    )
    .await
    .expect("vote 1 should succeed");

    assert_eq!(vote_1.poll_id(), poll.id());
    assert_eq!(vote_1.option_id(), opt_1);
    assert_eq!(vote_1.user_id(), student_1);

    // Segundo voto: estudante 2 na opção 2
    let vote_2 = execute_vote_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        VotePollCommand {
            academic_center_id: ca_id,
            poll_id: poll.id(),
            option_id: opt_2,
            user_id: student_2,
            voted_at: now,
        },
    )
    .await
    .expect("vote 2 should succeed");
    assert_eq!(vote_2.user_id(), student_2);

    // Tentativa de voto duplicado: estudante 1 tenta votar de novo
    let err_dup = execute_vote_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        VotePollCommand {
            academic_center_id: ca_id,
            poll_id: poll.id(),
            option_id: opt_2,
            user_id: student_1,
            voted_at: now,
        },
    )
    .await
    .expect_err("should reject duplicate vote");

    assert!(matches!(
        err_dup,
        PollApplicationError::DuplicateVote { .. }
    ));

    // Consulta de resultados
    let (_, results) = execute_get_poll(
        &ca_repo,
        &poll_repo,
        GetPollQuery {
            academic_center_id: ca_id,
            poll_id: poll.id(),
        },
    )
    .await
    .expect("should get poll with results");

    assert_eq!(results.total_votes, 2);
    let r1 = results.options.iter().find(|o| o.id == opt_1).unwrap();
    assert_eq!(r1.vote_count, 1);
    assert_eq!(r1.percentage, 50.0);
    let r2 = results.options.iter().find(|o| o.id == opt_2).unwrap();
    assert_eq!(r2.vote_count, 1);
    assert_eq!(r2.percentage, 50.0);
}

#[tokio::test]
async fn should_enforce_ca_isolation_strictly() {
    let ca_repo = InMemoryAcademicCenterRepository::new();
    let poll_repo = InMemoryPollRepository::new();
    let auth_port = DefaultAcademicCenterAuthorization;

    let ca_1_id = Uuid::new_v4();
    let ca_2_id = Uuid::new_v4();
    let manager_1 = Uuid::new_v4();
    let manager_2 = Uuid::new_v4();

    ca_repo.insert(create_test_ca(
        ca_1_id,
        manager_1,
        AcademicCenterStatus::Active,
    ));
    ca_repo.insert(create_test_ca(
        ca_2_id,
        manager_2,
        AcademicCenterStatus::Active,
    ));

    let now = Utc::now();
    let poll_ca1 = execute_create_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        CreatePollCommand {
            academic_center_id: ca_1_id,
            created_by_id: manager_1,
            title: "Consulta CA 1".to_string(),
            description: None,
            options: vec!["Op 1".to_string(), "Op 2".to_string()],
            starts_at: now - Duration::hours(1),
            ends_at: None,
            target_audience: None,
            vote_modality: None,
        },
    )
    .await
    .unwrap();

    let student = Uuid::new_v4();

    // Tentativa de votar na consulta do CA 1 usando o caminho do CA 2 -> Deve retornar 404 (PollNotFound)
    let err_iso_vote = execute_vote_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        VotePollCommand {
            academic_center_id: ca_2_id, // CA diferente da consulta
            poll_id: poll_ca1.id(),
            option_id: poll_ca1.options()[0].id(),
            user_id: student,
            voted_at: now,
        },
    )
    .await
    .expect_err("should isolate CA");

    assert!(matches!(
        err_iso_vote,
        PollApplicationError::PollNotFound(..)
    ));

    // Tentativa de obter detalhes da consulta do CA 1 usando caminho do CA 2
    let err_iso_get = execute_get_poll(
        &ca_repo,
        &poll_repo,
        GetPollQuery {
            academic_center_id: ca_2_id,
            poll_id: poll_ca1.id(),
        },
    )
    .await
    .expect_err("should isolate CA");

    assert!(matches!(
        err_iso_get,
        PollApplicationError::PollNotFound(..)
    ));

    // Tentativa de fechar consulta do CA 1 pelo gestor do CA 2
    let err_iso_close = execute_close_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        ClosePollCommand {
            academic_center_id: ca_2_id,
            poll_id: poll_ca1.id(),
            closed_by_id: manager_2,
            closed_at: now,
        },
    )
    .await
    .expect_err("should isolate CA");

    assert!(matches!(
        err_iso_close,
        PollApplicationError::PollNotFound(..)
    ));
}

#[tokio::test]
async fn should_close_poll_and_block_subsequent_votes() {
    let ca_repo = InMemoryAcademicCenterRepository::new();
    let poll_repo = InMemoryPollRepository::new();
    let auth_port = DefaultAcademicCenterAuthorization;

    let ca_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    ca_repo.insert(create_test_ca(
        ca_id,
        manager_id,
        AcademicCenterStatus::Active,
    ));

    let now = Utc::now();
    let poll = execute_create_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        CreatePollCommand {
            academic_center_id: ca_id,
            created_by_id: manager_id,
            title: "Consulta a fechar".to_string(),
            description: None,
            options: vec!["Sim".to_string(), "Não".to_string()],
            starts_at: now - Duration::hours(1),
            ends_at: None,
            target_audience: None,
            vote_modality: None,
        },
    )
    .await
    .unwrap();

    let student = Uuid::new_v4();

    // Voto antes de fechar -> Sucesso
    execute_vote_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        VotePollCommand {
            academic_center_id: ca_id,
            poll_id: poll.id(),
            option_id: poll.options()[0].id(),
            user_id: student,
            voted_at: now,
        },
    )
    .await
    .expect("vote before close ok");

    // Gestor encerra a consulta
    let closed_poll = execute_close_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        ClosePollCommand {
            academic_center_id: ca_id,
            poll_id: poll.id(),
            closed_by_id: manager_id,
            closed_at: now + Duration::minutes(5),
        },
    )
    .await
    .expect("should close poll");

    assert_eq!(closed_poll.status(), PollStatus::Closed);

    // Tentativa de novo voto após encerramento -> PollClosed
    let student_2 = Uuid::new_v4();
    let err_after_close = execute_vote_poll(
        &ca_repo,
        &poll_repo,
        &auth_port,
        VotePollCommand {
            academic_center_id: ca_id,
            poll_id: poll.id(),
            option_id: poll.options()[1].id(),
            user_id: student_2,
            voted_at: now + Duration::minutes(10),
        },
    )
    .await
    .expect_err("should block vote after close");

    assert!(matches!(err_after_close, PollApplicationError::PollClosed));
}

#[tokio::test]
async fn should_list_polls_of_academic_center() {
    let ca_repo = InMemoryAcademicCenterRepository::new();
    let poll_repo = InMemoryPollRepository::new();
    let auth_port = DefaultAcademicCenterAuthorization;

    let ca_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    ca_repo.insert(create_test_ca(
        ca_id,
        manager_id,
        AcademicCenterStatus::Active,
    ));

    let now = Utc::now();
    for i in 1..=3 {
        execute_create_poll(
            &ca_repo,
            &poll_repo,
            &auth_port,
            CreatePollCommand {
                academic_center_id: ca_id,
                created_by_id: manager_id,
                title: format!("Consulta {i}"),
                description: None,
                options: vec!["A".to_string(), "B".to_string()],
                starts_at: now,
                ends_at: None,
                target_audience: None,
                vote_modality: None,
            },
        )
        .await
        .unwrap();
    }

    let list = execute_list_polls(
        &ca_repo,
        &poll_repo,
        ListPollsQuery {
            academic_center_id: ca_id,
        },
    )
    .await
    .expect("should list polls");

    assert_eq!(list.len(), 3);
}
