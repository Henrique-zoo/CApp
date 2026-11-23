//! O cadastro de um Centro Acadêmico inicia sua utilização pela comunidade com informações básicas e responsáveis identificáveis.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: que encerrei minha sessão no aplicativo.
#[given(regex = r#"^que encerrei minha sessão no aplicativo$"#)]
fn given_que_encerrei_minha_sessao_no_aplicativo(_world: &mut AppWorld) {
    pending_step("que encerrei minha sessão no aplicativo");
}

/// Pendente: a operação deve ser recusada informando que a autenticação é obrigatória.
#[then(regex = r#"^a operação deve ser recusada informando que a autenticação é obrigatória$"#)]
fn then_a_operacao_deve_ser_recusada_informando_que_a_autenticacao_e_obrigatoria(
    _world: &mut AppWorld,
) {
    pending_step("a operação deve ser recusada informando que a autenticação é obrigatória");
}

/// Pendente: nenhuma associação à gestão inicial de {string} deve ser registrada.
#[then(regex = r#"^nenhuma associação à gestão inicial de "([^"]*)" deve ser registrada$"#)]
fn then_nenhuma_associacao_a_gestao_inicial_de_valor_deve_ser_registrada(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("nenhuma associação à gestão inicial de {string} deve ser registrada");
}

/// Pendente: nenhuma associação à gestão inicial deve ser registrada.
#[then(regex = r#"^nenhuma associação à gestão inicial deve ser registrada$"#)]
fn then_nenhuma_associacao_a_gestao_inicial_deve_ser_registrada(_world: &mut AppWorld) {
    pending_step("nenhuma associação à gestão inicial deve ser registrada");
}

/// Pendente: o CA com o nome oficial {string} deve ser criado.
#[then(regex = r#"^o CA com o nome oficial "([^"]*)" deve ser criado$"#)]
fn then_o_ca_com_o_nome_oficial_valor_deve_ser_criado(_world: &mut AppWorld, _value_1: String) {
    pending_step("o CA com o nome oficial {string} deve ser criado");
}

/// Pendente: o CA deve permanecer sem sigla.
#[then(regex = r#"^o CA deve permanecer sem sigla$"#)]
fn then_o_ca_deve_permanecer_sem_sigla(_world: &mut AppWorld) {
    pending_step("o CA deve permanecer sem sigla");
}

/// Pendente: o CA deve possuir a descrição institucional informada.
#[then(regex = r#"^o CA deve possuir a descrição institucional informada$"#)]
fn then_o_ca_deve_possuir_a_descricao_institucional_informada(_world: &mut AppWorld) {
    pending_step("o CA deve possuir a descrição institucional informada");
}

/// Pendente: o CA deve possuir a sigla {string}.
#[then(regex = r#"^o CA deve possuir a sigla "([^"]*)"$"#)]
fn then_o_ca_deve_possuir_a_sigla_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o CA deve possuir a sigla {string}");
}

/// Pendente: o CA deve possuir o nome oficial {string}.
#[then(regex = r#"^o CA deve possuir o nome oficial "([^"]*)"$"#)]
fn then_o_ca_deve_possuir_o_nome_oficial_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("o CA deve possuir o nome oficial {string}");
}

/// Pendente: o Centro Acadêmico {string} deve ser criado no CApp.
#[then(regex = r#"^o Centro Acadêmico "([^"]*)" deve ser criado no CApp$"#)]
fn then_o_centro_academico_valor_deve_ser_criado_no_capp(_world: &mut AppWorld, _value_1: String) {
    pending_step("o Centro Acadêmico {string} deve ser criado no CApp");
}

/// Pendente: o Centro Acadêmico {string} não deve ser criado.
#[then(regex = r#"^o Centro Acadêmico "([^"]*)" não deve ser criado$"#)]
fn then_o_centro_academico_valor_nao_deve_ser_criado(_world: &mut AppWorld, _value_1: String) {
    pending_step("o Centro Acadêmico {string} não deve ser criado");
}

/// Pendente: o cadastro deve ser concluído sem exigir importação ou migração de dados anteriores.
#[then(
    regex = r#"^o cadastro deve ser concluído sem exigir importação ou migração de dados anteriores$"#
)]
fn then_o_cadastro_deve_ser_concluido_sem_exigir_importacao_ou_migracao_de_dados_anteriores(
    _world: &mut AppWorld,
) {
    pending_step(
        "o cadastro deve ser concluído sem exigir importação ou migração de dados anteriores",
    );
}

