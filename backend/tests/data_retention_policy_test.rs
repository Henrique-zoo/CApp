//! Testes automatizados das políticas de retenção, minimização e descarte seguro de dados.
//!
//! Valida cálculos de retenção, expiração de registros, salvaguardas de sigilo eleitoral,
//! integridade referencial e sanitização de dados discentes com base em dados sintéticos homologados.

use backend::modules::identity::domain::{
    entities::{IdentityRetentionStatus, StudentIdentity},
    rules::{anonymize_identity, is_grace_period_expired, validate_unb_single_institution},
};
use backend::shared::retention::{
    APPROVED_MATRICULAS, AnonymousBallot, DataCategory, DisposalMethod, DisposalStrategy,
    ElectoralAuditStatus, ElectoralPrivacyGuardian, FORBIDDEN_MATRICULA, LegalBasis, PiiSanitizer,
    ReferentialIntegrityGuardian, RetentionError, RetentionHold, RetentionPeriod,
    RetentionSchedule, SensitivityLevel, StudentRecord, TurnoutRecord, UNB_EMAIL_DOMAIN,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use uuid::Uuid;

/// Data-base sintética fixa para testes determinísticos de retenção.
fn synthetic_base_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 15, 12, 0, 0).unwrap()
}

#[test]
fn test_data_classification_matrix_and_legal_bases() {
    let categories = [
        DataCategory::StudentPersonalData,
        DataCategory::ManagementMandate,
        DataCategory::FinancialAccountability,
        DataCategory::ElectoralTurnout,
        DataCategory::ElectoralBallot,
        DataCategory::ElectoralAudit,
        DataCategory::SessionAndToken,
        DataCategory::AuditLog,
        DataCategory::Telemetry,
    ];

    for category in categories {
        let period = RetentionSchedule::period_for(&category);
        let sensitivity = RetentionSchedule::sensitivity_for(&category);
        let legal_basis = RetentionSchedule::legal_basis_for(&category);
        let disposal_method = RetentionSchedule::disposal_method_for(&category);

        assert!(!category.name().is_empty());
        assert!(!legal_basis.description().is_empty());

        match category {
            DataCategory::StudentPersonalData => {
                assert_eq!(period, RetentionPeriod::ActivePlusToleranceDays(180));
                assert_eq!(sensitivity, SensitivityLevel::Restricted);
                assert_eq!(legal_basis, LegalBasis::ContractExecution);
                assert_eq!(disposal_method, DisposalMethod::IrreversibleAnonymization);
            }
            DataCategory::ManagementMandate => {
                assert_eq!(period, RetentionPeriod::Days(1826));
                assert_eq!(sensitivity, SensitivityLevel::Internal);
                assert_eq!(legal_basis, LegalBasis::RegularExerciseOfRights);
                assert_eq!(disposal_method, DisposalMethod::IrreversibleAnonymization);
            }
            DataCategory::FinancialAccountability => {
                assert_eq!(period, RetentionPeriod::Days(1826));
                assert_eq!(sensitivity, SensitivityLevel::Restricted);
                assert_eq!(legal_basis, LegalBasis::RegularExerciseOfRights);
                assert_eq!(disposal_method, DisposalMethod::IrreversibleAnonymization);
            }
            DataCategory::ElectoralTurnout => {
                assert_eq!(period, RetentionPeriod::Days(180));
                assert_eq!(sensitivity, SensitivityLevel::Restricted);
                assert_eq!(legal_basis, LegalBasis::ContractExecution);
                assert_eq!(disposal_method, DisposalMethod::PhysicalPurge);
            }
            DataCategory::ElectoralBallot => {
                assert_eq!(period, RetentionPeriod::Days(90));
                assert_eq!(sensitivity, SensitivityLevel::CriticalSensitive);
                assert_eq!(legal_basis, LegalBasis::ElectoralStatutoryDuty);
                assert_eq!(disposal_method, DisposalMethod::CryptoShredding);
            }
            DataCategory::ElectoralAudit => {
                assert_eq!(period, RetentionPeriod::Days(30));
                assert_eq!(sensitivity, SensitivityLevel::CriticalSensitive);
                assert_eq!(legal_basis, LegalBasis::RegularExerciseOfRights);
                assert_eq!(disposal_method, DisposalMethod::PhysicalPurge);
            }
            DataCategory::SessionAndToken => {
                assert_eq!(period, RetentionPeriod::Days(30));
                assert_eq!(sensitivity, SensitivityLevel::CriticalSensitive);
                assert_eq!(legal_basis, LegalBasis::ContractExecution);
                assert_eq!(disposal_method, DisposalMethod::PhysicalPurge);
            }
            DataCategory::AuditLog => {
                assert_eq!(period, RetentionPeriod::Days(180));
                assert_eq!(sensitivity, SensitivityLevel::Restricted);
                assert_eq!(legal_basis, LegalBasis::LegalOrRegulatoryObligation);
                assert_eq!(disposal_method, DisposalMethod::PhysicalPurge);
            }
            DataCategory::Telemetry => {
                assert_eq!(period, RetentionPeriod::Days(30));
                assert_eq!(sensitivity, SensitivityLevel::Internal);
                assert_eq!(legal_basis, LegalBasis::LegitimateInterest);
                assert_eq!(disposal_method, DisposalMethod::PhysicalPurge);
            }
        }
    }
}

