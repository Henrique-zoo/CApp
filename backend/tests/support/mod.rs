//! World, hooks e ciclo de vida compartilhados pelos dois runners Cucumber.
//!
//! Cada chamada a [`run`] cria um container PostgreSQL e executa até quatro
//! cenários simultaneamente. O hook `before` associa banco e router ao
//! [`AppWorld`]; o hook `after` os libera, inclusive para cenários que falham
//! sob o controle do runner.
//!
//! # Isolamento
//!
//! Cenários sem tag usam uma mesma cópia de leitura da base-modelo. Cenários
//! com `@isolated_database` na feature, regra ou cenário recebem uma cópia
//! própria e gravável. A configuração de leitura é uma convenção dos testes:
//! os steps não devem desabilitá-la nem alterar dados compartilhados.
//!
//! # Encerramento
//!
//! Depois que o runner retorna, a suíte recolhe bancos remanescentes, fecha
//! pools e remove o container antes de transformar estatísticas de falha em
//! panic. Essa sequência não garante limpeza explícita após aborto do processo
//! ou panic fora do fluxo controlado, como na inicialização do ambiente.

pub(crate) mod authentication_fixture;
mod database;
pub(crate) mod results;
pub(crate) mod world;

use std::{path::PathBuf, sync::Arc};

use cucumber::{StatsWriter, World as _, tag::Ext};
use futures::FutureExt as _;

use database::DatabaseServer;
pub(crate) use world::AppWorld;

/// Nome da tag Gherkin que solicita uma base independente e gravável.
const ISOLATED_DATABASE_TAG: &str = "isolated_database";

/// Verifica se a feature, regra ou cenário solicita banco isolado.
///
/// # Parâmetros
///
/// - `feature`: funcionalidade que contém o cenário.
/// - `rule`: regra Gherkin opcional que agrupa o cenário.
/// - `scenario`: cenário em preparação.
///
/// # Retorno
///
/// `true` se algum nível contiver [`ISOLATED_DATABASE_TAG`]. O parser entrega
/// as tags sem `@`; a comparação diferencia maiúsculas de minúsculas.
fn has_isolated_database_tag(
    feature: &cucumber::gherkin::Feature,
    rule: Option<&cucumber::gherkin::Rule>,
    scenario: &cucumber::gherkin::Scenario,
) -> bool {
    feature
        .tags
        .iter()
        .chain(rule.into_iter().flat_map(|rule| rule.tags.iter()))
        .chain(scenario.tags.iter())
        .any(|tag| tag == ISOLATED_DATABASE_TAG)
}

/// Executa os cenários e encerra a infraestrutura antes de validar o resultado.
///
/// # Parâmetros
///
/// - `features_path`: diretório de cenários selecionado pelo runner. Os dois
///   alvos fornecem caminhos absolutos derivados de `CARGO_MANIFEST_DIR`.
///
/// Um único container é reutilizado pelos cenários desta chamada. A execução
/// permite até quatro cenários concorrentes, com hooks de preparação e limpeza.
/// Os filtros de linha de comando são processados pelo Cucumber.
///
/// Steps falhos ou pulados, erros de parsing e erros nos hooks são tratados
/// como falha da suíte. Isso inclui steps sem definição, contabilizados como
/// pulados pelo runner. Zero cenários, por si só, não é considerado uma falha.
///
/// # Panics
///
/// Se a preparação ou o encerramento do PostgreSQL falhar, ou se as estatísticas
/// indicarem qualquer falha após o encerramento. Um panic na preparação ou no
/// próprio encerramento pode interromper a sequência explícita de limpeza.
pub(crate) async fn run(features_path: PathBuf, required_tag: Option<&'static str>) {
    let mut cli = cucumber::cli::Opts::<_, _, _>::parsed();
    let requested_tags = cli.tags_filter.take();
    let requested_name = cli.re_filter.take();

    let database_server = DatabaseServer::start().await;
    let before_suite = Arc::clone(&database_server.suite);
    let after_suite = Arc::clone(&database_server.suite);

    let writer = AppWorld::cucumber()
        .with_cli(cli)
        .max_concurrent_scenarios(4)
        .before(move |feature, rule, scenario, world| {
            let suite = Arc::clone(&before_suite);
            let isolated = has_isolated_database_tag(feature, rule, scenario);
            async move { world.attach_database(&suite, isolated).await }.boxed_local()
        })
        .after(move |_, _, _, _, world| {
            let suite = Arc::clone(&after_suite);
            async move {
                if let Some(world) = world {
                    world.detach_database(&suite).await;
                }
            }
            .boxed_local()
        })
        .filter_run(features_path, move |feature, rule, scenario| {
            let tags = feature
                .tags
                .iter()
                .chain(rule.into_iter().flat_map(|rule| rule.tags.iter()))
                .chain(scenario.tags.iter());

            let component_matches =
                required_tag.is_none_or(|required| tags.clone().any(|tag| tag == required));

            let tags_match = requested_tags
                .as_ref()
                .is_none_or(|filter| filter.eval(tags));

            let name_matches = requested_name
                .as_ref()
                .is_none_or(|filter| filter.is_match(&scenario.name));

            component_matches && tags_match && name_matches
        })
        .await;

    let failures = (
        writer.failed_steps(),
        writer.skipped_steps(),
        writer.parsing_errors(),
        writer.hook_errors(),
    );
    database_server.shutdown().await;

    let (failed_steps, skipped_steps, parsing_errors, hook_errors) = failures;
    assert!(
        failed_steps == 0 && skipped_steps == 0 && parsing_errors == 0 && hook_errors == 0,
        "Cucumber failed: {failed_steps} failed step(s), {skipped_steps} skipped step(s), \
         {parsing_errors} parsing error(s), {hook_errors} hook error(s)",
    )
}