/// Pendente: o cadastro deve ser recusado informando que o nome oficial é obrigatório.
#[then(regex = r#"^o cadastro deve ser recusado informando que o nome oficial é obrigatório$"#)]
fn then_o_cadastro_deve_ser_recusado_informando_que_o_nome_oficial_e_obrigatorio(
    _world: &mut AppWorld,
) {
    pending_step("o cadastro deve ser recusado informando que o nome oficial é obrigatório");
}

/// Pendente: o cadastro deve ser recusado informando que o usuário não foi encontrado.
#[then(regex = r#"^o cadastro deve ser recusado informando que o usuário não foi encontrado$"#)]
fn then_o_cadastro_deve_ser_recusado_informando_que_o_usuario_nao_foi_encontrado(
    _world: &mut AppWorld,
) {
    pending_step("o cadastro deve ser recusado informando que o usuário não foi encontrado");
}

/// Pendente: o estudante deve constar como responsável inicial pela administração do CA criado.
#[then(
    regex = r#"^o estudante deve constar como responsável inicial pela administração do CA criado$"#
)]
fn then_o_estudante_deve_constar_como_responsavel_inicial_pela_administracao_do_ca_criado(
    _world: &mut AppWorld,
) {
    pending_step(
        "o estudante deve constar como responsável inicial pela administração do CA criado",
    );
}

/// Pendente: o estudante {string} deve constar como responsável inicial pela administração de {string}.
#[then(
    regex = r#"^o estudante "([^"]*)" deve constar como responsável inicial pela administração de "([^"]*)"$"#
)]
fn then_o_estudante_valor_deve_constar_como_responsavel_inicial_pela_administracao_de_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "o estudante {string} deve constar como responsável inicial pela administração de {string}",
    );
}

/// Pendente: confirma o cadastro.
#[when(regex = r#"^confirma o cadastro$"#)]
fn when_confirma_o_cadastro(_world: &mut AppWorld) {
    pending_step("confirma o cadastro");
}

/// Pendente: indica a si mesmo como responsável inicial pela administração.
#[when(regex = r#"^indica a si mesmo como responsável inicial pela administração$"#)]
fn when_indica_a_si_mesmo_como_responsavel_inicial_pela_administracao(_world: &mut AppWorld) {
    pending_step("indica a si mesmo como responsável inicial pela administração");
}

/// Pendente: indica {string} como responsável inicial pela administração.
#[when(regex = r#"^indica "([^"]*)" como responsável inicial pela administração$"#)]
fn when_indica_valor_como_responsavel_inicial_pela_administracao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("indica {string} como responsável inicial pela administração");
}

/// Pendente: informa o nome oficial {string}.
#[when(regex = r#"^informa o nome oficial "([^"]*)"$"#)]
fn when_informa_o_nome_oficial_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("informa o nome oficial {string}");
}

/// Pendente: informa os seguintes dados básicos para o novo CA:.
#[when(regex = r#"^informa os seguintes dados básicos para o novo CA:$"#)]
fn when_informa_os_seguintes_dados_basicos_para_o_novo_ca(_world: &mut AppWorld) {
    pending_step("informa os seguintes dados básicos para o novo CA:");
}

/// Pendente: não informa uma sigla para o novo CA.
#[when(regex = r#"^não informa uma sigla para o novo CA$"#)]
fn when_nao_informa_uma_sigla_para_o_novo_ca(_world: &mut AppWorld) {
    pending_step("não informa uma sigla para o novo CA");
}

/// Pendente: tenta cadastrar o CA {string}.
#[when(regex = r#"^tenta cadastrar o CA "([^"]*)"$"#)]
fn when_tenta_cadastrar_o_ca_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("tenta cadastrar o CA {string}");
}

/// Pendente: tenta cadastrar um CA com a sigla {string} sem informar o nome oficial.
#[when(regex = r#"^tenta cadastrar um CA com a sigla "([^"]*)" sem informar o nome oficial$"#)]
fn when_tenta_cadastrar_um_ca_com_a_sigla_valor_sem_informar_o_nome_oficial(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("tenta cadastrar um CA com a sigla {string} sem informar o nome oficial");
}

/// Pendente: tento cadastrar um Centro Acadêmico com a sigla {string}.
#[when(regex = r#"^tento cadastrar um Centro Acadêmico com a sigla "([^"]*)"$"#)]
fn when_tento_cadastrar_um_centro_academico_com_a_sigla_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("tento cadastrar um Centro Acadêmico com a sigla {string}");
}