#[test]
fn test_retention_calculation_and_record_expiration() {
    let base_time = synthetic_base_time();

    // 1. Logs de auditoria (Marco Civil: 180 dias)
    let audit_exp =
        RetentionSchedule::calculate_expiration(&DataCategory::AuditLog, base_time).unwrap();
    assert_eq!(audit_exp, base_time + Duration::days(180));

    assert!(!RetentionSchedule::is_expired(
        &DataCategory::AuditLog,
        base_time,
        base_time + Duration::days(179)
    ));
    assert!(RetentionSchedule::is_expired(
        &DataCategory::AuditLog,
        base_time,
        base_time + Duration::days(180)
    ));
    assert!(RetentionSchedule::is_expired(
        &DataCategory::AuditLog,
        base_time,
        base_time + Duration::days(181)
    ));

    // 2. Histórico de mandato de CA (5 anos = 1826 dias)
    let mandate_exp =
        RetentionSchedule::calculate_expiration(&DataCategory::ManagementMandate, base_time)
            .unwrap();
    assert_eq!(mandate_exp, base_time + Duration::days(1826));

    assert!(!RetentionSchedule::is_expired(
        &DataCategory::ManagementMandate,
        base_time,
        base_time + Duration::days(1825)
    ));
    assert!(RetentionSchedule::is_expired(
        &DataCategory::ManagementMandate,
        base_time,
        base_time + Duration::days(1826)
    ));

    // 3. Trilha efêmera de auditoria eleitoral (30 dias)
    let electoral_audit_exp =
        RetentionSchedule::calculate_expiration(&DataCategory::ElectoralAudit, base_time).unwrap();
    assert_eq!(electoral_audit_exp, base_time + Duration::days(30));
    assert!(RetentionSchedule::is_expired(
        &DataCategory::ElectoralAudit,
        base_time,
        base_time + Duration::days(31)
    ));
}

#[test]
fn test_masking_approved_synthetic_matriculas() {
    // Uso exclusivo das matrículas didáticas homologadas
    for matricula in &APPROVED_MATRICULAS {
        let masked = PiiSanitizer::mask_matricula(matricula).unwrap();
        assert_eq!(masked.len(), 9);
        assert!(masked.contains("***"));
        assert_eq!(&masked[..4], &matricula[..4]);
        assert_eq!(&masked[7..], &matricula[7..]);
    }

    // Exemplos específicos esperados
    assert_eq!(
        PiiSanitizer::mask_matricula("241098765").unwrap(),
        "2410***65"
    );
    assert_eq!(
        PiiSanitizer::mask_matricula("242011111").unwrap(),
        "2420***11"
    );
    assert_eq!(
        PiiSanitizer::mask_matricula("231012345").unwrap(),
        "2310***45"
    );
    assert_eq!(
        PiiSanitizer::mask_matricula("262001234").unwrap(),
        "2620***34"
    );
}

