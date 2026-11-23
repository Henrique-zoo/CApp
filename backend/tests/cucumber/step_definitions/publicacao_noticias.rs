//! A publicação de notícias depende das permissões locais e da homologação das propostas e alterações.
//!
//! Estes registros ligam os cenários ao runner enquanto a automação real está
//! pendente. Cada passo chama `pending_step` e falha explicitamente se executado.
//! Não prepara estado nem simula resultados do produto. As expressões reutilizadas
//! por outras funcionalidades ficam no módulo compartilhado, com registro único.

use cucumber::{given, then, when};

use super::pending_step;
use crate::support::AppWorld;

/// Pendente: ele está preparando uma notícia com o título {string} e o conteúdo {string}.
#[given(
    regex = r#"^ele está preparando uma notícia com o título "([^"]*)" e o conteúdo "([^"]*)"$"#
)]
fn given_ele_esta_preparando_uma_noticia_com_o_titulo_valor_e_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("ele está preparando uma notícia com o título {string} e o conteúdo {string}");
}

/// Pendente: existe uma proposta de notícia de outro usuário aguardando homologação.
#[given(regex = r#"^existe uma proposta de notícia de outro usuário aguardando homologação$"#)]
fn given_existe_uma_proposta_de_noticia_de_outro_usuario_aguardando_homologacao(
    _world: &mut AppWorld,
) {
    pending_step("existe uma proposta de notícia de outro usuário aguardando homologação");
}

/// Pendente: nesse Centro Acadêmico existe uma notícia já publicada com o título {string} e o conteúdo {string}.
#[given(
    regex = r#"^nesse Centro Acadêmico existe uma notícia já publicada com o título "([^"]*)" e o conteúdo "([^"]*)"$"#
)]
fn given_nesse_centro_academico_existe_uma_noticia_ja_publicada_com_o_titulo_valor_e_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "nesse Centro Acadêmico existe uma notícia já publicada com o título {string} e o conteúdo {string}",
    );
}

/// Pendente: nesse Centro Acadêmico existe uma notícia não publicada com o título {string} e o conteúdo {string}.
#[given(
    regex = r#"^nesse Centro Acadêmico existe uma notícia não publicada com o título "([^"]*)" e o conteúdo "([^"]*)"$"#
)]
fn given_nesse_centro_academico_existe_uma_noticia_nao_publicada_com_o_titulo_valor_e_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "nesse Centro Acadêmico existe uma notícia não publicada com o título {string} e o conteúdo {string}",
    );
}

/// Pendente: nesse Centro Acadêmico existe uma notícia não publicada de sua autoria com o título {string} e o conteúdo {string}.
#[given(
    regex = r#"^nesse Centro Acadêmico existe uma notícia não publicada de sua autoria com o título "([^"]*)" e o conteúdo "([^"]*)"$"#
)]
fn given_nesse_centro_academico_existe_uma_noticia_nao_publicada_de_sua_autoria_com_o_titulo_valor_e_o_conteu(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step(
        "nesse Centro Acadêmico existe uma notícia não publicada de sua autoria com o título {string} e o conteúdo {string}",
    );
}

/// Pendente: não possui permissão de publicação direta de notícias nesse CA.
#[given(regex = r#"^não possui permissão de publicação direta de notícias nesse CA$"#)]
fn given_nao_possui_permissao_de_publicacao_direta_de_noticias_nesse_ca(_world: &mut AppWorld) {
    pending_step("não possui permissão de publicação direta de notícias nesse CA");
}

/// Pendente: o Centro Acadêmico {string} possui uma notícia não publicada com o título {string} e o conteúdo {string}.
#[given(
    regex = r#"^o Centro Acadêmico "([^"]*)" possui uma notícia não publicada com o título "([^"]*)" e o conteúdo "([^"]*)"$"#
)]
fn given_o_centro_academico_valor_possui_uma_noticia_nao_publicada_com_o_titulo_valor_e_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
    _value_3: String,
) {
    pending_step(
        "o Centro Acadêmico {string} possui uma notícia não publicada com o título {string} e o conteúdo {string}",
    );
}

/// Pendente: o usuário não possui publicação direta nem homologação de notícias em {string}.
#[given(
    regex = r#"^o usuário não possui publicação direta nem homologação de notícias em "([^"]*)"$"#
)]
fn given_o_usuario_nao_possui_publicacao_direta_nem_homologacao_de_noticias_em_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("o usuário não possui publicação direta nem homologação de notícias em {string}");
}

