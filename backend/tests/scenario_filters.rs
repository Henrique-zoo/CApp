//! Verifica a seleção de cenários a partir da herança real de tags Gherkin
//! e o vínculo completo entre as features automatizadas e os steps registrados.

#[allow(dead_code)]
mod step_definitions;
#[allow(dead_code)]
mod support;

use std::path::Path;

use cucumber::{World as _, gherkin::Feature};
use support::{AppWorld, scenario_filter};

/// Lê um cenário com ou sem regra e aplica o filtro usado pelo runner.
fn is_pending(source: &str) -> bool {
    let feature = Feature::parse(source, Default::default()).expect("Gherkin válido");
    let rule = feature.rules.first();
    let scenario = rule
        .map(|rule| &rule.scenarios[0])
        .unwrap_or_else(|| &feature.scenarios[0]);
    scenario_filter::is_pending_backend(&feature, rule, scenario)
}

/// Uma pendência na feature é herdada pelo cenário.
#[test]
fn inherits_backend_pending_from_feature() {
    assert!(is_pending(
        "@pending_backend\nFeature: Selection\n  @backend\n  Scenario: Pending\n    Given a step\n"
    ));
}

/// Uma pendência na regra é herdada pelo cenário.
#[test]
fn inherits_backend_pending_from_rule() {
    assert!(is_pending(
        "Feature: Selection\n  @pending_backend\n  Rule: Pending\n    @backend\n    Scenario: Pending\n      Given a step\n"
    ));
}

/// Uma pendência diretamente no cenário impede sua execução.
#[test]
fn recognizes_backend_pending_on_scenario() {
    assert!(is_pending(
        "Feature: Selection\n  @backend @pending_backend\n  Scenario: Pending\n    Given a step\n"
    ));
}

/// Um cenário pendente só no mobile continua elegível para o backend.
#[test]
fn mobile_pending_does_not_exclude_backend() {
    assert!(!is_pending(
        "@pending_mobile\nFeature: Selection\n  @backend\n  Scenario: Available\n    Given a step\n"
    ));
}

/// Cenários sem pendência continuam elegíveis para os demais filtros.
#[test]
fn scenarios_without_pending_tags_remain_available() {
    assert!(!is_pending(
        "Feature: Selection\n  @backend\n  Scenario: Available\n    Given a step\n"
    ));
}

/// Garante que `visualizacao_eventos.feature` e `inscricao_eventos.feature` estão habilitadas
/// para o backend (sem `@pending_backend`), mantêm `@pending_mobile`, e possuem todos os seus
/// steps registrados sem ambiguidade na coleção do Cucumber.
#[test]
fn visualizacao_and_inscricao_eventos_scenarios_are_enabled_and_all_steps_registered() {
    let features_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../features");
    let collection = AppWorld::collection();

    for (relative_path, expected_scenarios) in [
        ("visualizacao_eventos.feature", 6usize),
        ("inscricao_eventos.feature", 5usize),
    ] {
        let path = features_dir.join(relative_path);
        let feature = Feature::parse_path(&path, Default::default())
            .unwrap_or_else(|err| panic!("deve fazer parse de {relative_path}: {err}"));

        assert!(
            feature.tags.iter().any(|tag| tag == "backend"),
            "{relative_path} deve manter a tag @backend"
        );
        assert!(
            feature.tags.iter().any(|tag| tag == "pending_mobile"),
            "{relative_path} deve manter a tag @pending_mobile"
        );

        let mut scenario_count = 0usize;
        for rule in &feature.rules {
            for scenario in &rule.scenarios {
                scenario_count += 1;
                assert!(
                    !scenario_filter::is_pending_backend(&feature, Some(rule), scenario),
                    "cenário '{}' em {relative_path} não deve estar marcado como @pending_backend",
                    scenario.name
                );

                let steps = feature
                    .background
                    .iter()
                    .flat_map(|bg| &bg.steps)
                    .chain(rule.background.iter().flat_map(|bg| &bg.steps))
                    .chain(scenario.steps.iter());

                for step in steps {
                    match collection.find(step) {
                        Ok(Some(_)) => {}
                        Ok(None) => panic!(
                            "step não registrado em {relative_path} / '{}': {}",
                            scenario.name, step.value
                        ),
                        Err(err) => panic!(
                            "step ambíguo em {relative_path} / '{}': {} ({err})",
                            scenario.name, step.value
                        ),
                    }
                }
            }
        }
        assert_eq!(scenario_count, expected_scenarios);
    }
}

/// Garante que `controle_vagas_eventos.feature` habilita os 6 cenários MVP de controle básico de vagas
/// no backend (sem `@pending_backend`), mantém os 5 cenários pós-MVP da fila de espera (#279) marcados
/// com `@pending_backend`, e possui 100% dos steps registrados sem ambiguidade no Cucumber.
#[test]
fn controle_vagas_eventos_mvp_scenarios_are_enabled_and_all_steps_registered() {
    let features_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../features");
    let collection = AppWorld::collection();
    let controle_path = features_dir.join("controle_vagas_eventos.feature");
    let feature = Feature::parse_path(&controle_path, Default::default())
        .expect("deve fazer parse de controle_vagas_eventos.feature");

    assert!(
        feature.tags.iter().any(|tag| tag == "backend"),
        "controle_vagas_eventos.feature deve manter a tag @backend"
    );
    assert!(
        feature.tags.iter().any(|tag| tag == "pending_mobile"),
        "controle_vagas_eventos.feature deve manter a tag @pending_mobile"
    );

    let mut enabled_mvp_scenarios = 0usize;
    let mut pending_waitlist_scenarios = 0usize;

    for rule in &feature.rules {
        for scenario in &rule.scenarios {
            let is_mvp = scenario.tags.iter().any(|tag| tag == "mvp");
            let is_pending = scenario_filter::is_pending_backend(&feature, Some(rule), scenario);

            if is_mvp {
                assert!(
                    !is_pending,
                    "cenário MVP '{}' não deve estar marcado como @pending_backend",
                    scenario.name
                );
                enabled_mvp_scenarios += 1;
            } else {
                assert!(
                    is_pending,
                    "cenário pós-MVP da fila '{}' deve permanecer @pending_backend para a issue #279",
                    scenario.name
                );
                pending_waitlist_scenarios += 1;
            }

            let steps = feature
                .background
                .iter()
                .flat_map(|bg| &bg.steps)
                .chain(rule.background.iter().flat_map(|bg| &bg.steps))
                .chain(scenario.steps.iter());

            for step in steps {
                match collection.find(step) {
                    Ok(Some(_)) => {}
                    Ok(None) => panic!(
                        "step não registrado em controle_vagas_eventos.feature / '{}': {}",
                        scenario.name, step.value
                    ),
                    Err(err) => panic!(
                        "step ambíguo em controle_vagas_eventos.feature / '{}': {} ({err})",
                        scenario.name, step.value
                    ),
                }
            }
        }
    }

    assert_eq!(enabled_mvp_scenarios, 6);
    assert_eq!(pending_waitlist_scenarios, 5);
}