#[test]
fn test_masking_unb_institutional_emails() {
    for matricula in &APPROVED_MATRICULAS {
        let email = format!("{matricula}{UNB_EMAIL_DOMAIN}");
        let masked_email = PiiSanitizer::mask_email(&email).unwrap();

        assert!(masked_email.ends_with(UNB_EMAIL_DOMAIN));
        assert!(masked_email.contains("***"));
    }

    assert_eq!(
        PiiSanitizer::mask_email("241098765@aluno.unb.br").unwrap(),
        "2410***65@aluno.unb.br"
    );
    assert_eq!(
        PiiSanitizer::mask_email("231012345@aluno.unb.br").unwrap(),
        "2310***45@aluno.unb.br"
    );

    // Rejeição estrita de domínio não institucional
    let non_unb = "estudante@gmail.com";
    assert_eq!(
        PiiSanitizer::mask_email(non_unb),
        Err(RetentionError::InvalidUnbEmail(non_unb.to_string()))
    );
}

#[test]
fn test_strict_prohibition_of_forbidden_matricula_242003979() {
    // A matrícula 242003979 é terminantemente proibida
    assert_eq!(
        PiiSanitizer::validate_matricula(FORBIDDEN_MATRICULA),
        Err(RetentionError::ForbiddenMatricula(
            FORBIDDEN_MATRICULA.to_string()
        ))
    );

    assert_eq!(
        PiiSanitizer::mask_matricula(FORBIDDEN_MATRICULA),
        Err(RetentionError::ForbiddenMatricula(
            FORBIDDEN_MATRICULA.to_string()
        ))
    );

    let forbidden_email = format!("{FORBIDDEN_MATRICULA}{UNB_EMAIL_DOMAIN}");
    assert_eq!(
        PiiSanitizer::validate_unb_email(&forbidden_email),
        Err(RetentionError::ForbiddenMatricula(
            FORBIDDEN_MATRICULA.to_string()
        ))
    );

    // Tentativa de criar perfil discente com a matrícula proibida é rejeitada
    let attempt = StudentRecord::new(
        Uuid::new_v4(),
        FORBIDDEN_MATRICULA,
        forbidden_email,
        "Usuário Proibido",
    );
    assert_eq!(
        attempt,
        Err(RetentionError::ForbiddenMatricula(
            FORBIDDEN_MATRICULA.to_string()
        ))
    );
}

#[test]
fn test_unb_single_institution_invariance_prohibits_institution_id() {
    // Invariante: O CApp é exclusivo da UnB; institution_id é estritamente proibido
    assert_eq!(
        PiiSanitizer::validate_no_institution_id(Some("unb-instituicao")),
        Err(RetentionError::InstitutionIdProhibited)
    );

    assert_eq!(
        validate_unb_single_institution(Some(Uuid::new_v4())),
        Err(RetentionError::InstitutionIdProhibited)
    );

    assert!(PiiSanitizer::validate_no_institution_id::<String>(None).is_ok());
    assert!(validate_unb_single_institution::<Uuid>(None).is_ok());
}

