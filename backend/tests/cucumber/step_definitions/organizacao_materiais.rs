//! Consulta do catálogo e filtro por disciplina pelo router real.

use axum::http::StatusCode;
use cucumber::{given, then, when};
use serde_json::Value;

use super::{
    pending_step,
    visualizacao_materiais::{active_center, reads},
};
use crate::support::AppWorld;

fn catalog(world: &mut AppWorld) -> Vec<Value> {
    reads(world)
        .response()
        .json(StatusCode::OK)
        .as_array()
        .expect("catálogo de disciplinas")
        .to_owned()
}

async fn seed(world: &mut AppWorld, ca: usize, title: String, discipline: &str) {
    let scenario = reads(world);
    let id = scenario.ensure_discipline(ca, discipline).await;
    scenario.seed_in(ca, id, title, false).await;
}

#[given(regex = r#"^a disciplina "([^"]*)" pertence à grade curricular do contexto acadêmico$"#)]
#[given(regex = r#"^a grade curricular desse contexto possui a disciplina "([^"]*)"$"#)]
async fn available_discipline(world: &mut AppWorld, name: String) {
    let scenario = reads(world);
    scenario.ensure_discipline(scenario.active, &name).await;
}

#[given(
    regex = r#"^a grade curricular do contexto acadêmico possui as disciplinas "([^"]*)", "([^"]*)" e "([^"]*)"$"#
)]
async fn available_disciplines(world: &mut AppWorld, first: String, second: String, third: String) {
    let scenario = reads(world);
    for name in [first, second, third] {
        scenario.ensure_discipline(scenario.active, &name).await;
    }
}

#[given(regex = r#"^existe a disciplina "([^"]*)" em outro contexto acadêmico$"#)]
async fn foreign_discipline(world: &mut AppWorld, name: String) {
    let scenario = reads(world);
    scenario.ensure_discipline(1 - scenario.active, &name).await;
}

#[given(
    regex = r#"^existe o material publicado "([^"]*)" associado à disciplina "([^"]*)"(?: nesse CA)?$"#
)]
#[given(regex = r#"^o material publicado "([^"]*)" está associado à disciplina "([^"]*)"$"#)]
async fn associated_material(world: &mut AppWorld, title: String, discipline: String) {
    let ca = reads(world).active;
    seed(world, ca, title, &discipline).await;
}

#[given(
    regex = r#"^existe o material publicado "([^"]*)" associado à disciplina "([^"]*)" no "([^"]*)"$"#
)]
async fn foreign_material(world: &mut AppWorld, title: String, discipline: String, ca: String) {
    let ca = reads(world).ca(&ca);
    seed(world, ca, title, &discipline).await;
}

#[given(regex = r#"^existe o material publicado "([^"]*)" no formato (PDF|de link)$"#)]
async fn material_by_format(world: &mut AppWorld, title: String, format: String) {
    let scenario = reads(world);
    scenario.seed(scenario.active, title, format == "PDF").await;
}

#[given(regex = r#"^ambos estão associados à disciplina "([^"]*)"$"#)]
async fn both_associated(world: &mut AppWorld, discipline: String) {
    select_discipline(world, discipline).await;
    assert_eq!(reads(world).titles().len(), 2);
    reads(world).assert_selected_discipline();
}

#[given(
    regex = r#"^existem materiais publicados associados às disciplinas "([^"]*)" e "([^"]*)"$"#
)]
async fn materials_in_two_disciplines(world: &mut AppWorld, first: String, second: String) {
    let ca = reads(world).active;
    for name in [first, second] {
        seed(world, ca, format!("Resumo de {name}"), &name).await;
    }
}

#[given(
    regex = r#"^existem os materiais publicados "([^"]*)" e "([^"]*)" associados à disciplina "([^"]*)"$"#
)]
async fn two_materials_in_discipline(
    world: &mut AppWorld,
    first: String,
    second: String,
    discipline: String,
) {
    let ca = reads(world).active;
    for title in [first, second] {
        seed(world, ca, title, &discipline).await;
    }
}

