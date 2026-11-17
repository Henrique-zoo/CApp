//! Confere o vínculo entre os cenários do produto e os steps registrados.
//!
//! As tags de pendência suspendem a execução funcional, mas não dispensam o
//! contrato entre Gherkin e automação. Este teste expande os exemplos e consulta
//! a coleção real do Cucumber, inclusive para cenários pendentes. Não cria World,
//! banco ou clientes e não executa os corpos dos steps.

mod step_definitions;
#[allow(dead_code)]
mod support;

use cucumber::{
    World as _,
    gherkin::{Feature, Rule, Scenario, Step},
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use support::AppWorld;

/// Localiza as especificações de produto sem depender do diretório de execução.
fn feature_paths(directory: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(directory).expect("diretório de features legível") {
        let path = entry.expect("entrada de feature legível").path();
        if path.is_dir() {
            paths.extend(feature_paths(&path));
        } else if path
            .extension()
            .is_some_and(|extension| extension == "feature")
        {
            paths.push(path);
        }
    }
    paths.sort();
    paths
}

/// Produz uma instância por linha de exemplos; cenários comuns têm uma instância.
fn examples(scenario: &Scenario) -> Vec<BTreeMap<String, String>> {
    if scenario.examples.is_empty() {
        return vec![BTreeMap::new()];
    }
    let mut result = Vec::new();
    for example in &scenario.examples {
        let table = example
            .table
            .as_ref()
            .expect("exemplos devem possuir tabela");
        let headers = table
            .rows
            .first()
            .expect("exemplos devem possuir cabeçalho");
        for row in table.rows.iter().skip(1) {
            result.push(headers.iter().cloned().zip(row.iter().cloned()).collect());
        }
    }
    assert!(
        !result.is_empty(),
        "esquema sem linhas de exemplos: {}",
        scenario.name
    );
    result
}

/// Confere contexto e ações usando a mesma resolução de expressões do runner.
fn check_scenario(
    collection: &cucumber::step::Collection<AppWorld>,
    feature: &Feature,
    rule: Option<&Rule>,
    scenario: &Scenario,
    failures: &mut Vec<String>,
) -> usize {
    let selected = feature
        .tags
        .iter()
        .chain(rule.into_iter().flat_map(|rule| rule.tags.iter()))
        .chain(scenario.tags.iter())
        .any(|tag| tag == "backend");
    if !selected {
        return 0;
    }
    let steps: Vec<&Step> = feature
        .background
        .iter()
        .flat_map(|background| &background.steps)
        .chain(rule.into_iter().flat_map(|rule| {
            rule.background
                .iter()
                .flat_map(|background| &background.steps)
        }))
        .chain(scenario.steps.iter())
        .collect();
    let mut checked = 0;
    for values in examples(scenario) {
        for step in &steps {
            let mut expanded = (*step).clone();
            for (name, value) in &values {
                expanded.value = expanded.value.replace(&format!("<{name}>"), value);
            }
            checked += 1;
            let problem = match collection.find(&expanded) {
                Ok(Some(_)) => continue,
                Ok(None) => "step não registrado".to_owned(),
                Err(error) => format!("step ambíguo: {error}"),
            };
            failures.push(format!(
                "{} / {} / {}: {problem}",
                feature.name, scenario.name, expanded.value
            ));
        }
    }
    checked
}

/// Cenários pendentes também precisam de exatamente uma definição para cada step.
#[test]
fn every_backend_step_has_one_registered_definition() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../features");
    let collection = AppWorld::collection();
    let mut failures = Vec::new();
    let mut checked = 0;
    for path in feature_paths(&root) {
        let feature = Feature::parse_path(&path, Default::default()).expect("Gherkin válido");
        for scenario in &feature.scenarios {
            checked += check_scenario(&collection, &feature, None, scenario, &mut failures);
        }
        for rule in &feature.rules {
            for scenario in &rule.scenarios {
                checked +=
                    check_scenario(&collection, &feature, Some(rule), scenario, &mut failures);
            }
        }
    }
    assert!(checked > 0, "nenhum step de backend encontrado");
    assert!(
        failures.is_empty(),
        "Definições BDD incompletas:\n{}",
        failures.join("\n")
    );
}