#[test]
fn test_log_sanitization_removes_pii_and_bearer_tokens() {
    let raw_log = "WARN: Falha de autenticação do discente 241098765 (241098765@aluno.unb.br) usando Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.xyz";
    let sanitized_log = PiiSanitizer::sanitize_log(raw_log);

    // Matrícula e e-mail devem estar mascarados
    assert!(sanitized_log.contains("2410***65"));
    assert!(sanitized_log.contains("2410***65@aluno.unb.br"));
    assert!(!sanitized_log.contains("241098765@aluno.unb.br"));

    // Token Bearer deve estar redigido
    assert!(sanitized_log.contains("Bearer [REDACTED_BEARER_TOKEN]"));
    assert!(!sanitized_log.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
}

#[test]
fn test_student_profile_irreversible_anonymization() {
    let user_id = Uuid::new_v4();
    let mut student = StudentRecord::new(
        user_id,
        "241098765",
        "241098765@aluno.unb.br",
        "Estudante Didático de Engenharia",
    )
    .unwrap();

    student.phone = Some("+5561999999999".to_string());
    student.photo_url = Some("https://storage.capp.unb.br/perfil.jpg".to_string());

    assert!(!student.is_anonymized);

    // Executa anonimização irreversível
    PiiSanitizer::anonymize_student_record(&mut student).unwrap();

    assert!(student.is_anonymized);
    assert_eq!(student.name, "Estudante Anonimizado");
    assert_eq!(
        student.email,
        format!("anonimizado-{user_id}@anonimo.capp.unb.br")
    );
    assert!(student.phone.is_none());
    assert!(student.photo_url.is_none());
    assert!(student.matricula.contains("***"));
}

#[test]
fn test_electoral_secret_ballot_isolation_and_quantization() {
    let election_id = Uuid::new_v4();
    let voter_user_id = Uuid::new_v4();
    let candidate_id = Uuid::new_v4();
    let ballot_id = Uuid::new_v4();
    let base_time = synthetic_base_time();

    // 1. Registro de comparecimento (Turnout) possui user_id
    let turnout = TurnoutRecord {
        turnout_id: Uuid::new_v4(),
        user_id: voter_user_id,
        election_id,
        voted_at: base_time,
        receipt_hash: "sha256_mock_comprovante_eleitoral".to_string(),
    };
    assert_eq!(turnout.user_id, voter_user_id);

    // 2. Cédula eleitoral anônima (Ballot) NÃO possui user_id
    let ballot = AnonymousBallot {
        ballot_id,
        election_id,
        candidate_chapa_id: candidate_id,
        encrypted_vote_payload: vec![0xDE, 0xAD, 0xBE, 0xEF],
        batch_timestamp: ElectoralPrivacyGuardian::quantize_timestamp(base_time, 15),
    };

    // A cédula deve ser aceita se desvinculada do eleitor
    assert!(
        ElectoralPrivacyGuardian::validate_ballot_isolation(&ballot, Some(voter_user_id)).is_ok()
    );

    // Se houver tentativa de igualar ballot_id com voter_user_id, rejeita por violação de sigilo
    assert_eq!(
        ElectoralPrivacyGuardian::validate_ballot_isolation(&ballot, Some(ballot_id)),
        Err(RetentionError::ElectoralSecretViolation(
            "O ballot_id não pode ser igual ao user_id".to_string()
        ))
    );

    // 3. Teste de quantização de horário para quebrar correlação temporal
    let vote_instant = Utc.with_ymd_and_hms(2026, 10, 15, 14, 27, 45).unwrap();
    let quantized = ElectoralPrivacyGuardian::quantize_timestamp(vote_instant, 15);
    // 14:27 arredonda para a janela de 14:15:00
    assert_eq!(
        quantized,
        Utc.with_ymd_and_hms(2026, 10, 15, 14, 15, 0).unwrap()
    );
}

#[test]
fn test_electoral_audit_lifecycle_progressive_disposal() {
    let homologated_at = synthetic_base_time();

    // Dia 15 pós-homologação: dentro da janela de 30 dias de impugnação
    let status_day_15 = ElectoralPrivacyGuardian::evaluate_audit_status(
        homologated_at,
        homologated_at + Duration::days(15),
    );
    assert_eq!(status_day_15, ElectoralAuditStatus::AppealWindowActive);

    // Dia 35 pós-homologação: fim da impugnação, trilha técnica de auditoria expurgada
    let status_day_35 = ElectoralPrivacyGuardian::evaluate_audit_status(
        homologated_at,
        homologated_at + Duration::days(35),
    );
    assert_eq!(status_day_35, ElectoralAuditStatus::AuditLogsPurged);

    // Dia 190 pós-homologação: comparecimento nominal purgado, apenas quórum consolidado
    let status_day_190 = ElectoralPrivacyGuardian::evaluate_audit_status(
        homologated_at,
        homologated_at + Duration::days(190),
    );
    assert_eq!(status_day_190, ElectoralAuditStatus::TurnoutRecordsPurged);
}

#[test]
fn test_referential_integrity_preserves_management_and_financial_history() {
    let user_id = Uuid::new_v4();
    let base_time = synthetic_base_time();

    // Caso A: Usuário foi diretor do CA com mandato encerrado há 2 anos (guarda de 5 anos obrigatória)
    let holds = vec![RetentionHold::ManagementMandate {
        term_id: Uuid::new_v4(),
        term_ended_at: base_time - Duration::days(730), // 2 anos atrás
    }];

    // Tentativa de exclusão física aos 2 anos deve ser bloqueada
    let disposal_strategy = ReferentialIntegrityGuardian::evaluate_user_disposal(&holds, base_time);
    assert_eq!(disposal_strategy, DisposalStrategy::AnonymizeInPlace);

    let auth_result =
        ReferentialIntegrityGuardian::authorize_physical_purge(user_id, &holds, base_time);
    assert!(matches!(
        auth_result,
        Err(RetentionError::ReferentialIntegrityBlocked(_))
    ));

    // Caso B: Aos 6 anos pós-mandato (após os 5 anos regulamentares = 1826 dias)
    let future_time = base_time + Duration::days(1827);
    let expired_strategy =
        ReferentialIntegrityGuardian::evaluate_user_disposal(&holds, future_time);
    assert_eq!(expired_strategy, DisposalStrategy::PhysicalPurge);

    let auth_future =
        ReferentialIntegrityGuardian::authorize_physical_purge(user_id, &holds, future_time);
    assert!(auth_future.is_ok());

    // Caso C: Usuário comum sem cargos e sem prestação de contas pendente
    let no_holds: Vec<RetentionHold> = Vec::new();
    assert_eq!(
        ReferentialIntegrityGuardian::evaluate_user_disposal(&no_holds, base_time),
        DisposalStrategy::PhysicalPurge
    );
    assert!(
        ReferentialIntegrityGuardian::authorize_physical_purge(user_id, &no_holds, base_time)
            .is_ok()
    );
}

#[test]
fn test_identity_domain_entity_and_grace_period_rules() {
    let user_id = Uuid::new_v4();
    let base_time = synthetic_base_time();

    let mut identity = StudentIdentity::new_active(
        user_id,
        "241098765",
        "241098765@aluno.unb.br",
        "Discente de Redes UnB",
    )
    .unwrap();

    assert_eq!(identity.retention_status, IdentityRetentionStatus::Active);

    // Simula desvinculação com início do período de tolerância recursal (180 dias)
    identity.retention_status = IdentityRetentionStatus::InactiveGracePeriod {
        unlinked_at: base_time,
    };

    // Aos 100 dias: dentro da tolerância
    assert!(!is_grace_period_expired(
        base_time,
        base_time + Duration::days(100)
    ));

    // Aos 181 dias: tolerância expirada, pronto para anonimização
    assert!(is_grace_period_expired(
        base_time,
        base_time + Duration::days(181)
    ));

    // Executa anonimização na entidade de domínio
    anonymize_identity(&mut identity, base_time + Duration::days(181)).unwrap();
    assert_eq!(identity.name, "Estudante Anonimizado");
    assert_eq!(
        identity.email,
        format!("anonimizado-{user_id}@anonimo.capp.unb.br")
    );
    assert_eq!(
        identity.retention_status,
        IdentityRetentionStatus::Anonymized {
            anonymized_at: base_time + Duration::days(181)
        }
    );
}