#[given(regex = r"^não existem disciplinas cadastradas na grade curricular do contexto acadêmico$")]
async fn no_disciplines(world: &mut AppWorld) {
    reads(world).clear_catalog().await;
}

#[given(regex = r#"^não existem materiais publicados associados à disciplina "([^"]*)"$"#)]
async fn no_materials(world: &mut AppWorld, discipline: String) {
    select_discipline(world, discipline).await;
    assert!(reads(world).titles().is_empty());
}

#[given(regex = r#"^o estudante está consultando os materiais de "([^"]*)"$"#)]
async fn initial_selection(world: &mut AppWorld, discipline: String) {
    select_discipline(world, discipline).await;
    assert!(!reads(world).titles().is_empty());
}

#[given(regex = r#"^que o estudante está no "([^"]*)"$"#)]
async fn student_center(world: &mut AppWorld, ca: String) {
    active_center(world, ca).await;
}

#[when(
    regex = r"^(?:o estudante acessa|acessa novamente) a organização de materiais por disciplina$"
)]
#[when(regex = r"^o estudante retorna à lista de disciplinas$")]
async fn open_catalog(world: &mut AppWorld) {
    let scenario = reads(world);
    scenario.selected_discipline = None;
    scenario
        .get(format!("{}/disciplines", scenario.root()))
        .await;
}

#[when(regex = r#"^o estudante altera o CA ativo para "([^"]*)"$"#)]
async fn switch_center(world: &mut AppWorld, ca: String) {
    let scenario = reads(world);
    scenario.active = scenario.ca(&ca);
    open_catalog(world).await;
    assert!(!catalog(world).is_empty());
}

#[when(regex = r#"^(?:o estudante )?consulta os materiais da disciplina "([^"]*)"$"#)]
#[when(regex = r#"^(?:o estudante )?seleciona(?: novamente)? a disciplina "([^"]*)"$"#)]
#[when(regex = r#"^seleciona "([^"]*)"$"#)]
async fn select_discipline(world: &mut AppWorld, name: String) {
    let scenario = reads(world);
    let discipline = scenario
        .discipline(scenario.active, &name)
        .await
        .expect("disciplina da fixture");
    scenario.selected_discipline = Some(discipline);
    scenario
        .get(format!("{}?disciplineId={discipline}", scenario.root()))
        .await;
}

#[when(regex = r"^o estudante sai da organização de materiais$")]
fn leave_catalog(world: &mut AppWorld) {
    let scenario = reads(world);
    scenario.response = None;
    scenario.selected_discipline = None;
}

#[when(regex = r#"^seleciona o material "([^"]*)"$"#)]
async fn select_material(world: &mut AppWorld, title: String) {
    let scenario = reads(world);
    assert!(scenario.titles().contains(&title));
    let id = scenario.id(&title);
    scenario.get(format!("{}/{id}", scenario.root())).await;
    assert_eq!(
        scenario.response().json(StatusCode::OK)["id"],
        id.to_string()
    );
}

#[then(regex = r#"^a disciplina "([^"]*)" é apresentada$"#)]
fn discipline_shown(world: &mut AppWorld, name: String) {
    assert!(catalog(world).iter().any(|item| item["name"] == name));
}

#[then(regex = r#"^a disciplina "([^"]*)" não é apresentada$"#)]
fn discipline_hidden(world: &mut AppWorld, name: String) {
    assert!(!catalog(world).iter().any(|item| item["name"] == name));
}

#[then(regex = r"^as disciplinas da grade curricular são apresentadas para consulta$")]
fn grade_shown(world: &mut AppWorld) {
    let response = catalog(world);
    let mut names: Vec<_> = response
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["Cálculo I", "Física I", "Álgebra Linear"]);
}

