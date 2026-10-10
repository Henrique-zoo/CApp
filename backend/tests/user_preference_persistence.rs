//! Testes de integração demonstrando a persistência e regras de domínio da preferência de CA favorito (Issue #44).
//!
//! Valida a integridade da entrega incremental I01 ("Acesso e meu CA"), comprovando:
//! 1. Persistência de preferência utilizando os CAs didáticos da UnB (CAER / CACC).
//! 2. Desacoplamento estrito entre conveniência de navegação e direitos acadêmicos/políticos.
//! 3. Integridade referencial das migrations SQLx e da fixture determinística.

use backend::{
    domain::{
        academic_center::{AcademicCenter, CACC_DIDACTIC_ID, CAER_DIDACTIC_ID},
        app_user::{AppUser, CALOURO_DIDACTIC_ID, VETERANO_1_DIDACTIC_ID},
        user_preference::UserPreference,
    },
    modules::identity::{
        domain::rules::{
            assert_navigation_only_scope, has_academic_enrollment_via_preference,
            has_administrative_privileges_via_preference, is_eligible_to_vote_via_preference,
        },
        infrastructure::repository::{InMemoryUserPreferenceRepository, UserPreferenceRepository},
    },
};

#[tokio::test]
async fn should_persist_and_switch_favorite_ca_between_didactic_cas() {
    let repo = InMemoryUserPreferenceRepository::new();

    let veterano = AppUser::veterano_didactic();
    let caer = AcademicCenter::caer_didactic();
    let cacc = AcademicCenter::cacc_didactic();

    // 1. Persistência inicial: Veterano seleciona CAER como favorito
    let mut pref =
        UserPreference::new(veterano.id, Some(caer.id)).expect("preferência válida com CAER");
    repo.save(&pref)
        .await
        .expect("deve salvar preferência inicial");

    let saved = repo
        .find_by_user_id(veterano.id)
        .await
        .expect("consulta bem-sucedida")
        .expect("preferência deve existir");

    assert_eq!(saved.user_id(), VETERANO_1_DIDACTIC_ID);
    assert_eq!(saved.favorite_ca_id(), Some(CAER_DIDACTIC_ID));
    assert!(saved.is_favorite(&CAER_DIDACTIC_ID));
    assert!(!saved.is_favorite(&CACC_DIDACTIC_ID));

    // 2. Regra estrita de domínio: seleção não confere privilégios nem vínculo acadêmico
    assert!(saved.is_navigation_convenience_only());
    assert!(!saved.confers_academic_enrollment());
    assert!(!saved.confers_electoral_eligibility());
    assert!(!saved.confers_administrative_permissions());
    assert!(assert_navigation_only_scope(&saved).is_ok());
    assert!(!is_eligible_to_vote_via_preference(&saved, &caer.id));
    assert!(!has_administrative_privileges_via_preference(
        &saved, &caer.id
    ));
    assert!(!has_academic_enrollment_via_preference(&saved, &caer.id));

    // 3. Alternância para CACC como contexto favorito
    pref.set_favorite_ca(cacc.id)
        .expect("deve atualizar para CACC");
    repo.save(&pref)
        .await
        .expect("deve persistir alteração de CA favorito");

    let updated = repo
        .find_by_user_id(veterano.id)
        .await
        .expect("consulta pós-atualização")
        .expect("preferência deve existir");

    assert_eq!(updated.favorite_ca_id(), Some(CACC_DIDACTIC_ID));
    assert!(updated.is_favorite(&CACC_DIDACTIC_ID));
    assert!(!updated.is_favorite(&CAER_DIDACTIC_ID));

    // 4. Limpeza da preferência de navegação (contexto neutro/padrão)
    pref.clear_favorite_ca();
    repo.save(&pref)
        .await
        .expect("deve persistir limpeza do favorito");

    let cleared = repo
        .find_by_user_id(veterano.id)
        .await
        .expect("consulta pós-limpeza")
        .expect("preferência ainda existe com valor nulo");

    assert_eq!(cleared.favorite_ca_id(), None);
    assert!(!cleared.has_favorite_ca());
}

#[tokio::test]
async fn should_load_and_validate_didactic_fixture_data() {
    let repo = InMemoryUserPreferenceRepository::with_didactic_data();

    // Veterano UnB com CAER favorito
    let veterano_pref = repo
        .find_by_user_id(VETERANO_1_DIDACTIC_ID)
        .await
        .expect("busca veterano")
        .expect("deve conter preferência didática do veterano");

    assert_eq!(veterano_pref.favorite_ca_id(), Some(CAER_DIDACTIC_ID));
    assert!(veterano_pref.is_favorite(&CAER_DIDACTIC_ID));

    // Calouro UnB com CACC favorito
    let calouro_pref = repo
        .find_by_user_id(CALOURO_DIDACTIC_ID)
        .await
        .expect("busca calouro")
        .expect("deve conter preferência didática do calouro");

    assert_eq!(calouro_pref.favorite_ca_id(), Some(CACC_DIDACTIC_ID));
    assert!(calouro_pref.is_favorite(&CACC_DIDACTIC_ID));

    // Desacoplamento assegurado em ambos os casos
    assert!(!veterano_pref.confers_electoral_eligibility());
    assert!(!calouro_pref.confers_administrative_permissions());
}

#[test]
fn should_verify_migration_sql_foreign_keys_and_constraints() {
    let migration_user_preference =
        include_str!("../migrations/20261008000003_create_user_preference_table.sql");

    // Verifica integridade referencial com chaves estrangeiras
    assert!(
        migration_user_preference
            .contains("FOREIGN KEY (user_id) REFERENCES app_user(id) ON DELETE CASCADE"),
        "Migration deve declarar chave estrangeira para app_user com ON DELETE CASCADE"
    );
    assert!(
        migration_user_preference.contains(
            "FOREIGN KEY (favorite_ca_id) REFERENCES academic_center(id) ON DELETE SET NULL"
        ),
        "Migration deve declarar chave estrangeira para academic_center com ON DELETE SET NULL"
    );
    assert!(
        migration_user_preference.contains("user_id UUID PRIMARY KEY"),
        "Migration deve definir user_id como PRIMARY KEY para garantir unicidade por usuário"
    );
    assert!(
        migration_user_preference.contains("favorite_ca_id UUID NULL"),
        "Migration deve permitir favorite_ca_id nulo para usuários sem preferência definida"
    );
    assert!(
        migration_user_preference.contains("idx_user_preference_favorite_ca_id"),
        "Migration deve criar índice para busca rápida por favorite_ca_id"
    );
}

#[test]
fn should_verify_api_bdd_fixture_contains_didactic_data() {
    let fixture_sql = include_str!("fixtures/api_bdd.sql");

    // Verifica CAs didáticos da UnB
    assert!(fixture_sql.contains("Centro Acadêmico de Engenharia de Redes"));
    assert!(fixture_sql.contains("CAER"));
    assert!(fixture_sql.contains("018f0000-0000-7000-8000-000000000001"));
    assert!(fixture_sql.contains("Centro Acadêmico de Ciência da Computação"));
    assert!(fixture_sql.contains("CACC"));
    assert!(fixture_sql.contains("018f0000-0000-7000-8000-000000000002"));

    // Verifica usuários didáticos da UnB
    assert!(fixture_sql.contains("231012345@aluno.unb.br"));
    assert!(fixture_sql.contains("262001234@aluno.unb.br"));

    // Verifica preferências didáticas da UnB
    assert!(fixture_sql.contains("018f0000-0000-7000-8000-000000000011"));
    assert!(fixture_sql.contains("018f0000-0000-7000-8000-000000000013"));
}
