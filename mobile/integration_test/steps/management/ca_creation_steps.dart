/// O cadastro de um Centro Acadêmico inicia sua utilização pela comunidade com informações básicas e responsáveis identificáveis.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns têm um único registro compartilhado.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final caCreationSteps = <StepDefinition>[
  pendingStep(
    r'^a operação deve ser recusada informando que a autenticação é obrigatória$',
  ),
  pendingStep(r'^confirma o cadastro$'),
  pendingStep(
    r'^indica a si mesmo como responsável inicial pela administração$',
  ),
  pendingStep(
    r'^indica "([^"]*)" como responsável inicial pela administração$',
  ),
  pendingStep(r'^informa o nome oficial "([^"]*)"$'),
  pendingStep(r'^informa os seguintes dados básicos para o novo CA:$'),
  pendingStep(
    r'^nenhuma associação à gestão inicial de "([^"]*)" deve ser registrada$',
  ),
  pendingStep(r'^nenhuma associação à gestão inicial deve ser registrada$'),
  pendingStep(r'^não informa uma sigla para o novo CA$'),
  pendingStep(r'^o CA com o nome oficial "([^"]*)" deve ser criado$'),
  pendingStep(r'^o CA deve permanecer sem sigla$'),
  pendingStep(r'^o CA deve possuir a descrição institucional informada$'),
  pendingStep(r'^o CA deve possuir a sigla "([^"]*)"$'),
  pendingStep(r'^o CA deve possuir o nome oficial "([^"]*)"$'),
  pendingStep(r'^o Centro Acadêmico "([^"]*)" deve ser criado no CApp$'),
  pendingStep(r'^o Centro Acadêmico "([^"]*)" não deve ser criado$'),
  pendingStep(
    r'^o cadastro deve ser concluído sem exigir importação ou migração de dados anteriores$',
  ),
  pendingStep(
    r'^o cadastro deve ser recusado informando que o nome oficial é obrigatório$',
  ),
  pendingStep(
    r'^o cadastro deve ser recusado informando que o usuário não foi encontrado$',
  ),
  pendingStep(
    r'^o estudante deve constar como responsável inicial pela administração do CA criado$',
  ),
  pendingStep(
    r'^o estudante "([^"]*)" deve constar como responsável inicial pela administração de "([^"]*)"$',
  ),
  pendingStep(r'^que encerrei minha sessão no aplicativo$'),
  pendingStep(r'^tenta cadastrar o CA "([^"]*)"$'),
  pendingStep(
    r'^tenta cadastrar um CA com a sigla "([^"]*)" sem informar o nome oficial$',
  ),
  pendingStep(r'^tento cadastrar um Centro Acadêmico com a sigla "([^"]*)"$'),
];