/// Pendente: que o Centro Acadêmico {string} possui uma notícia pendente de homologação.
#[given(regex = r#"^que o Centro Acadêmico "([^"]*)" possui uma notícia pendente de homologação$"#)]
fn given_que_o_centro_academico_valor_possui_uma_noticia_pendente_de_homologacao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que o Centro Acadêmico {string} possui uma notícia pendente de homologação");
}

/// Pendente: que o Centro Acadêmico {string} possui uma notícia rejeitada.
#[given(regex = r#"^que o Centro Acadêmico "([^"]*)" possui uma notícia rejeitada$"#)]
fn given_que_o_centro_academico_valor_possui_uma_noticia_rejeitada(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que o Centro Acadêmico {string} possui uma notícia rejeitada");
}

/// Pendente: que o gestor do Centro Acadêmico {string} não possui permissão de publicação direta.
#[given(
    regex = r#"^que o gestor do Centro Acadêmico "([^"]*)" não possui permissão de publicação direta$"#
)]
fn given_que_o_gestor_do_centro_academico_valor_nao_possui_permissao_de_publicacao_direta(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "que o gestor do Centro Acadêmico {string} não possui permissão de publicação direta",
    );
}

/// Pendente: que o integrante da gestão não possui publicação direta nem homologação de notícias no CA selecionado.
#[given(
    regex = r#"^que o integrante da gestão não possui publicação direta nem homologação de notícias no CA selecionado$"#
)]
fn given_que_o_integrante_da_gestao_nao_possui_publicacao_direta_nem_homologacao_de_noticias_no_ca_selecionad(
    _world: &mut AppWorld,
) {
    pending_step(
        "que o integrante da gestão não possui publicação direta nem homologação de notícias no CA selecionado",
    );
}

/// Pendente: que o integrante possui a permissão {string} no CA selecionado.
#[given(regex = r#"^que o integrante possui a permissão "([^"]*)" no CA selecionado$"#)]
fn given_que_o_integrante_possui_a_permissao_valor_no_ca_selecionado(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que o integrante possui a permissão {string} no CA selecionado");
}

/// Pendente: que o integrante possui somente a permissão {string} no CA selecionado.
#[given(regex = r#"^que o integrante possui somente a permissão "([^"]*)" no CA selecionado$"#)]
fn given_que_o_integrante_possui_somente_a_permissao_valor_no_ca_selecionado(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("que o integrante possui somente a permissão {string} no CA selecionado");
}

/// Pendente: que o usuário está autenticado no CApp.
#[given(regex = r#"^que o usuário está autenticado no CApp$"#)]
fn given_que_o_usuario_esta_autenticado_no_capp(_world: &mut AppWorld) {
    pending_step("que o usuário está autenticado no CApp");
}

/// Pendente: que o usuário não possui permissão de publicação direta nem de homologação de notícias no Centro Acadêmico {string}.
#[given(
    regex = r#"^que o usuário não possui permissão de publicação direta nem de homologação de notícias no Centro Acadêmico "([^"]*)"$"#
)]
fn given_que_o_usuario_nao_possui_permissao_de_publicacao_direta_nem_de_homologacao_de_noticias_no_centro_aca(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "que o usuário não possui permissão de publicação direta nem de homologação de notícias no Centro Acadêmico {string}",
    );
}

/// Pendente: que o usuário possui permissão de publicação direta de notícias no Centro Acadêmico {string}.
#[given(
    regex = r#"^que o usuário possui permissão de publicação direta de notícias no Centro Acadêmico "([^"]*)"$"#
)]
fn given_que_o_usuario_possui_permissao_de_publicacao_direta_de_noticias_no_centro_academico_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "que o usuário possui permissão de publicação direta de notícias no Centro Acadêmico {string}",
    );
}

/// Pendente: que o usuário possui permissão de publicação direta no Centro Acadêmico {string}.
#[given(
    regex = r#"^que o usuário possui permissão de publicação direta no Centro Acadêmico "([^"]*)"$"#
)]
fn given_que_o_usuario_possui_permissao_de_publicacao_direta_no_centro_academico_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step(
        "que o usuário possui permissão de publicação direta no Centro Acadêmico {string}",
    );
}

