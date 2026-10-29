//! Definições de passos (Step Definitions) para visualização de notícias (#78).
//!
//! Automatiza os cenários Gherkin de `features/visualizacao_noticias.feature`, cobrindo
//! listagem exclusiva de notícias publicadas, isolamento estrito por Centro Acadêmico ativo,
//! ordenação cronológica decrescente por data de publicação, consulta a detalhes e
//! bloqueio de exibição para conteúdos não publicados.

use axum::http::StatusCode;
use backend::modules::news::{api::dto::NewsArticleResponseDto, domain::NewsStatus};
use chrono::{NaiveDate, Utc};
use cucumber::{given, then, when};

use crate::support::AppWorld;

/// Define o Centro Acadêmico ativo no cenário atual.
#[given(expr = "que o CA ativo é {string}")]
fn que_o_ca_ativo_e(world: &mut AppWorld, ca_name: String) {
    world.news_viewing.set_active_ca(&ca_name);
}

/// Cadastra duas notícias publicadas no Centro Acadêmico ativo.
#[given(expr = "existem as notícias publicadas {string} e {string} nesse CA")]
fn existem_noticias_publicadas_nesse_ca(world: &mut AppWorld, title1: String, title2: String) {
    let ca = world
        .news_viewing
        .active_ca_name
        .clone()
        .unwrap_or_else(|| "CA de Computação".to_string());
    let now = Utc::now();
    world.news_viewing.add_article(
        &ca,
        &title1,
        "Conteúdo publicado de teste 1",
        NewsStatus::Published,
        Some(now),
    );
    world.news_viewing.add_article(
        &ca,
        &title2,
        "Conteúdo publicado de teste 2",
        NewsStatus::Published,
        Some(now),
    );
}

/// Cadastra uma notícia em rascunho no Centro Acadêmico ativo.
#[given(expr = "existe a notícia {string} em rascunho nesse CA")]
fn existe_noticia_em_rascunho(world: &mut AppWorld, title: String) {
    let ca = world
        .news_viewing
        .active_ca_name
        .clone()
        .unwrap_or_else(|| "CA de Computação".to_string());
    world
        .news_viewing
        .add_article(&ca, &title, "Conteúdo em rascunho", NewsStatus::Draft, None);
}

/// Cadastra uma notícia pendente de homologação no Centro Acadêmico ativo.
#[given(expr = "existe a notícia {string} pendente de homologação nesse CA")]
fn existe_noticia_pendente_homologacao(world: &mut AppWorld, title: String) {
    let ca = world
        .news_viewing
        .active_ca_name
        .clone()
        .unwrap_or_else(|| "CA de Computação".to_string());
    world.news_viewing.add_article(
        &ca,
        &title,
        "Conteúdo pendente de homologação",
        NewsStatus::PendingReview,
        None,
    );
}

/// Cadastra uma notícia rejeitada no Centro Acadêmico ativo.
#[given(expr = "existe a notícia {string} rejeitada nesse CA")]
fn existe_noticia_rejeitada(world: &mut AppWorld, title: String) {
    let ca = world
        .news_viewing
        .active_ca_name
        .clone()
        .unwrap_or_else(|| "CA de Computação".to_string());
    world.news_viewing.add_article(
        &ca,
        &title,
        "Conteúdo rejeitado",
        NewsStatus::Rejected,
        None,
    );
}

/// Cadastra uma notícia arquivada no Centro Acadêmico ativo.
#[given(expr = "existe a notícia {string} arquivada nesse CA")]
fn existe_noticia_arquivada(world: &mut AppWorld, title: String) {
    let ca = world
        .news_viewing
        .active_ca_name
        .clone()
        .unwrap_or_else(|| "CA de Computação".to_string());
    world.news_viewing.add_article(
        &ca,
        &title,
        "Conteúdo arquivado",
        NewsStatus::Archived,
        None,
    );
}

/// Assegura a ausência de notícias publicadas no Centro Acadêmico ativo.
#[given("não existem notícias publicadas nesse CA")]
fn nao_existem_noticias_publicadas(_world: &mut AppWorld) {
    // O fixture nasce sem artigos registrados para o Centro Acadêmico ativo.
}

/// Cadastra uma notícia publicada avulsa no Centro Acadêmico ativo.
#[given(expr = "existe a notícia publicada {string} nesse CA")]
fn existe_noticia_publicada_nesse_ca(world: &mut AppWorld, title: String) {
    let ca = world
        .news_viewing
        .active_ca_name
        .clone()
        .unwrap_or_else(|| "CA de Computação".to_string());
    world.news_viewing.add_article(
        &ca,
        &title,
        "Conteúdo padrão da notícia",
        NewsStatus::Published,
        Some(Utc::now()),
    );
}

