//! A publicação de materiais permite que integrantes autorizados disponibilizem
//! links externos e arquivos acadêmicos nos Centros Acadêmicos.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. Expressões de contexto
//! e autorização já registradas em outros módulos são reutilizadas.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que uma tentativa anterior de envio do arquivo do material {string} falhou.
#[given(regex = r#"^que uma tentativa anterior de envio do arquivo do material "([^"]*)" falhou$"#)]
fn given_tentativa_anterior_envio_arquivo_falhou(_world: &mut AppWorld, _material: String) {
    pending_step("que uma tentativa anterior de envio do arquivo do material {string} falhou");
}

/// Pendente: o integrante publica o material {string} com o link externo {string}.
#[when(regex = r#"^o integrante publica o material "([^"]*)" com o link externo "([^"]*)"$"#)]
fn when_integrante_publica_material_com_link(
    _world: &mut AppWorld,
    _material: String,
    _link: String,
) {
    pending_step("o integrante publica o material {string} com o link externo {string}");
}

/// Pendente: o integrante tenta publicar o material {string} com um link externo inválido.
#[when(
    regex = r#"^o integrante tenta publicar o material "([^"]*)" com um link externo inválido$"#
)]
fn when_integrante_tenta_publicar_material_com_link_invalido(
    _world: &mut AppWorld,
    _material: String,
) {
    pending_step("o integrante tenta publicar o material {string} com um link externo inválido");
}

/// Pendente: o integrante tenta publicar o material {string} com o link externo {string}.
#[when(
    regex = r#"^o integrante tenta publicar o material "([^"]*)" com o link externo "([^"]*)"$"#
)]
fn when_integrante_tenta_publicar_material_com_link(
    _world: &mut AppWorld,
    _material: String,
    _link: String,
) {
    pending_step("o integrante tenta publicar o material {string} com o link externo {string}");
}

/// Pendente: o integrante publica o material {string} com um arquivo PDF válido.
#[when(regex = r#"^o integrante publica o material "([^"]*)" com um arquivo PDF válido$"#)]
fn when_integrante_publica_material_com_pdf_valido(_world: &mut AppWorld, _material: String) {
    pending_step("o integrante publica o material {string} com um arquivo PDF válido");
}

/// Pendente: o integrante tenta publicar o material {string} com um arquivo em formato não aceito.
#[when(
    regex = r#"^o integrante tenta publicar o material "([^"]*)" com um arquivo em formato não aceito$"#
)]
fn when_integrante_tenta_publicar_material_com_formato_nao_aceito(
    _world: &mut AppWorld,
    _material: String,
) {
    pending_step(
        "o integrante tenta publicar o material {string} com um arquivo em formato não aceito",
    );
}

/// Pendente: o integrante tenta publicar o material {string} com um arquivo acima do limite de tamanho permitido.
#[when(
    regex = r#"^o integrante tenta publicar o material "([^"]*)" com um arquivo acima do limite de tamanho permitido$"#
)]
fn when_integrante_tenta_publicar_material_com_arquivo_acima_limite(
    _world: &mut AppWorld,
    _material: String,
) {
    pending_step(
        "o integrante tenta publicar o material {string} com um arquivo acima do limite de tamanho permitido",
    );
}

/// Pendente: o integrante publica o material {string} com um arquivo no limite de tamanho permitido.
#[when(
    regex = r#"^o integrante publica o material "([^"]*)" com um arquivo no limite de tamanho permitido$"#
)]
fn when_integrante_publica_material_com_arquivo_no_limite(
    _world: &mut AppWorld,
    _material: String,
) {
    pending_step(
        "o integrante publica o material {string} com um arquivo no limite de tamanho permitido",
    );
}

/// Pendente: o integrante tenta publicar o material {string} com um arquivo PDF válido, mas ocorre uma falha durante o envio.
#[when(
    regex = r#"^o integrante tenta publicar o material "([^"]*)" com um arquivo PDF válido, mas ocorre uma falha durante o envio$"#
)]
fn when_integrante_tenta_publicar_pdf_com_falha_no_envio(_world: &mut AppWorld, _material: String) {
    pending_step(
        "o integrante tenta publicar o material {string} com um arquivo PDF válido, mas ocorre uma falha durante o envio",
    );
}

/// Pendente: o integrante realiza uma nova tentativa com um arquivo PDF válido.
#[when(regex = r#"^o integrante realiza uma nova tentativa com um arquivo PDF válido$"#)]
fn when_integrante_realiza_nova_tentativa_com_pdf(_world: &mut AppWorld) {
    pending_step("o integrante realiza uma nova tentativa com um arquivo PDF válido");
}

/// Pendente: o integrante tenta publicar o material {string} com um arquivo PDF válido.
#[when(regex = r#"^o integrante tenta publicar o material "([^"]*)" com um arquivo PDF válido$"#)]
fn when_integrante_tenta_publicar_material_com_pdf_valido(
    _world: &mut AppWorld,
    _material: String,
) {
    pending_step("o integrante tenta publicar o material {string} com um arquivo PDF válido");
}

/// Pendente: a publicação do material é concluída.
#[then(regex = r#"^a publicação do material é concluída$"#)]
fn then_publicacao_material_concluida(_world: &mut AppWorld) {
    pending_step("a publicação do material é concluída");
}

/// Pendente: o endereço do link externo fica associado ao material publicado.
#[then(regex = r#"^o endereço do link externo fica associado ao material publicado$"#)]
fn then_endereco_link_associado_material_publicado(_world: &mut AppWorld) {
    pending_step("o endereço do link externo fica associado ao material publicado");
}

/// Pendente: a publicação do material é recusada.
#[then(regex = r#"^a publicação do material é recusada$"#)]
fn then_publicacao_material_recusada(_world: &mut AppWorld) {
    pending_step("a publicação do material é recusada");
}

/// Pendente: o material não fica disponível como publicado.
#[then(regex = r#"^o material não fica disponível como publicado$"#)]
fn then_material_nao_fica_disponivel_como_publicado(_world: &mut AppWorld) {
    pending_step("o material não fica disponível como publicado");
}

/// Pendente: a publicação do material é recusada por falta de permissão.
#[then(regex = r#"^a publicação do material é recusada por falta de permissão$"#)]
fn then_publicacao_material_recusada_por_falta_permissao(_world: &mut AppWorld) {
    pending_step("a publicação do material é recusada por falta de permissão");
}

/// Pendente: o arquivo PDF fica associado ao material publicado.
#[then(regex = r#"^o arquivo PDF fica associado ao material publicado$"#)]
fn then_arquivo_pdf_associado_material_publicado(_world: &mut AppWorld) {
    pending_step("o arquivo PDF fica associado ao material publicado");
}

/// Pendente: a publicação do material não é concluída.
#[then(regex = r#"^a publicação do material não é concluída$"#)]
fn then_publicacao_material_nao_concluida(_world: &mut AppWorld) {
    pending_step("a publicação do material não é concluída");
}

/// Pendente: o arquivo fica associado ao material publicado.
#[then(regex = r#"^o arquivo fica associado ao material publicado$"#)]
fn then_arquivo_associado_material_publicado(_world: &mut AppWorld) {
    pending_step("o arquivo fica associado ao material publicado");
}