/// Pendente: que uma notícia foi retirada da listagem após uma edição.
#[given(regex = r#"^que uma notícia foi retirada da listagem após uma edição$"#)]
fn given_que_uma_noticia_foi_retirada_da_listagem_apos_uma_edicao(_world: &mut AppWorld) {
    pending_step("que uma notícia foi retirada da listagem após uma edição");
}

/// Pendente: selecionou o contexto {string}.
#[given(regex = r#"^selecionou o contexto "([^"]*)"$"#)]
fn given_selecionou_o_contexto_valor(_world: &mut AppWorld, _value_1: String) {
    pending_step("selecionou o contexto {string}");
}

/// Pendente: selecionou um Centro Acadêmico.
#[given(regex = r#"^selecionou um Centro Acadêmico$"#)]
fn given_selecionou_um_centro_academico(_world: &mut AppWorld) {
    pending_step("selecionou um Centro Acadêmico");
}

/// Pendente: sua alteração aguarda nova homologação no CA selecionado.
#[given(regex = r#"^sua alteração aguarda nova homologação no CA selecionado$"#)]
fn given_sua_alteracao_aguarda_nova_homologacao_no_ca_selecionado(_world: &mut AppWorld) {
    pending_step("sua alteração aguarda nova homologação no CA selecionado");
}

/// Pendente: a alteração só pode voltar a ser publicada após homologação por um integrante autorizado.
#[then(
    regex = r#"^a alteração só pode voltar a ser publicada após homologação por um integrante autorizado$"#
)]
fn then_a_alteracao_so_pode_voltar_a_ser_publicada_apos_homologacao_por_um_integrante_autorizado(
    _world: &mut AppWorld,
) {
    pending_step(
        "a alteração só pode voltar a ser publicada após homologação por um integrante autorizado",
    );
}

/// Pendente: a alteração é recusada.
#[then(regex = r#"^a alteração é recusada$"#)]
fn then_a_alteracao_e_recusada(_world: &mut AppWorld) {
    pending_step("a alteração é recusada");
}

/// Pendente: a data de criação da notícia fica registrada.
#[then(regex = r#"^a data de criação da notícia fica registrada$"#)]
fn then_a_data_de_criacao_da_noticia_fica_registrada(_world: &mut AppWorld) {
    pending_step("a data de criação da notícia fica registrada");
}

/// Pendente: a data de publicação da notícia fica registrada.
#[then(regex = r#"^a data de publicação da notícia fica registrada$"#)]
fn then_a_data_de_publicacao_da_noticia_fica_registrada(_world: &mut AppWorld) {
    pending_step("a data de publicação da notícia fica registrada");
}

/// Pendente: a edição é recusada.
#[then(regex = r#"^a edição é recusada$"#)]
fn then_a_edicao_e_recusada(_world: &mut AppWorld) {
    pending_step("a edição é recusada");
}

/// Pendente: a homologação é recusada por falta da permissão correspondente.
#[then(regex = r#"^a homologação é recusada por falta da permissão correspondente$"#)]
fn then_a_homologacao_e_recusada_por_falta_da_permissao_correspondente(_world: &mut AppWorld) {
    pending_step("a homologação é recusada por falta da permissão correspondente");
}

/// Pendente: a notícia continua fora da listagem publicada.
#[then(regex = r#"^a notícia continua fora da listagem publicada$"#)]
fn then_a_noticia_continua_fora_da_listagem_publicada(_world: &mut AppWorld) {
    pending_step("a notícia continua fora da listagem publicada");
}

/// Pendente: a notícia entra na listagem publicada do Centro Acadêmico {string}.
#[then(regex = r#"^a notícia entra na listagem publicada do Centro Acadêmico "([^"]*)"$"#)]
fn then_a_noticia_entra_na_listagem_publicada_do_centro_academico_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a notícia entra na listagem publicada do Centro Acadêmico {string}");
}

/// Pendente: a notícia fica pendente de homologação.
#[then(regex = r#"^a notícia fica pendente de homologação$"#)]
fn then_a_noticia_fica_pendente_de_homologacao(_world: &mut AppWorld) {
    pending_step("a notícia fica pendente de homologação");
}

