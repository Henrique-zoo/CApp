/// A publicação de materiais permite disponibilizar links externos e arquivos
/// acadêmicos por integrantes autorizados da gestão dos Centros Acadêmicos.
///
/// Registra o vocabulário usado pelo runner enquanto a automação está pendente.
/// Cada ação usa [pendingStep] e falha explicitamente quando executada. A futura
/// implementação deverá exercitar o aplicativo e seus efeitos reais antes de
/// remover `@pending_mobile`. Expressões comuns são reutilizadas de outros módulos.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final materialPublicationSteps = <StepDefinition>[
  pendingStep(r'^a publicação do material é concluída$'),
  pendingStep(r'^a publicação do material é recusada$'),
  pendingStep(r'^o material não fica disponível como publicado$'),
  pendingStep(r'^a publicação do material é recusada por falta de permissão$'),
  pendingStep(r'^a publicação do material não é concluída$'),
  pendingStep(r'^o arquivo fica associado ao material publicado$'),
  pendingStep(r'^o arquivo PDF fica associado ao material publicado$'),
  pendingStep(
    r'^o endereço do link externo fica associado ao material publicado$',
  ),
  pendingStep(
    r'^o integrante publica o material "([^"]*)" com um arquivo no limite de tamanho permitido$',
  ),
  pendingStep(
    r'^o integrante publica o material "([^"]*)" com um arquivo PDF válido$',
  ),
  pendingStep(
    r'^o integrante publica o material "([^"]*)" com o link externo "([^"]*)"$',
  ),
  pendingStep(
    r'^o integrante tenta publicar o material "([^"]*)" com um arquivo PDF válido, mas ocorre uma falha durante o envio$',
  ),
  pendingStep(
    r'^o integrante realiza uma nova tentativa com um arquivo PDF válido$',
  ),
  pendingStep(
    r'^o integrante tenta publicar o material "([^"]*)" com um arquivo acima do limite de tamanho permitido$',
  ),
  pendingStep(
    r'^o integrante tenta publicar o material "([^"]*)" com um arquivo em formato não aceito$',
  ),
  pendingStep(
    r'^o integrante tenta publicar o material "([^"]*)" com um arquivo PDF válido$',
  ),
  pendingStep(
    r'^o integrante tenta publicar o material "([^"]*)" com um link externo inválido$',
  ),
  pendingStep(
    r'^o integrante tenta publicar o material "([^"]*)" com o link externo "([^"]*)"$',
  ),
  pendingStep(
    r'^que uma tentativa anterior de envio do arquivo do material "([^"]*)" falhou$',
  ),
];
