//! Definições de passos (Step Definitions) para criação e publicação de notícias (#77).
//!
//! Automatiza os cenários Gherkin de `features/noticias.feature`, cobrindo
//! criação como solicitação, validação de campos obrigatórios, edição de
//! conteúdo, publicação direta e restrições de permissão por Centro Acadêmico.

use chrono::Utc;
use cucumber::{given, then, when};

use crate::support::{AppWorld, world::NewsStatusState};

/// Contexto: usuário faz parte de um Centro Acadêmico específico.
#[given(expr = "que o usuário faz parte do Centro Acadêmico {string}")]
fn usuario_faz_parte_do_ca(world: &mut AppWorld, ca_name: String) {
    world.news_fixture.set_user_ca(ca_name);
}

/// Contexto: usuário faz parte do seu Centro Acadêmico padrão.
#[given("que o usuário faz parte do seu Centro Acadêmico")]
fn usuario_faz_parte_do_seu_ca(world: &mut AppWorld) {
    world.news_fixture.set_user_ca("CA de Computação");
}

/// Usuário está autorizado a editar notícias no Centro Acadêmico informado.
#[given(expr = "que o usuário está autorizado a editar notícias no Centro Acadêmico {string}")]
fn usuario_autorizado_editar_noticias(world: &mut AppWorld, ca_name: String) {
    world.news_fixture.set_user_ca(ca_name.clone());
    world.news_fixture.set_can_edit(ca_name, true);
}

/// Usuário não possui permissão para editar notícias no Centro Acadêmico informado.
#[given(
    expr = "que o usuário não possui permissão para editar notícias no Centro Acadêmico {string}"
)]
fn usuario_sem_permissao_editar_noticias(world: &mut AppWorld, ca_name: String) {
    world.news_fixture.set_user_ca(ca_name.clone());
    world.news_fixture.set_can_edit(ca_name, false);
}

/// Usuário possui permissão de publicação direta no Centro Acadêmico informado.
#[given(expr = "que o usuário possui permissão de publicação direta no Centro Acadêmico {string}")]
fn usuario_com_publicacao_direta(world: &mut AppWorld, ca_name: String) {
    world.news_fixture.set_user_ca(ca_name.clone());
    world.news_fixture.set_can_publish_directly(ca_name, true);
}

/// Gestor não possui permissão de publicação direta no Centro Acadêmico informado.
#[given(
    expr = "que o gestor do Centro Acadêmico {string} não possui permissão de publicação direta"
)]
fn gestor_sem_publicacao_direta(world: &mut AppWorld, ca_name: String) {
    world.news_fixture.set_user_ca(ca_name.clone());
    world.news_fixture.set_can_publish_directly(ca_name, false);
}

/// Notícia não publicada existente no Centro Acadêmico do contexto.
#[given(
    expr = "nesse Centro Acadêmico existe uma notícia não publicada com o título {string} e o conteúdo {string}"
)]
#[given(
    expr = "que nesse Centro Acadêmico existe uma notícia não publicada com o título {string} e o conteúdo {string}"
)]
fn noticia_nao_publicada_existente(world: &mut AppWorld, title: String, body: String) {
    let ca = world.news_fixture.get_effective_ca();
    world
        .news_fixture
        .add_existing_article(&ca, &title, &body, NewsStatusState::Solicitacao, None);
}

/// Notícia já publicada existente no Centro Acadêmico do contexto.
#[given(
    expr = "nesse Centro Acadêmico existe uma notícia já publicada com o título {string} e o conteúdo {string}"
)]
#[given(
    expr = "que nesse Centro Acadêmico existe uma notícia já publicada com o título {string} e o conteúdo {string}"
)]
fn noticia_ja_publicada_existente(world: &mut AppWorld, title: String, body: String) {
    let ca = world.news_fixture.get_effective_ca();
    world.news_fixture.add_existing_article(
        &ca,
        &title,
        &body,
        NewsStatusState::Publicada,
        Some(Utc::now()),
    );
}

/// Notícia não publicada pertencente a outro Centro Acadêmico especificado.
#[given(
    expr = "o Centro Acadêmico {string} possui uma notícia não publicada com o título {string} e o conteúdo {string}"
)]
#[given(
    expr = "que o Centro Acadêmico {string} possui uma notícia não publicada com o título {string} e o conteúdo {string}"
)]
fn noticia_outro_ca(world: &mut AppWorld, ca_name: String, title: String, body: String) {
    world.news_fixture.add_existing_article(
        &ca_name,
        &title,
        &body,
        NewsStatusState::Solicitacao,
        None,
    );
}

/// Notícia pendente de homologação em Centro Acadêmico especificado.
#[given(expr = "que o Centro Acadêmico {string} possui uma notícia pendente de homologação")]
#[given(expr = "o Centro Acadêmico {string} possui uma notícia pendente de homologação")]
fn ca_possui_noticia_pendente(world: &mut AppWorld, ca_name: String) {
    world.news_fixture.add_existing_article(
        &ca_name,
        "Proposta de Notícia Acadêmica",
        "Conteúdo aguardando validação pela diretoria.",
        NewsStatusState::Solicitacao,
        None,
    );
}

