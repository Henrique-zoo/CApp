//! Verifica a seleção de cenários a partir da herança real de tags Gherkin.

#[path = "support/scenario_filter.rs"]
mod scenario_filter;

use cucumber::gherkin::Feature;

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