#[then(regex = r"^o estudante pode selecionar uma disciplina para visualizar seus materiais$")]
async fn select_from_catalog(world: &mut AppWorld) {
    let response = catalog(world);
    let ids: Vec<uuid::Uuid> = response
        .iter()
        .map(|item| item["id"].as_str().unwrap().parse().unwrap())
        .collect();
    let scenario = reads(world);
    for id in ids {
        scenario.selected_discipline = Some(id);
        scenario
            .get(format!("{}?disciplineId={id}", scenario.root()))
            .await;
        assert!(scenario.titles().is_empty());
    }
}

#[then(regex = r"^ambos ficam disponíveis na organização da disciplina$")]
fn both_formats_shown(world: &mut AppWorld) {
    let scenario = reads(world);
    scenario.assert_selected_discipline();
    let response = scenario.response().json(StatusCode::OK);
    let mut formats: Vec<_> = response["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["content"]["type"].as_str().unwrap())
        .collect();
    formats.sort_unstable();
    assert_eq!(formats, ["external_link", "file"]);
}

#[then(regex = r#"^a organização corresponde ao contexto do "([^"]*)"$"#)]
fn organization_in_center(world: &mut AppWorld, ca: String) {
    let scenario = reads(world);
    assert_eq!(scenario.active, scenario.ca(&ca));
    scenario.assert_selected_discipline();
}

#[then(
    regex = r"^(?:nenhum material é apresentado|o aplicativo informa que não existem materiais disponíveis para essa disciplina)$"
)]
fn empty_selection(world: &mut AppWorld) {
    assert!(reads(world).titles().is_empty());
}

#[then(
    regex = r"^(?:nenhuma disciplina é apresentada para consulta|o aplicativo informa que não existem disciplinas disponíveis para a organização dos materiais)$"
)]
fn empty_catalog(world: &mut AppWorld) {
    assert!(catalog(world).is_empty());
}

#[then(regex = r#"^o material "([^"]*)" (?:é apresentado|continua associado à disciplina)$"#)]
fn material_shown(world: &mut AppWorld, title: String) {
    let scenario = reads(world);
    assert!(scenario.titles().contains(&title));
    scenario.assert_selected_discipline();
}

#[then(
    regex = r#"^o material "([^"]*)" não é apresentado(?: nessa consulta| na consulta atual)?$"#
)]
fn material_hidden(world: &mut AppWorld, title: String) {
    assert!(!reads(world).titles().contains(&title));
}

#[then(regex = r#"^os materiais de "([^"]*)" não são apresentados nessa consulta$"#)]
async fn other_discipline_hidden(world: &mut AppWorld, name: String) {
    let scenario = reads(world);
    let other = scenario
        .discipline(scenario.active, &name)
        .await
        .unwrap()
        .to_string();
    let result = scenario.response().json(StatusCode::OK);
    assert!(
        result["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["disciplineId"] != other)
    );
}

#[then(regex = r#"^são apresentados os materiais associados à disciplina "([^"]*)"$"#)]
async fn results_by_discipline(world: &mut AppWorld, name: String) {
    let scenario = reads(world);
    let discipline = scenario.discipline(scenario.active, &name).await.unwrap();
    assert_eq!(scenario.selected_discipline, Some(discipline));
    assert_eq!(scenario.titles(), [format!("Resumo de {name}")]);
    scenario.assert_selected_discipline();
}

#[then(regex = r"^permanece disponível para consulta$")]
fn still_listed(world: &mut AppWorld) {
    let scenario = reads(world);
    assert!(!scenario.titles().is_empty());
    scenario.assert_selected_discipline();
}

#[then(regex = r"^o material selecionado fica disponível para acesso$")]
fn can_access_selected_material(world: &mut AppWorld) {
    let response = reads(world).response().json(StatusCode::OK);
    assert!(response["content"]["url"].as_str().is_some());
}

#[then(regex = r#"^sua associação com a disciplina "([^"]*)" é mantida$"#)]
async fn selected_material_keeps_discipline(world: &mut AppWorld, name: String) {
    let scenario = reads(world);
    let id = scenario.discipline(scenario.active, &name).await.unwrap();
    let response = scenario.response().json(StatusCode::OK);
    assert_eq!(response["disciplineId"], id.to_string());
}

#[given(regex = r#"^existe o material arquivado "([^"]*)" associado à disciplina "([^"]*)"$"#)]
async fn archived_material(world: &mut AppWorld, title: String, name: String) {
    let scenario = reads(world);
    let discipline = scenario.ensure_discipline(scenario.active, &name).await;
    let id = scenario
        .seed_in(scenario.active, discipline, title, false)
        .await;
    scenario.archive(id).await;
}

#[given(regex = r#"^existe um material publicado nesse CA$"#)]
fn given_existe_um_material_publicado_nesse_ca(_world: &mut AppWorld) {
    pending_step("existe um material publicado nesse CA");
}

#[given(regex = r#"^o integrante não possui a permissão "([^"]*)" nesse CA$"#)]
fn given_o_integrante_nao_possui_a_permissao_valor_nesse_ca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o integrante não possui a permissão {string} nesse CA");
}