/// Atualiza o corpo/conteúdo da última notícia cadastrada.
#[given(expr = "o conteúdo da notícia é {string}")]
fn o_conteudo_da_noticia_e(world: &mut AppWorld, body: String) {
    world.news_viewing.update_last_article_body(&body);
}

/// Cadastra notícias publicadas a partir de uma tabela com datas específicas de publicação.
#[given("existem as seguintes notícias publicadas nesse CA:")]
fn existem_seguintes_noticias_publicadas(world: &mut AppWorld, step: &cucumber::gherkin::Step) {
    let ca = world
        .news_viewing
        .active_ca_name
        .clone()
        .unwrap_or_else(|| "CA de Computação".to_string());
    let table = step.table.as_ref().expect("tabela Gherkin esperada");

    for row in table.rows.iter().skip(1) {
        let title = &row[0];
        let date_str = &row[1];
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .expect("data válida no formato AAAA-MM-DD");
        let published_at = date
            .and_hms_opt(12, 0, 0)
            .expect("hora válida para timestamp")
            .and_utc();

        world.news_viewing.add_article(
            &ca,
            title,
            "Conteúdo com data de publicação controlada",
            NewsStatus::Published,
            Some(published_at),
        );
    }
}

/// Cadastra uma notícia publicada em outro Centro Acadêmico explicitamente nomeado.
#[given(expr = "existe a notícia publicada {string} no CA {string}")]
fn existe_noticia_publicada_em_outro_ca(world: &mut AppWorld, title: String, ca_name: String) {
    world.news_viewing.add_article(
        &ca_name,
        &title,
        "Conteúdo pertencente a outro Centro Acadêmico",
        NewsStatus::Published,
        Some(Utc::now()),
    );
}

/// Consulta a listagem de notícias publicadas do Centro Acadêmico ativo via rota HTTP.
#[when("consulto a listagem de notícias")]
#[when("consulto novamente a listagem de notícias")]
async fn consulto_listagem_noticias(world: &mut AppWorld) {
    world.ensure_news_viewing_app();
    let ca_id = world.news_viewing.active_ca_id();
    world.get(&format!("/news?ca_id={ca_id}")).await;
}

/// Consulta os detalhes de uma notícia publicada existente pelo seu título.
#[when(expr = "consulto os detalhes da notícia {string}")]
async fn consulto_detalhes_noticia(world: &mut AppWorld, title: String) {
    world.ensure_news_viewing_app();
    let news_id = world
        .news_viewing
        .get_article_id(&title)
        .unwrap_or_else(|| panic!("notícia com título '{title}' não encontrada no cenário"));
    world.get(&format!("/news/{news_id}")).await;
}

/// Altera o Centro Acadêmico ativo para outro especificado.
#[when(expr = "altero o CA ativo para {string}")]
fn altero_o_ca_ativo_para(world: &mut AppWorld, ca_name: String) {
    world.news_viewing.set_active_ca(&ca_name);
}

/// Tenta consultar os detalhes de uma notícia não publicada pelo seu título.
#[when(expr = "tento consultar os detalhes da notícia {string}")]
async fn tento_consultar_detalhes_noticia(world: &mut AppWorld, title: String) {
    world.ensure_news_viewing_app();
    let news_id = world
        .news_viewing
        .get_article_id(&title)
        .unwrap_or_else(|| panic!("notícia com título '{title}' não encontrada no cenário"));
    world.get(&format!("/news/{news_id}")).await;
}

