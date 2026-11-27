//! Registra os steps de I06 (links) e I11 (arquivos) da publicação de materiais.
//!
//! A automação funcional acompanha as entregas de publicação, conteúdo e consulta.
//! Os steps de contexto acadêmico e autorização comuns usam os registros existentes.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Prepara um estudante que pode consultar os materiais de cada CA indicado.
#[given(
    regex = r#"^em cada um dos CAs "([^"]*)" e "([^"]*)" há um estudante com acesso à consulta e sem permissão de publicação$"#
)]
fn given_estudantes_com_acesso_aos_materiais(
    _world: &mut AppWorld,
    _primeiro_ca: String,
    _segundo_ca: String,
) {
    pending_step(
        "em cada um dos CAs {string} e {string} há um estudante com acesso à consulta e sem permissão de publicação",
    );
}

/// Prepara uma permissão que vale apenas no CA indicado.
#[given(regex = r#"^que a permissão "([^"]*)" do integrante vale somente no "([^"]*)"$"#)]
fn given_permissao_somente_em_outro_ca(_world: &mut AppWorld, _permissao: String, _ca: String) {
    pending_step("que a permissão {string} do integrante vale somente no {string}");
}

/// Prepara um PDF conforme a posição indicada em relação ao limite de tamanho.
#[given(
    regex = r#"^o arquivo "([^"]*)" é um PDF válido (abaixo do limite de tamanho|exatamente no limite de tamanho|1 byte acima do limite de tamanho)$"#
)]
fn given_pdf_com_tamanho(_world: &mut AppWorld, _arquivo: String, _tamanho: String) {
    pending_step("o arquivo {string} é um PDF válido na faixa de tamanho informada");
}

/// Prepara um arquivo que só infringe a regra de formato.
#[given(
    regex = r#"^o arquivo "([^"]*)" tem tamanho abaixo do limite e formato não aceito para materiais$"#
)]
fn given_arquivo_com_formato_nao_aceito(_world: &mut AppWorld, _arquivo: String) {
    pending_step(
        "o arquivo {string} tem tamanho abaixo do limite e formato não aceito para materiais",
    );
}

/// Prepara uma falha de envio para a tentativa em curso.
#[given(regex = r"^o envio do arquivo falhará nesta tentativa$")]
fn given_envio_falhara(_world: &mut AppWorld) {
    pending_step("o envio do arquivo falhará nesta tentativa");
}

/// Prepara uma tentativa anterior de envio malsucedida para o mesmo material.
#[given(regex = r#"^o envio anterior de "([^"]*)" para publicar "([^"]*)" no CA ativo falhou$"#)]
fn given_envio_anterior_falhou(_world: &mut AppWorld, _arquivo: String, _material: String) {
    pending_step("o envio anterior de {string} para publicar {string} no CA ativo falhou");
}

/// Solicita a publicação de um link externo no contexto acadêmico ativo.
#[when(
    regex = r#"^o integrante solicita a publicação de "([^"]*)" em "([^"]*)" com o link "([^"]*)"$"#
)]
fn when_solicita_publicacao_com_link(
    _world: &mut AppWorld,
    _material: String,
    _disciplina: String,
    _endereco: String,
) {
    pending_step("o integrante solicita a publicação de {string} em {string} com o link {string}");
}

/// Solicita a publicação de um arquivo no contexto acadêmico ativo.
#[when(
    regex = r#"^o integrante solicita a publicação de "([^"]*)" em "([^"]*)" com o arquivo "([^"]*)"$"#
)]
fn when_solicita_publicacao_com_arquivo(
    _world: &mut AppWorld,
    _material: String,
    _disciplina: String,
    _arquivo: String,
) {
    pending_step(
        "o integrante solicita a publicação de {string} em {string} com o arquivo {string}",
    );
}

/// Confere a visibilidade do material para um estudante sem poderes de gestão.
#[then(
    regex = r#"^um estudante sem permissão de publicação encontra "([^"]*)" em "([^"]*)" no "([^"]*)"$"#
)]
fn then_estudante_encontra_material(
    _world: &mut AppWorld,
    _material: String,
    _disciplina: String,
    _ca: String,
) {
    pending_step(
        "um estudante sem permissão de publicação encontra {string} em {string} no {string}",
    );
}

/// Confere que o endereço publicado é recuperável.
#[then(
    regex = r#"^o estudante consegue acessar o endereço "([^"]*)" a partir do material publicado$"#
)]
fn then_estudante_acessa_link(_world: &mut AppWorld, _endereco: String) {
    pending_step("o estudante consegue acessar o endereço {string} a partir do material publicado");
}

/// Confere que o PDF publicado é recuperável.
#[then(regex = r#"^o estudante consegue acessar o PDF "([^"]*)" a partir do material publicado$"#)]
fn then_estudante_acessa_pdf(_world: &mut AppWorld, _arquivo: String) {
    pending_step("o estudante consegue acessar o PDF {string} a partir do material publicado");
}

/// Confere que o material não está disponível na consulta do CA indicado.
#[then(regex = r#"^o material "([^"]*)" não aparece entre os materiais publicados do "([^"]*)"$"#)]
fn then_material_ausente_da_consulta(_world: &mut AppWorld, _material: String, _ca: String) {
    pending_step("o material {string} não aparece entre os materiais publicados do {string}");
}

/// Confere a razão da recusa comunicada ao solicitante.
#[then(
    regex = r"^a publicação do material é recusada por (endereço externo inválido|falta de permissão|disciplina incompatível com o CA ativo|formato de arquivo não aceito|tamanho de arquivo acima do limite)$"
)]
fn then_publicacao_recusada(_world: &mut AppWorld, _motivo: String) {
    pending_step("a publicação do material é recusada pelo motivo informado");
}

/// Confere que a falha de envio é comunicada ao integrante.
#[then(regex = r"^o integrante é informado de que o envio do arquivo falhou$")]
fn then_integrante_informado_da_falha(_world: &mut AppWorld) {
    pending_step("o integrante é informado de que o envio do arquivo falhou");
}

/// Confere que a nova tentativa produz uma única publicação para consulta.
#[then(
    regex = r#"^um estudante sem permissão de publicação encontra uma única publicação de "([^"]*)" em "([^"]*)" no "([^"]*)"$"#
)]
fn then_estudante_encontra_publicacao_unica(
    _world: &mut AppWorld,
    _material: String,
    _disciplina: String,
    _ca: String,
) {
    pending_step(
        "um estudante sem permissão de publicação encontra uma única publicação de {string} em {string} no {string}",
    );
}
