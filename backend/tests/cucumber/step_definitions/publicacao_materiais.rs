//! Fluxos I06 e I11 com casos de uso de produção, PostgreSQL e storage controlado.

use std::sync::atomic::Ordering;

use backend::{
    infrastructure::database::PgDatabase,
    modules::materials::application::{
        ports::MaterialRepository,
        services::{self, MAX_PDF_BYTES, MaterialError, MaterialUpload},
    },
};
use cucumber::{given, then, when};
use uuid::Uuid;

use crate::support::{
    AppWorld,
    materials::{MaterialsScenario, ScenarioStorage, fixtures},
};

fn scenario(world: &mut AppWorld) -> &mut MaterialsScenario {
    world
        .materials
        .as_mut()
        .expect("contexto da publicação preparado")
}

#[given(regex = r#"^que o CA ativo para publicação de materiais é "([^"]*)"$"#)]
async fn active_material_center(world: &mut AppWorld, active: String) {
    let ca_names = [active, "CA de Matemática".into()];
    let discipline_names = ["Cálculo I".into(), "Direito Constitucional".into()];
    let fixture = fixtures::Fixture::new(
        world.database_pool(),
        ca_names.each_ref().map(String::as_str),
        discipline_names.each_ref().map(String::as_str),
    )
    .await;
    world.materials = Some(MaterialsScenario {
        fixture,
        ca_names,
        discipline_names,
        upload: None,
        storage: ScenarioStorage::default(),
        request_id: Uuid::new_v4(),
        result: None,
        found: None,
    });
}

#[given(
    regex = r#"^a disciplina "([^"]*)" está disponível para publicar materiais (no CA ativo|somente em outro CA)$"#
)]
async fn available_discipline(world: &mut AppWorld, name: String, context: String) {
    let db = PgDatabase::new(world.database_pool().clone());
    let state = scenario(world);
    let index = usize::from(context == "somente em outro CA");
    let available = services::disciplines(&db, state.fixture.author, state.fixture.ca[index])
        .await
        .unwrap();
    assert!(
        available
            .iter()
            .any(|item| item.id == state.discipline(&name) && item.name == name)
    );
}

#[given(
    regex = r#"^em cada um dos CAs "([^"]*)" e "([^"]*)" há um estudante com acesso à consulta e sem permissão de publicação$"#
)]
async fn students_can_read(world: &mut AppWorld, first: String, second: String) {
    let db = PgDatabase::new(world.database_pool().clone());
    let state = scenario(world);
    for name in [first, second] {
        let ca = state.ca(&name);
        let context = db
            .center(state.fixture.students[ca], state.fixture.ca[ca])
            .await
            .unwrap();
        assert!(!context.can_publish);
    }
}

#[given(regex = r"^que o integrante pode gerenciar materiais no CA ativo$")]
async fn can_publish(world: &mut AppWorld) {
    let pool = world.database_pool().clone();
    let state = scenario(world);
    state.fixture.grant(&pool, state.fixture.ca[0]).await;
}

#[given(regex = r"^que o integrante não pode gerenciar materiais no CA ativo$")]
async fn cannot_publish(world: &mut AppWorld) {
    let db = PgDatabase::new(world.database_pool().clone());
    let state = scenario(world);
    assert!(
        !db.center(state.fixture.author, state.fixture.ca[0])
            .await
            .unwrap()
            .can_publish
    );
}

#[given(regex = r#"^que a permissão "([^"]*)" do integrante vale somente no "([^"]*)"$"#)]
async fn other_center_permission(world: &mut AppWorld, permission: String, name: String) {
    assert_eq!(permission, "Gerenciar materiais");
    let pool = world.database_pool().clone();
    let state = scenario(world);
    state
        .fixture
        .grant(&pool, state.fixture.ca[state.ca(&name)])
        .await;
}

#[given(
    regex = r#"^o arquivo "([^"]*)" é um PDF válido (abaixo do limite de tamanho|exatamente no limite de tamanho|1 byte acima do limite de tamanho)$"#
)]
fn prepare_pdf(world: &mut AppWorld, name: String, size: String) {
    let size = match size.as_str() {
        "exatamente no limite de tamanho" => Some(MAX_PDF_BYTES),
        "1 byte acima do limite de tamanho" => Some(MAX_PDF_BYTES + 1),
        _ => None,
    };
    scenario(world).upload = Some((name, fixtures::pdf(size)));
}

#[given(
    regex = r#"^o arquivo "([^"]*)" tem tamanho abaixo do limite e formato não aceito para materiais$"#
)]
fn prepare_invalid_file(world: &mut AppWorld, name: String) {
    scenario(world).upload = Some((name, b"formato incompativel".to_vec()));
}

#[given(regex = r"^o envio do arquivo falhará nesta tentativa$")]
fn storage_failure(world: &mut AppWorld) {
    scenario(world)
        .storage
        .fail_next
        .store(true, Ordering::SeqCst);
}