/// Verifica a presença de duas notícias específicas na listagem retornada.
#[then(expr = "vejo {string} e {string}")]
fn vejo_duas_noticias(world: &mut AppWorld, title1: String, title2: String) {
    let response = world
        .last_response
        .as_ref()
        .expect("resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let articles: Vec<NewsArticleResponseDto> = serde_json::from_str(&response.body)
        .expect("corpo da resposta deve conter JSON de notícias");

    assert!(
        articles.iter().any(|a| a.title == title1),
        "notícia '{title1}' deveria estar presente na listagem"
    );
    assert!(
        articles.iter().any(|a| a.title == title2),
        "notícia '{title2}' deveria estar presente na listagem"
    );
}

/// Verifica a presença de uma notícia específica na listagem retornada.
#[then(expr = "vejo {string}")]
fn vejo_uma_noticia(world: &mut AppWorld, title: String) {
    let response = world
        .last_response
        .as_ref()
        .expect("resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let articles: Vec<NewsArticleResponseDto> = serde_json::from_str(&response.body)
        .expect("corpo da resposta deve conter JSON de notícias");

    assert!(
        articles.iter().any(|a| a.title == title),
        "notícia '{title}' deveria estar presente na listagem"
    );
}

/// Verifica a ausência de uma notícia específica na listagem retornada.
#[then(expr = "não vejo {string}")]
fn nao_vejo_noticia(world: &mut AppWorld, title: String) {
    let response = world
        .last_response
        .as_ref()
        .expect("resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let articles: Vec<NewsArticleResponseDto> = serde_json::from_str(&response.body)
        .expect("corpo da resposta deve conter JSON de notícias");

    assert!(
        !articles.iter().any(|a| a.title == title),
        "notícia '{title}' não deveria estar presente na listagem"
    );
}

/// Assegura que nenhum conteúdo não publicado conste na listagem do mural.
#[then("não vejo notícias em rascunho, pendentes de homologação, rejeitadas ou arquivadas")]
fn nao_vejo_noticias_nao_publicadas(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let articles: Vec<NewsArticleResponseDto> = serde_json::from_str(&response.body)
        .expect("corpo da resposta deve conter JSON de notícias");

    for article in &articles {
        assert_eq!(
            article.status, "published",
            "a listagem só deve conter notícias publicadas, mas encontrou status '{}'",
            article.status
        );
    }

    let titulos_nao_publicados = [
        "Rascunho de evento",
        "Proposta de palestra",
        "Comunicado rejeitado",
        "Aviso arquivado",
    ];

    for title in titulos_nao_publicados {
        assert!(
            !articles.iter().any(|a| a.title == title),
            "notícia não publicada '{title}' não deve constar na listagem"
        );
    }
}

/// Verifica se a listagem responde informando lista vazia quando não há notícias publicadas.
#[then("a listagem informa que não há notícias publicadas")]
fn listagem_informa_sem_noticias(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let articles: Vec<NewsArticleResponseDto> = serde_json::from_str(&response.body)
        .expect("corpo da resposta deve conter JSON de notícias");

    assert!(
        articles.is_empty(),
        "a listagem pública deveria estar vazia quando não há notícias publicadas"
    );
}

/// Verifica se o título da notícia consultada confere com o valor esperado.
#[then(expr = "vejo o título {string}")]
fn vejo_titulo_noticia(world: &mut AppWorld, expected_title: String) {
    let response = world
        .last_response
        .as_ref()
        .expect("resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let article: NewsArticleResponseDto = serde_json::from_str(&response.body)
        .expect("corpo da resposta deve conter JSON de detalhe da notícia");

    assert_eq!(article.title, expected_title);
}

/// Verifica se o corpo/conteúdo da notícia consultada confere com o valor esperado.
#[then(expr = "vejo o conteúdo {string}")]
fn vejo_conteudo_noticia(world: &mut AppWorld, expected_body: String) {
    let response = world
        .last_response
        .as_ref()
        .expect("resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let article: NewsArticleResponseDto = serde_json::from_str(&response.body)
        .expect("corpo da resposta deve conter JSON de detalhe da notícia");

    assert_eq!(article.body, expected_body);
}

/// Verifica se as notícias aparecem na ordem cronológica decrescente estipulada pela tabela.
#[then("as notícias aparecem nesta ordem:")]
fn noticias_aparecem_na_ordem(world: &mut AppWorld, step: &cucumber::gherkin::Step) {
    let response = world
        .last_response
        .as_ref()
        .expect("resposta HTTP deve ter sido capturada");
    assert_eq!(response.status, StatusCode::OK);
    let articles: Vec<NewsArticleResponseDto> = serde_json::from_str(&response.body)
        .expect("corpo da resposta deve conter JSON de notícias");

    let table = step.table.as_ref().expect("tabela Gherkin esperada");
    let expected_order: Vec<String> = table
        .rows
        .iter()
        .skip(1)
        .map(|row| row[0].clone())
        .collect();

    let actual_order: Vec<String> = articles.into_iter().map(|a| a.title).collect();
    assert_eq!(
        actual_order, expected_order,
        "as notícias publicadas devem ser ordenadas da mais recente para a mais antiga"
    );
}

/// Assegura que o conteúdo de notícia não publicada não é disponibilizado (retornando 404).
#[then("o conteúdo da notícia não é disponibilizado")]
fn conteudo_nao_disponibilizado(world: &mut AppWorld) {
    let response = world
        .last_response
        .as_ref()
        .expect("resposta HTTP deve ter sido capturada");
    assert_eq!(
        response.status,
        StatusCode::NOT_FOUND,
        "a consulta por notícia não publicada não deve expor o conteúdo e deve retornar 404"
    );
}