/// Pendente: a notícia fica pendente de homologação por um integrante autorizado a homologar notícias.
#[then(
    regex = r#"^a notícia fica pendente de homologação por um integrante autorizado a homologar notícias$"#
)]
fn then_a_noticia_fica_pendente_de_homologacao_por_um_integrante_autorizado_a_homologar_noticias(
    _world: &mut AppWorld,
) {
    pending_step(
        "a notícia fica pendente de homologação por um integrante autorizado a homologar notícias",
    );
}

/// Pendente: a notícia fica pendente de nova homologação.
#[then(regex = r#"^a notícia fica pendente de nova homologação$"#)]
fn then_a_noticia_fica_pendente_de_nova_homologacao(_world: &mut AppWorld) {
    pending_step("a notícia fica pendente de nova homologação");
}

/// Pendente: a notícia fica registrada com o título {string} e o conteúdo {string}.
#[then(regex = r#"^a notícia fica registrada com o título "([^"]*)" e o conteúdo "([^"]*)"$"#)]
fn then_a_noticia_fica_registrada_com_o_titulo_valor_e_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a notícia fica registrada com o título {string} e o conteúdo {string}");
}

/// Pendente: a notícia fica registrada no Centro Acadêmico {string} como solicitação.
#[then(regex = r#"^a notícia fica registrada no Centro Acadêmico "([^"]*)" como solicitação$"#)]
fn then_a_noticia_fica_registrada_no_centro_academico_valor_como_solicitacao(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a notícia fica registrada no Centro Acadêmico {string} como solicitação");
}

/// Pendente: a notícia não aparece na listagem publicada.
#[then(regex = r#"^a notícia não aparece na listagem publicada$"#)]
fn then_a_noticia_nao_aparece_na_listagem_publicada(_world: &mut AppWorld) {
    pending_step("a notícia não aparece na listagem publicada");
}

/// Pendente: a notícia não aparece para os usuários.
#[then(regex = r#"^a notícia não aparece para os usuários$"#)]
fn then_a_noticia_nao_aparece_para_os_usuarios(_world: &mut AppWorld) {
    pending_step("a notícia não aparece para os usuários");
}

/// Pendente: a notícia não aparece para os usuários do Centro Acadêmico {string}.
#[then(regex = r#"^a notícia não aparece para os usuários do Centro Acadêmico "([^"]*)"$"#)]
fn then_a_noticia_nao_aparece_para_os_usuarios_do_centro_academico_valor(
    _world: &mut AppWorld,
    _value_1: String,
) {
    pending_step("a notícia não aparece para os usuários do Centro Acadêmico {string}");
}

/// Pendente: a notícia não entra na listagem publicada.
#[then(regex = r#"^a notícia não entra na listagem publicada$"#)]
fn then_a_noticia_nao_entra_na_listagem_publicada(_world: &mut AppWorld) {
    pending_step("a notícia não entra na listagem publicada");
}

/// Pendente: a notícia não fica pendente de homologação.
#[then(regex = r#"^a notícia não fica pendente de homologação$"#)]
fn then_a_noticia_nao_fica_pendente_de_homologacao(_world: &mut AppWorld) {
    pending_step("a notícia não fica pendente de homologação");
}

/// Pendente: a notícia não possui data de publicação.
#[then(regex = r#"^a notícia não possui data de publicação$"#)]
fn then_a_noticia_nao_possui_data_de_publicacao(_world: &mut AppWorld) {
    pending_step("a notícia não possui data de publicação");
}

/// Pendente: a notícia permanece com o título {string} e o conteúdo {string}.
#[then(regex = r#"^a notícia permanece com o título "([^"]*)" e o conteúdo "([^"]*)"$"#)]
fn then_a_noticia_permanece_com_o_titulo_valor_e_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("a notícia permanece com o título {string} e o conteúdo {string}");
}

/// Pendente: a notícia permanece pendente de homologação.
#[then(regex = r#"^a notícia permanece pendente de homologação$"#)]
fn then_a_noticia_permanece_pendente_de_homologacao(_world: &mut AppWorld) {
    pending_step("a notícia permanece pendente de homologação");
}

/// Pendente: a notícia permanece pendente de homologação por um integrante autorizado a homologar notícias.
#[then(
    regex = r#"^a notícia permanece pendente de homologação por um integrante autorizado a homologar notícias$"#
)]
fn then_a_noticia_permanece_pendente_de_homologacao_por_um_integrante_autorizado_a_homologar_noticias(
    _world: &mut AppWorld,
) {
    pending_step(
        "a notícia permanece pendente de homologação por um integrante autorizado a homologar notícias",
    );
}