#[given(regex = r#"^o envio anterior de "([^"]*)" para publicar "([^"]*)" no CA ativo falhou$"#)]
async fn previous_failure(world: &mut AppWorld, file_name: String, title: String) {
    storage_failure(world);
    let db = PgDatabase::new(world.database_pool().clone());
    let state = scenario(world);
    state
        .publish(
            &db,
            title,
            state.fixture.discipline[0],
            state.pdf(&file_name),
        )
        .await;
    assert!(matches!(
        state.rejection(),
        MaterialError::StorageUnavailable
    ));
}

#[when(
    regex = r#"^o integrante solicita a publicação de "([^"]*)" em "([^"]*)" com o link "([^"]*)"$"#
)]
async fn publish_link(world: &mut AppWorld, title: String, discipline: String, url: String) {
    let db = PgDatabase::new(world.database_pool().clone());
    let state = scenario(world);
    state
        .publish(
            &db,
            title,
            state.discipline(&discipline),
            MaterialUpload::Link(url),
        )
        .await;
}

#[when(
    regex = r#"^o integrante solicita a publicação de "([^"]*)" em "([^"]*)" com o arquivo "([^"]*)"$"#
)]
async fn publish_pdf(world: &mut AppWorld, title: String, discipline: String, name: String) {
    let db = PgDatabase::new(world.database_pool().clone());
    let state = scenario(world);
    state
        .publish(&db, title, state.discipline(&discipline), state.pdf(&name))
        .await;
}

#[then(
    regex = r#"^um estudante sem permissão de publicação encontra(?: uma única publicação de)? "([^"]*)" em "([^"]*)" no "([^"]*)"$"#
)]
async fn found_material(world: &mut AppWorld, title: String, discipline: String, ca: String) {
    let db = PgDatabase::new(world.database_pool().clone());
    let state = scenario(world);
    let ca = state.ca(&ca);
    let page = services::list(
        &db,
        state.fixture.students[ca],
        state.fixture.ca[ca],
        Some(state.discipline(&discipline)),
        30,
        0,
    )
    .await
    .unwrap();
    let found: Vec<_> = page
        .items
        .iter()
        .filter(|entry| entry.material.title() == title)
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].material.author_id(), state.fixture.author.id);
    state.found = Some((ca, found[0].material.id()));
}

#[then(
    regex = r#"^o estudante consegue acessar o endereço "([^"]*)" a partir do material publicado$"#
)]
async fn access_link(world: &mut AppWorld, url: String) {
    let db = PgDatabase::new(world.database_pool().clone());
    let state = scenario(world);
    let (ca, id) = state.found.expect("material localizado");
    let published = services::details(&db, state.fixture.students[ca], state.fixture.ca[ca], id)
        .await
        .unwrap();
    assert_eq!(
        published.material.content().external_url(),
        Some(url.as_str())
    );
}

#[then(regex = r#"^o estudante consegue acessar o PDF "([^"]*)" a partir do material publicado$"#)]
async fn access_pdf(world: &mut AppWorld, file: String) {
    let db = PgDatabase::new(world.database_pool().clone());
    let state = scenario(world);
    let (ca, id) = state.found.expect("material localizado");
    let (metadata, bytes) = services::download(
        &db,
        Some(&state.storage),
        state.fixture.students[ca],
        state.fixture.ca[ca],
        id,
    )
    .await
    .unwrap();
    assert_eq!(metadata.name, file);
    assert_eq!(bytes, state.upload.as_ref().unwrap().1);
}

#[then(regex = r#"^o material "([^"]*)" não aparece entre os materiais publicados do "([^"]*)"$"#)]
async fn absent_material(world: &mut AppWorld, title: String, name: String) {
    let db = PgDatabase::new(world.database_pool().clone());
    let state = scenario(world);
    let ca = state.ca(&name);
    let page = services::list(
        &db,
        state.fixture.students[ca],
        state.fixture.ca[ca],
        None,
        30,
        0,
    )
    .await
    .unwrap();
    assert!(
        !page
            .items
            .iter()
            .any(|entry| entry.material.title() == title)
    );
}

#[then(
    regex = r"^a publicação do material é recusada por (endereço externo inválido|falta de permissão|disciplina incompatível com o CA ativo|formato de arquivo não aceito|tamanho de arquivo acima do limite)$"
)]
fn publication_rejected(world: &mut AppWorld, reason: String) {
    let expected = match reason.as_str() {
        "endereço externo inválido" => MaterialError::InvalidLink,
        "falta de permissão" => MaterialError::Forbidden,
        "disciplina incompatível com o CA ativo" => MaterialError::InvalidDiscipline,
        "formato de arquivo não aceito" => MaterialError::InvalidPdf,
        "tamanho de arquivo acima do limite" => MaterialError::TooLarge,
        _ => unreachable!("motivo validado pela expressão"),
    };
    assert_eq!(
        std::mem::discriminant(scenario(world).rejection()),
        std::mem::discriminant(&expected)
    );
}

#[then(regex = r"^o integrante é informado de que o envio do arquivo falhou$")]
fn failure_reported(world: &mut AppWorld) {
    assert!(matches!(
        scenario(world).rejection(),
        MaterialError::StorageUnavailable
    ));
}