#[given(regex = r#"^que o integrante possui a permissão "([^"]*)" no CA ativo$"#)]
fn given_que_o_integrante_possui_a_permissao_valor_no_ca_ativo(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que o integrante possui a permissão {string} no CA ativo");
}

#[then(regex = r#"^a associação do material com a disciplina é registrada$"#)]
fn then_a_associacao_do_material_com_a_disciplina_e_registrada(_world: &mut AppWorld) {
    pending_step("a associação do material com a disciplina é registrada");
}

#[then(regex = r#"^a associação é recusada$"#)]
fn then_a_associacao_e_recusada(_world: &mut AppWorld) {
    pending_step("a associação é recusada");
}

#[then(regex = r#"^a associação é recusada por falta de permissão$"#)]
fn then_a_associacao_e_recusada_por_falta_de_permissao(_world: &mut AppWorld) {
    pending_step("a associação é recusada por falta de permissão");
}

#[then(regex = r#"^a organização anterior do material permanece inalterada$"#)]
fn then_a_organizacao_anterior_do_material_permanece_inalterada(_world: &mut AppWorld) {
    pending_step("a organização anterior do material permanece inalterada");
}

#[then(regex = r#"^o material fica disponível para consulta nessa disciplina$"#)]
fn then_o_material_fica_disponivel_para_consulta_nessa_disciplina(_world: &mut AppWorld) {
    pending_step("o material fica disponível para consulta nessa disciplina");
}

#[then(regex = r#"^o material não fica associado à disciplina incompatível$"#)]
fn then_o_material_nao_fica_associado_a_disciplina_incompativel(_world: &mut AppWorld) {
    pending_step("o material não fica associado à disciplina incompatível");
}

#[then(regex = r#"^o material passa a pertencer à organização da disciplina "([^"]*)"$"#)]
fn then_o_material_passa_a_pertencer_a_organizacao_da_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o material passa a pertencer à organização da disciplina {string}");
}

#[when(regex = r#"^esse integrante associa "([^"]*)" à disciplina "([^"]*)"$"#)]
fn when_esse_integrante_associa_valor_a_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("esse integrante associa {string} à disciplina {string}");
}

#[when(regex = r#"^esse integrante tenta associar "([^"]*)" à disciplina "([^"]*)"$"#)]
fn when_esse_integrante_tenta_associar_valor_a_disciplina_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("esse integrante tenta associar {string} à disciplina {string}");
}

#[when(regex = r#"^tenta associar o material a uma disciplina$"#)]
fn when_tenta_associar_o_material_a_uma_disciplina(_world: &mut AppWorld) {
    pending_step("tenta associar o material a uma disciplina");
}
