/// A organização por disciplina permite localizar materiais compatíveis com o contexto acadêmico.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final materialOrganizationSteps = <StepDefinition>[
  pendingStep(r'^a associação do material com a disciplina é registrada$'),
  pendingStep(r'^a associação é recusada$'),
  pendingStep(r'^a associação é recusada por falta de permissão$'),
  pendingStep(r'^a disciplina "([^"]*)" não é apresentada$'),
  pendingStep(
    r'^a disciplina "([^"]*)" pertence à grade curricular do contexto acadêmico$',
  ),
  pendingStep(r'^a disciplina "([^"]*)" é apresentada$'),
  pendingStep(
    r'^a grade curricular desse contexto possui a disciplina "([^"]*)"$',
  ),
  pendingStep(
    r'^a grade curricular do contexto acadêmico possui as disciplinas "([^"]*)", "([^"]*)" e "([^"]*)"$',
  ),
  pendingStep(r'^a organização anterior do material permanece inalterada$'),
  pendingStep(r'^a organização corresponde ao contexto do "([^"]*)"$'),
  pendingStep(r'^acessa novamente a organização de materiais por disciplina$'),
  pendingStep(r'^ambos estão associados à disciplina "([^"]*)"$'),
  pendingStep(r'^ambos ficam disponíveis na organização da disciplina$'),
  pendingStep(
    r'^as disciplinas da grade curricular são apresentadas para consulta$',
  ),
  pendingStep(r'^consulta os materiais da disciplina "([^"]*)"$'),
  pendingStep(r'^esse integrante associa "([^"]*)" à disciplina "([^"]*)"$'),
  pendingStep(
    r'^esse integrante tenta associar "([^"]*)" à disciplina "([^"]*)"$',
  ),
  pendingStep(r'^existe a disciplina "([^"]*)" em outro contexto acadêmico$'),
  pendingStep(
    r'^existe o material publicado "([^"]*)" associado à disciplina "([^"]*)"$',
  ),
  pendingStep(
    r'^existe o material publicado "([^"]*)" associado à disciplina "([^"]*)" nesse CA$',
  ),
  pendingStep(
    r'^existe o material publicado "([^"]*)" associado à disciplina "([^"]*)" no "([^"]*)"$',
  ),
  pendingStep(r'^existe o material publicado "([^"]*)" no formato PDF$'),
  pendingStep(r'^existe o material publicado "([^"]*)" no formato de link$'),
  pendingStep(r'^existe o material "([^"]*)" ainda não publicado$'),
  pendingStep(r'^existe um material publicado nesse CA$'),
  pendingStep(
    r'^existem materiais publicados associados às disciplinas "([^"]*)" e "([^"]*)"$',
  ),
  pendingStep(
    r'^existem os materiais publicados "([^"]*)" e "([^"]*)" associados à disciplina "([^"]*)"$',
  ),
  pendingStep(r'^nenhum material é apresentado$'),
  pendingStep(r'^nenhuma disciplina é apresentada para consulta$'),
  pendingStep(
    r'^não existem disciplinas cadastradas na grade curricular do contexto acadêmico$',
  ),
  pendingStep(
    r'^não existem materiais publicados associados à disciplina "([^"]*)"$',
  ),
  pendingStep(
    r'^o aplicativo informa que não existem disciplinas disponíveis para a organização dos materiais$',
  ),
  pendingStep(
    r'^o aplicativo informa que não existem materiais disponíveis para essa disciplina$',
  ),
  pendingStep(
    r'^o estudante acessa a organização de materiais por disciplina$',
  ),
  pendingStep(r'^o estudante altera o CA ativo para "([^"]*)"$'),
  pendingStep(r'^o estudante consulta os materiais da disciplina "([^"]*)"$'),
  pendingStep(r'^o estudante está consultando os materiais de "([^"]*)"$'),
  pendingStep(
    r'^o estudante pode selecionar uma disciplina para visualizar seus materiais$',
  ),
  pendingStep(r'^o estudante retorna à lista de disciplinas$'),
  pendingStep(r'^o estudante sai da organização de materiais$'),
  pendingStep(r'^o estudante seleciona a disciplina "([^"]*)"$'),
  pendingStep(r'^o integrante não possui a permissão "([^"]*)" nesse CA$'),
  pendingStep(r'^o material fica disponível para consulta nessa disciplina$'),
  pendingStep(r'^o material não fica associado à disciplina incompatível$'),
  pendingStep(
    r'^o material passa a pertencer à organização da disciplina "([^"]*)"$',
  ),
  pendingStep(
    r'^o material publicado "([^"]*)" está associado à disciplina "([^"]*)"$',
  ),
  pendingStep(r'^o material selecionado fica disponível para acesso$'),
  pendingStep(r'^o material "([^"]*)" continua associado à disciplina$'),
  pendingStep(r'^o material "([^"]*)" não é apresentado$'),
  pendingStep(r'^o material "([^"]*)" não é apresentado na consulta atual$'),
  pendingStep(r'^o material "([^"]*)" não é apresentado nessa consulta$'),
  pendingStep(r'^o material "([^"]*)" é apresentado$'),
  pendingStep(
    r'^os materiais de "([^"]*)" não são apresentados nessa consulta$',
  ),
  pendingStep(r'^permanece disponível para consulta$'),
  pendingStep(r'^que o estudante está no "([^"]*)"$'),
  pendingStep(r'^que o integrante possui a permissão "([^"]*)" no CA ativo$'),
  pendingStep(r'^seleciona a disciplina "([^"]*)"$'),
  pendingStep(r'^seleciona novamente a disciplina "([^"]*)"$'),
  pendingStep(r'^seleciona o material "([^"]*)"$'),
  pendingStep(r'^seleciona "([^"]*)"$'),
  pendingStep(r'^sua associação com a disciplina "([^"]*)" é mantida$'),
  pendingStep(
    r'^são apresentados os materiais associados à disciplina "([^"]*)"$',
  ),
  pendingStep(r'^tenta associar o material a uma disciplina$'),
];