/// Notícia rejeitada em Centro Acadêmico especificado.
#[given(expr = "que o Centro Acadêmico {string} possui uma notícia rejeitada")]
#[given(expr = "o Centro Acadêmico {string} possui uma notícia rejeitada")]
fn ca_possui_noticia_rejeitada(world: &mut AppWorld, ca_name: String) {
    world.news_fixture.add_existing_article(
        &ca_name,
        "Notícia Desaprovada",
        "Conteúdo rejeitado na moderação.",
        NewsStatusState::Rejeitada,
        None,
    );
}

/// Criação de notícia com título e conteúdo informados.
#[when(expr = "ele cria uma notícia com o título {string} e o conteúdo {string}")]
fn criar_noticia(world: &mut AppWorld, title: String, body: String) {
    let ca = world.news_fixture.get_effective_ca();
    world.news_fixture.create_news(&ca, &title, &body);
}

/// Tentativa de criação de notícia com título e conteúdo informados.
#[when(expr = "ele tenta criar uma notícia com o título {string} e o conteúdo {string}")]
fn tentar_criar_noticia(world: &mut AppWorld, title: String, body: String) {
    let ca = world.news_fixture.get_effective_ca();
    world.news_fixture.create_news(&ca, &title, &body);
}

/// Alteração do título e conteúdo da notícia.
#[when(expr = "ele altera o título para {string} e o conteúdo para {string}")]
fn alterar_noticia(world: &mut AppWorld, new_title: String, new_body: String) {
    let old_title = world
        .news_fixture
        .get_current_article()
        .map(|a| a.title.clone())
        .unwrap_or_default();
    world
        .news_fixture
        .update_article(&old_title, &new_title, &new_body);
}

/// Tentativa de alteração de notícia para novos valores de título e conteúdo.
#[when(expr = "ele tenta alterar a notícia para o título {string} e o conteúdo {string}")]
#[when(expr = "ele tenta alterar essa notícia para o título {string} e o conteúdo {string}")]
#[when(expr = "ele tenta alterar o título para {string} e o conteúdo para {string}")]
fn tentar_alterar_noticia(world: &mut AppWorld, new_title: String, new_body: String) {
    let old_title = world
        .news_fixture
        .get_current_article()
        .map(|a| a.title.clone())
        .unwrap_or_default();
    world
        .news_fixture
        .update_article(&old_title, &new_title, &new_body);
}

/// Publicação direta da notícia pelo usuário autorizado.
#[when("ele publica essa notícia diretamente")]
fn publicar_noticia_diretamente(world: &mut AppWorld) {
    let title = world
        .news_fixture
        .get_current_article()
        .map(|a| a.title.clone());
    world.news_fixture.publish_directly(title.as_deref());
}

/// Tentativa de publicação direta da notícia.
#[when("ele tenta publicar essa notícia diretamente")]
fn tentar_publicar_noticia_diretamente(world: &mut AppWorld) {
    let title = world
        .news_fixture
        .get_current_article()
        .map(|a| a.title.clone());
    world.news_fixture.publish_directly(title.as_deref());
}

/// Formação da listagem pública de notícias do Centro Acadêmico.
#[when("a listagem de notícias publicadas desse Centro Acadêmico é formada")]
fn formar_listagem_publicada(world: &mut AppWorld) {
    let ca = world.news_fixture.get_effective_ca();
    world.news_fixture.form_published_listing(&ca);
}

/// Verifica se a notícia ficou registrada no CA como solicitação.
#[then(expr = "a notícia fica registrada no Centro Acadêmico {string} como solicitação")]
fn noticia_registrada_como_solicitacao(world: &mut AppWorld, ca_name: String) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia registrada");
    assert_eq!(article.ca_name, ca_name);
    assert_eq!(article.status, NewsStatusState::Solicitacao);
}

/// Verifica se o usuário atual ficou vinculado como autor da notícia.
#[then("o usuário fica vinculado como autor da notícia")]
fn usuario_vinculado_como_autor(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia registrada");
    assert_eq!(article.author, world.news_fixture.current_user);
}

/// Verifica se a data de criação da notícia foi registrada.
#[then("a data de criação da notícia fica registrada")]
fn data_criacao_registrada(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia registrada");
    assert!(article.created_at <= Utc::now());
}

/// Verifica se a notícia fica pendente de validação por um administrador.
#[then("a notícia fica pendente de validação por um administrador")]
#[then("a notícia permanece pendente de validação por um administrador")]
#[then("a notícia permanece pendente de homologação")]
fn noticia_pendente_validacao(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia registrada");
    assert_eq!(article.status, NewsStatusState::Solicitacao);
}

/// Verifica se a notícia não aparece para os usuários.
#[then("a notícia não aparece para os usuários")]
fn noticia_nao_aparece_usuarios(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia registrada");
    assert_ne!(article.status, NewsStatusState::Publicada);
}

