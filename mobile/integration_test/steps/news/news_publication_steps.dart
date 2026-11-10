/// A publicação de notícias depende das permissões locais e da homologação das propostas e alterações.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final newsPublicationSteps = <StepDefinition>[
  pendingStep(
    r'^a alteração só pode voltar a ser publicada após homologação por um integrante autorizado$',
  ),
  pendingStep(r'^a alteração é recusada$'),
  pendingStep(r'^a data de criação da notícia fica registrada$'),
  pendingStep(r'^a data de publicação da notícia fica registrada$'),
  pendingStep(r'^a edição é recusada$'),
  pendingStep(
    r'^a homologação é recusada por falta da permissão correspondente$',
  ),
  pendingStep(
    r'^a listagem de notícias publicadas desse Centro Acadêmico é formada$',
  ),
  pendingStep(r'^a notícia continua fora da listagem publicada$'),
  pendingStep(
    r'^a notícia entra na listagem publicada do Centro Acadêmico "([^"]*)"$',
  ),
  pendingStep(r'^a notícia fica pendente de homologação$'),
  pendingStep(
    r'^a notícia fica pendente de homologação por um integrante autorizado a homologar notícias$',
  ),
  pendingStep(r'^a notícia fica pendente de nova homologação$'),
  pendingStep(
    r'^a notícia fica registrada com o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(
    r'^a notícia fica registrada no Centro Acadêmico "([^"]*)" como solicitação$',
  ),
  pendingStep(r'^a notícia não aparece na listagem publicada$'),
  pendingStep(r'^a notícia não aparece para os usuários$'),
  pendingStep(
    r'^a notícia não aparece para os usuários do Centro Acadêmico "([^"]*)"$',
  ),
  pendingStep(r'^a notícia não entra na listagem publicada$'),
  pendingStep(r'^a notícia não fica pendente de homologação$'),
  pendingStep(r'^a notícia não possui data de publicação$'),
  pendingStep(
    r'^a notícia permanece com o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(r'^a notícia permanece pendente de homologação$'),
  pendingStep(
    r'^a notícia permanece pendente de homologação por um integrante autorizado a homologar notícias$',
  ),
  pendingStep(r'^a notícia permanece rejeitada$'),
  pendingStep(
    r'^a notícia sai da listagem publicada enquanto aguarda a aprovação$',
  ),
  pendingStep(
    r'^a notícia volta à listagem publicada com o conteúdo homologado$',
  ),
  pendingStep(r'^a notícia é publicada sem homologação prévia$'),
  pendingStep(r'^a proposta permanece pendente e fora da listagem publicada$'),
  pendingStep(r'^a publicação direta é recusada$'),
  pendingStep(r'^a publicação é recusada$'),
  pendingStep(r'^aprova a alteração$'),
  pendingStep(
    r'^cria e publica diretamente uma notícia própria com título e conteúdo válidos$',
  ),
  pendingStep(r'^cria uma notícia com título e conteúdo válidos$'),
  pendingStep(
    r'^ele altera o título para "([^"]*)" e o conteúdo para "([^"]*)"$',
  ),
  pendingStep(
    r'^ele cria uma notícia com o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(
    r'^ele está preparando uma notícia com o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(r'^ele publica essa notícia diretamente$'),
  pendingStep(
    r'^ele tenta alterar a notícia para o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(
    r'^ele tenta alterar essa notícia para o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(
    r'^ele tenta alterar o título para "([^"]*)" e o conteúdo para "([^"]*)"$',
  ),
  pendingStep(r'^ele tenta criar e publicar essa notícia diretamente$'),
  pendingStep(
    r'^ele tenta criar uma notícia com o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(r'^ele tenta publicar essa notícia diretamente$'),
  pendingStep(r'^essa notícia não integra a listagem publicada$'),
  pendingStep(
    r'^existe uma proposta de notícia de outro usuário aguardando homologação$',
  ),
  pendingStep(r'^nenhuma notícia é registrada$'),
  pendingStep(
    r'^nesse Centro Acadêmico existe uma notícia já publicada com o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(
    r'^nesse Centro Acadêmico existe uma notícia não publicada com o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(
    r'^nesse Centro Acadêmico existe uma notícia não publicada de sua autoria com o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(
    r'^não possui permissão de publicação direta de notícias nesse CA$',
  ),
  pendingStep(
    r'^o Centro Acadêmico "([^"]*)" possui uma notícia não publicada com o título "([^"]*)" e o conteúdo "([^"]*)"$',
  ),
  pendingStep(r'^o registro da notícia é aceito$'),
  pendingStep(r'^o registro é recusado$'),
  pendingStep(r'^o usuário fica vinculado como autor da notícia$'),
  pendingStep(
    r'^o usuário não possui publicação direta nem homologação de notícias em "([^"]*)"$',
  ),
  pendingStep(
    r'^que o Centro Acadêmico "([^"]*)" possui uma notícia pendente de homologação$',
  ),
  pendingStep(
    r'^que o Centro Acadêmico "([^"]*)" possui uma notícia rejeitada$',
  ),
  pendingStep(
    r'^que o gestor do Centro Acadêmico "([^"]*)" não possui permissão de publicação direta$',
  ),
  pendingStep(
    r'^que o integrante da gestão não possui publicação direta nem homologação de notícias no CA selecionado$',
  ),
  pendingStep(
    r'^que o integrante possui a permissão "([^"]*)" no CA selecionado$',
  ),
  pendingStep(
    r'^que o integrante possui somente a permissão "([^"]*)" no CA selecionado$',
  ),
  pendingStep(r'^que o usuário está autenticado no CApp$'),
  pendingStep(
    r'^que o usuário não possui permissão de publicação direta nem de homologação de notícias no Centro Acadêmico "([^"]*)"$',
  ),
  pendingStep(
    r'^que o usuário possui permissão de publicação direta de notícias no Centro Acadêmico "([^"]*)"$',
  ),
  pendingStep(
    r'^que o usuário possui permissão de publicação direta no Centro Acadêmico "([^"]*)"$',
  ),
  pendingStep(r'^que uma notícia foi retirada da listagem após uma edição$'),
  pendingStep(r'^selecionou o contexto "([^"]*)"$'),
  pendingStep(r'^selecionou um Centro Acadêmico$'),
  pendingStep(r'^sua alteração aguarda nova homologação no CA selecionado$'),
  pendingStep(r'^tenta homologar essa proposta$'),
];