/// Pendente: a notícia permanece rejeitada.
#[then(regex = r#"^a notícia permanece rejeitada$"#)]
fn then_a_noticia_permanece_rejeitada(_world: &mut AppWorld) {
    pending_step("a notícia permanece rejeitada");
}

/// Pendente: a notícia sai da listagem publicada enquanto aguarda a aprovação.
#[then(regex = r#"^a notícia sai da listagem publicada enquanto aguarda a aprovação$"#)]
fn then_a_noticia_sai_da_listagem_publicada_enquanto_aguarda_a_aprovacao(_world: &mut AppWorld) {
    pending_step("a notícia sai da listagem publicada enquanto aguarda a aprovação");
}

/// Pendente: a notícia volta à listagem publicada com o conteúdo homologado.
#[then(regex = r#"^a notícia volta à listagem publicada com o conteúdo homologado$"#)]
fn then_a_noticia_volta_a_listagem_publicada_com_o_conteudo_homologado(_world: &mut AppWorld) {
    pending_step("a notícia volta à listagem publicada com o conteúdo homologado");
}

/// Pendente: a notícia é publicada sem homologação prévia.
#[then(regex = r#"^a notícia é publicada sem homologação prévia$"#)]
fn then_a_noticia_e_publicada_sem_homologacao_previa(_world: &mut AppWorld) {
    pending_step("a notícia é publicada sem homologação prévia");
}

/// Pendente: a proposta permanece pendente e fora da listagem publicada.
#[then(regex = r#"^a proposta permanece pendente e fora da listagem publicada$"#)]
fn then_a_proposta_permanece_pendente_e_fora_da_listagem_publicada(_world: &mut AppWorld) {
    pending_step("a proposta permanece pendente e fora da listagem publicada");
}

/// Pendente: a publicação direta é recusada.
#[then(regex = r#"^a publicação direta é recusada$"#)]
fn then_a_publicacao_direta_e_recusada(_world: &mut AppWorld) {
    pending_step("a publicação direta é recusada");
}

/// Pendente: a publicação é recusada.
#[then(regex = r#"^a publicação é recusada$"#)]
fn then_a_publicacao_e_recusada(_world: &mut AppWorld) {
    pending_step("a publicação é recusada");
}

/// Pendente: essa notícia não integra a listagem publicada.
#[then(regex = r#"^essa notícia não integra a listagem publicada$"#)]
fn then_essa_noticia_nao_integra_a_listagem_publicada(_world: &mut AppWorld) {
    pending_step("essa notícia não integra a listagem publicada");
}

/// Pendente: nenhuma notícia é registrada.
#[then(regex = r#"^nenhuma notícia é registrada$"#)]
fn then_nenhuma_noticia_e_registrada(_world: &mut AppWorld) {
    pending_step("nenhuma notícia é registrada");
}

/// Pendente: o registro da notícia é aceito.
#[then(regex = r#"^o registro da notícia é aceito$"#)]
fn then_o_registro_da_noticia_e_aceito(_world: &mut AppWorld) {
    pending_step("o registro da notícia é aceito");
}

/// Pendente: o registro é recusado.
#[then(regex = r#"^o registro é recusado$"#)]
fn then_o_registro_e_recusado(_world: &mut AppWorld) {
    pending_step("o registro é recusado");
}

/// Pendente: o usuário fica vinculado como autor da notícia.
#[then(regex = r#"^o usuário fica vinculado como autor da notícia$"#)]
fn then_o_usuario_fica_vinculado_como_autor_da_noticia(_world: &mut AppWorld) {
    pending_step("o usuário fica vinculado como autor da notícia");
}

/// Pendente: a listagem de notícias publicadas desse Centro Acadêmico é formada.
#[when(regex = r#"^a listagem de notícias publicadas desse Centro Acadêmico é formada$"#)]
fn when_a_listagem_de_noticias_publicadas_desse_centro_academico_e_formada(_world: &mut AppWorld) {
    pending_step("a listagem de notícias publicadas desse Centro Acadêmico é formada");
}

/// Pendente: aprova a alteração.
#[when(regex = r#"^aprova a alteração$"#)]
fn when_aprova_a_alteracao(_world: &mut AppWorld) {
    pending_step("aprova a alteração");
}