/// Verifica se a notícia não aparece para os usuários do CA informado.
#[then(expr = "a notícia não aparece para os usuários do Centro Acadêmico {string}")]
fn noticia_nao_aparece_usuarios_ca(world: &mut AppWorld, ca_name: String) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia registrada");
    assert!(!world.news_fixture.is_published_for_ca(article.id, &ca_name));
}

/// Verifica se a notícia não possui data de publicação.
#[then("a notícia não possui data de publicação")]
fn noticia_sem_data_publicacao(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia registrada");
    assert!(article.published_at.is_none());
}

/// Verifica se o registro da notícia foi aceito com sucesso.
#[then("o registro da notícia é aceito")]
fn registro_noticia_aceito(world: &mut AppWorld) {
    assert_eq!(
        world.news_fixture.last_operation_succeeded,
        Some(true),
        "o registro da notícia deveria ser aceito"
    );
}

/// Verifica se a notícia ficou registrada com o título e conteúdo especificados.
#[then(expr = "a notícia fica registrada com o título {string} e o conteúdo {string}")]
fn noticia_com_titulo_e_conteudo(world: &mut AppWorld, title: String, body: String) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia registrada");
    assert_eq!(article.title, title);
    assert_eq!(article.body, body);
}

/// Verifica se a operação de registro foi recusada.
#[then("o registro é recusado")]
fn registro_recusado(world: &mut AppWorld) {
    assert_eq!(
        world.news_fixture.last_operation_succeeded,
        Some(false),
        "o registro da notícia deveria ser recusado"
    );
}

/// Verifica se nenhuma notícia foi registrada na operação recusada.
#[then("nenhuma notícia é registrada")]
fn nenhuma_noticia_registrada(world: &mut AppWorld) {
    assert!(
        world.news_fixture.articles.is_empty(),
        "nenhuma notícia deveria constar registrada após a recusa"
    );
}

/// Verifica se a notícia continua fora da listagem publicada.
#[then("a notícia continua fora da listagem publicada")]
fn noticia_continua_fora_listagem_publicada(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia");
    assert_ne!(article.status, NewsStatusState::Publicada);
}

/// Verifica se a notícia continua na listagem publicada.
#[then("a notícia continua na listagem publicada")]
fn noticia_continua_na_listagem_publicada(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia");
    assert_eq!(article.status, NewsStatusState::Publicada);
}

/// Verifica se a data de publicação da notícia permanece registrada.
#[then("a data de publicação da notícia permanece registrada")]
#[then("a data de publicação da notícia fica registrada")]
fn data_publicacao_registrada(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia");
    assert!(article.published_at.is_some());
}

/// Verifica se a alteração / edição foi recusada.
#[then("a alteração é recusada")]
#[then("a edição é recusada")]
fn alteracao_recusada(world: &mut AppWorld) {
    assert_eq!(
        world.news_fixture.last_operation_succeeded,
        Some(false),
        "a alteração ou edição deveria ser recusada"
    );
}

/// Verifica se a notícia permanece com o título e conteúdo anteriores.
#[then(expr = "a notícia permanece com o título {string} e o conteúdo {string}")]
fn noticia_permanece_com_conteudo(world: &mut AppWorld, title: String, body: String) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia");
    assert_eq!(article.title, title);
    assert_eq!(article.body, body);
}

/// Verifica se a notícia entra na listagem publicada do CA informado.
#[then(expr = "a notícia entra na listagem publicada do Centro Acadêmico {string}")]
fn noticia_entra_listagem_publicada(world: &mut AppWorld, ca_name: String) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia");
    assert_eq!(article.ca_name, ca_name);
    assert_eq!(article.status, NewsStatusState::Publicada);
}

/// Verifica se a notícia não fica pendente de homologação.
#[then("a notícia não fica pendente de homologação")]
fn noticia_nao_fica_pendente_homologacao(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia");
    assert_ne!(article.status, NewsStatusState::Solicitacao);
}

/// Verifica se essa notícia não integra a listagem publicada formada.
#[then("essa notícia não integra a listagem publicada")]
fn noticia_nao_integra_listagem_publicada(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia");
    assert!(
        !world.news_fixture.published_listing_contains(article.id),
        "notícia não publicada não deve integrar a listagem publicada"
    );
}

/// Verifica se a notícia permanece rejeitada.
#[then("a notícia permanece rejeitada")]
fn noticia_permanece_rejeitada(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia");
    assert_eq!(article.status, NewsStatusState::Rejeitada);
}

/// Verifica se a publicação ou publicação direta foi recusada.
#[then("a publicação é recusada")]
#[then("a publicação direta é recusada")]
fn publicacao_recusada(world: &mut AppWorld) {
    assert_eq!(
        world.news_fixture.last_operation_succeeded,
        Some(false),
        "a publicação deveria ser recusada"
    );
}

/// Verifica se a notícia não entra na listagem publicada.
#[then("a notícia não entra na listagem publicada")]
fn noticia_nao_entra_listagem_publicada(world: &mut AppWorld) {
    let article = world
        .news_fixture
        .get_current_article()
        .expect("deve haver uma notícia");
    assert_ne!(article.status, NewsStatusState::Publicada);
}
