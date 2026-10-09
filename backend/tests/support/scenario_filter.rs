//! Seleção dos cenários cuja automação ainda está pendente no backend.

use cucumber::gherkin::{Feature, Rule, Scenario};

/// Verifica a pendência do backend nas tags herdadas pelo cenário.
///
/// Tags da feature, da regra e do cenário são consideradas. A pendência do
/// mobile não impede a execução no backend, nem substitui os filtros da CLI.
pub(super) fn is_pending_backend(
    feature: &Feature,
    rule: Option<&Rule>,
    scenario: &Scenario,
) -> bool {
    feature
        .tags
        .iter()
        .chain(rule.into_iter().flat_map(|rule| rule.tags.iter()))
        .chain(scenario.tags.iter())
        .any(|tag| tag == "pending_backend")
}
