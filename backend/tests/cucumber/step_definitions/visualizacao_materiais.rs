//! Consulta de materiais pelo router real com sessão e dados locais.

use axum::http::{StatusCode, header};
use cucumber::{given, then, when};

use super::pending_step;
use crate::support::{
    AppWorld,
    materials::{MATERIAL_LINK, MATERIAL_PDF, MaterialReadScenario, fixtures},
};

pub(super) fn reads(world: &mut AppWorld) -> &mut MaterialReadScenario {
    world
        .material_reads
        .as_mut()
        .expect("contexto de consulta de materiais")
}

#[given(regex = r#"^que o CA ativo para consulta de materiais é "([^"]*)"$"#)]
pub(super) async fn active_center(world: &mut AppWorld, name: String) {
    world.material_reads =
        Some(MaterialReadScenario::new(world.database_pool().clone(), name).await);
}

#[given(regex = r#"^a descrição do material é "([^"]*)"$"#)]
async fn material_description(world: &mut AppWorld, description: String) {
    reads(world).describe_last(description).await;
}

#[given(regex = r#"^existe o material publicado "([^"]*)" no CA "([^"]*)"$"#)]
async fn material_in_other_center(world: &mut AppWorld, title: String, ca: String) {
    let scenario = reads(world);
    scenario.seed(scenario.ca(&ca), title, false).await;
}

#[given(regex = r#"^existe o material "([^"]*)" arquivado nesse CA$"#)]
async fn archived_material(world: &mut AppWorld, title: String) {
    let scenario = reads(world);
    let id = scenario.seed(scenario.active, title, true).await;
    scenario.archive(id).await;
}

#[given(regex = r#"^existem os materiais publicados "([^"]*)" e "([^"]*)" nesse CA$"#)]
async fn published_materials(world: &mut AppWorld, first: String, second: String) {
    let scenario = reads(world);
    scenario.seed(scenario.active, first, false).await;
    scenario.seed(scenario.active, second, false).await;
}

#[given(regex = r"^não existem materiais publicados nesse CA$")]
async fn no_publications(world: &mut AppWorld) {
    list_materials(world).await;
    assert!(reads(world).titles().is_empty());
}

#[given(regex = r#"^o material possui conteúdo disponível no formato "([^"]*)"$"#)]
async fn material_content(world: &mut AppWorld, format: String) {
    match format.as_str() {
        "PDF" => reads(world).use_pdf_for_last().await,
        "link externo" => {}
        _ => pending_step("o material possui conteúdo disponível em outro formato"),
    }
}

#[given(regex = r"^que não foi possível obter os materiais publicados do CA ativo$")]
async fn list_dependency_fails(world: &mut AppWorld) {
    active_center(world, "CA de Computação".into()).await;
    reads(world).fail_list_dependency().await;
}

#[when(regex = r"^consulto (?:novamente )?a lista de materiais$")]
async fn list_materials(world: &mut AppWorld) {
    let scenario = reads(world);
    scenario.get(scenario.root()).await;
}

#[when(regex = r#"^altero o CA ativo da consulta de materiais para "([^"]*)"$"#)]
fn change_center(world: &mut AppWorld, ca: String) {
    let scenario = reads(world);
    scenario.active = scenario.ca(&ca);
}

#[when(regex = r#"^consulto os detalhes do material "([^"]*)"$"#)]
async fn open_details(world: &mut AppWorld, title: String) {
    let scenario = reads(world);
    scenario
        .get(format!("{}/{}", scenario.root(), scenario.id(&title)))
        .await;
}

#[when(regex = r#"^acesso o conteúdo do material "([^"]*)"$"#)]
async fn open_content(world: &mut AppWorld, title: String) {
    open_details(world, title.clone()).await;
    let scenario = reads(world);
    let details = scenario.response().json(StatusCode::OK);
    if details["content"]["type"] == "file" {
        assert_eq!(details["content"]["fileName"], MATERIAL_PDF);
        assert_eq!(details["content"]["sizeBytes"], fixtures::pdf(None).len());
        scenario
            .get(format!("{}/{}/file", scenario.root(), scenario.id(&title)))
            .await;
    }
}

#[when(regex = r#"^tento acessar o conteúdo do material "([^"]*)"$"#)]
async fn try_archived_content(world: &mut AppWorld, title: String) {
    let scenario = reads(world);
    let path = format!("{}/{}", scenario.root(), scenario.id(&title));
    scenario.get(path.clone()).await;
    archived_not_found(world);
    reads(world).get(format!("{path}/file")).await;
}

#[then(regex = r"^a listagem informa que não há materiais publicados$")]
fn empty_list(world: &mut AppWorld) {
    let response = reads(world).response().json(StatusCode::OK);
    assert_eq!(response["items"], serde_json::json!([]));
    assert!(response["nextOffset"].is_null());
}

#[then(regex = r"^o aplicativo informa que não foi possível carregar os materiais$")]
fn list_failure_reported(world: &mut AppWorld) {
    let response = reads(world)
        .response()
        .json(StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response["error"]["code"], "material_persistence_failure");
}

#[then(regex = r#"^vejo o material "([^"]*)"$"#)]
fn material_listed(world: &mut AppWorld, title: String) {
    let scenario = reads(world);
    assert!(scenario.titles().contains(&title));
    let response = scenario.response().json(StatusCode::OK);
    for item in response["items"].as_array().unwrap() {
        assert_eq!(
            item["caId"],
            scenario.fixture.ca[scenario.active].to_string()
        );
    }
}

#[then(regex = r#"^vejo os materiais "([^"]*)" e "([^"]*)"$"#)]
fn materials_listed(world: &mut AppWorld, first: String, second: String) {
    material_listed(world, first);
    material_listed(world, second);
}

#[then(regex = r#"^não vejo o material "([^"]*)"$"#)]
fn material_not_listed(world: &mut AppWorld, title: String) {
    assert!(!reads(world).titles().contains(&title));
}

#[then(regex = r#"^vejo o título do material "([^"]*)"$"#)]
fn detail_title(world: &mut AppWorld, title: String) {
    let scenario = reads(world);
    let details = scenario.response().json(StatusCode::OK);
    assert_eq!(details["title"], title);
    assert_eq!(details["id"], scenario.id(&title).to_string());
    assert_eq!(
        details["caId"],
        scenario.fixture.ca[scenario.active].to_string()
    );
}

#[then(regex = r#"^vejo a descrição "([^"]*)"$"#)]
fn detail_description(world: &mut AppWorld, description: String) {
    assert_eq!(
        reads(world).response().json(StatusCode::OK)["description"],
        description
    );
}

#[then(regex = r"^o conteúdo do material não é disponibilizado$")]
fn archived_not_found(world: &mut AppWorld) {
    let error = reads(world).response().json(StatusCode::NOT_FOUND);
    assert_eq!(error["error"]["code"], "material_not_found");
}

#[then(regex = r#"^o conteúdo publicado é disponibilizado no formato "([^"]*)"$"#)]
fn content_delivered(world: &mut AppWorld, format: String) {
    let response = reads(world).response();
    match format.as_str() {
        "PDF" => {
            assert_eq!(response.status, StatusCode::OK);
            assert_eq!(response.headers[header::CONTENT_TYPE], "application/pdf");
            assert_eq!(response.body.as_ref(), fixtures::pdf(None));
        }
        "link externo" => {
            let details = response.json(StatusCode::OK);
            assert_eq!(details["content"]["type"], "external_link");
            assert_eq!(details["content"]["url"], MATERIAL_LINK);
        }
        _ => pending_step("o conteúdo publicado é disponibilizado em outro formato"),
    }
}