/// Pendente: cria e publica diretamente uma notícia própria com título e conteúdo válidos.
#[when(regex = r#"^cria e publica diretamente uma notícia própria com título e conteúdo válidos$"#)]
fn when_cria_e_publica_diretamente_uma_noticia_propria_com_titulo_e_conteudo_validos(
    _world: &mut AppWorld,
) {
    pending_step("cria e publica diretamente uma notícia própria com título e conteúdo válidos");
}

/// Pendente: cria uma notícia com título e conteúdo válidos.
#[when(regex = r#"^cria uma notícia com título e conteúdo válidos$"#)]
fn when_cria_uma_noticia_com_titulo_e_conteudo_validos(_world: &mut AppWorld) {
    pending_step("cria uma notícia com título e conteúdo válidos");
}

/// Pendente: ele altera o título para {string} e o conteúdo para {string}.
#[when(regex = r#"^ele altera o título para "([^"]*)" e o conteúdo para "([^"]*)"$"#)]
fn when_ele_altera_o_titulo_para_valor_e_o_conteudo_para_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("ele altera o título para {string} e o conteúdo para {string}");
}

/// Pendente: ele cria uma notícia com o título {string} e o conteúdo {string}.
#[when(regex = r#"^ele cria uma notícia com o título "([^"]*)" e o conteúdo "([^"]*)"$"#)]
fn when_ele_cria_uma_noticia_com_o_titulo_valor_e_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("ele cria uma notícia com o título {string} e o conteúdo {string}");
}

/// Pendente: ele publica essa notícia diretamente.
#[when(regex = r#"^ele publica essa notícia diretamente$"#)]
fn when_ele_publica_essa_noticia_diretamente(_world: &mut AppWorld) {
    pending_step("ele publica essa notícia diretamente");
}

/// Pendente: ele tenta alterar a notícia para o título {string} e o conteúdo {string}.
#[when(regex = r#"^ele tenta alterar a notícia para o título "([^"]*)" e o conteúdo "([^"]*)"$"#)]
fn when_ele_tenta_alterar_a_noticia_para_o_titulo_valor_e_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("ele tenta alterar a notícia para o título {string} e o conteúdo {string}");
}

/// Pendente: ele tenta alterar essa notícia para o título {string} e o conteúdo {string}.
#[when(
    regex = r#"^ele tenta alterar essa notícia para o título "([^"]*)" e o conteúdo "([^"]*)"$"#
)]
fn when_ele_tenta_alterar_essa_noticia_para_o_titulo_valor_e_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("ele tenta alterar essa notícia para o título {string} e o conteúdo {string}");
}

/// Pendente: ele tenta alterar o título para {string} e o conteúdo para {string}.
#[when(regex = r#"^ele tenta alterar o título para "([^"]*)" e o conteúdo para "([^"]*)"$"#)]
fn when_ele_tenta_alterar_o_titulo_para_valor_e_o_conteudo_para_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("ele tenta alterar o título para {string} e o conteúdo para {string}");
}

/// Pendente: ele tenta criar e publicar essa notícia diretamente.
#[when(regex = r#"^ele tenta criar e publicar essa notícia diretamente$"#)]
fn when_ele_tenta_criar_e_publicar_essa_noticia_diretamente(_world: &mut AppWorld) {
    pending_step("ele tenta criar e publicar essa notícia diretamente");
}

/// Pendente: ele tenta criar uma notícia com o título {string} e o conteúdo {string}.
#[when(regex = r#"^ele tenta criar uma notícia com o título "([^"]*)" e o conteúdo "([^"]*)"$"#)]
fn when_ele_tenta_criar_uma_noticia_com_o_titulo_valor_e_o_conteudo_valor(
    _world: &mut AppWorld,
    _value_1: String,
    _value_2: String,
) {
    pending_step("ele tenta criar uma notícia com o título {string} e o conteúdo {string}");
}

/// Pendente: ele tenta publicar essa notícia diretamente.
#[when(regex = r#"^ele tenta publicar essa notícia diretamente$"#)]
fn when_ele_tenta_publicar_essa_noticia_diretamente(_world: &mut AppWorld) {
    pending_step("ele tenta publicar essa notícia diretamente");
}

/// Pendente: tenta homologar essa proposta.
#[when(regex = r#"^tenta homologar essa proposta$"#)]
fn when_tenta_homologar_essa_proposta(_world: &mut AppWorld) {
    pending_step("tenta homologar essa proposta");
}
