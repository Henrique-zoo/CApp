/// Registra a publicação de links (I06) e arquivos (I11) para o runner mobile.
///
/// Os steps permanecem pendentes até que o aplicativo execute os fluxos de
/// publicação e consulta. Os steps comuns de contexto usam os registros existentes.
library;

import '../../support/step_registry.dart';

/// Expressões pendentes consumidas pelo registro global dos cenários.
final materialPublicationSteps = <StepDefinition>[
  pendingStep(
    r'^em cada um dos CAs "([^"]*)" e "([^"]*)" há um estudante com acesso à consulta e sem permissão de publicação$',
  ),
  pendingStep(
    r'^que a permissão "([^"]*)" do integrante vale somente no "([^"]*)"$',
  ),
  pendingStep(
    r'^o arquivo "([^"]*)" é um PDF válido (abaixo do limite de tamanho|exatamente no limite de tamanho|1 byte acima do limite de tamanho)$',
  ),
  pendingStep(
    r'^o arquivo "([^"]*)" tem tamanho abaixo do limite e formato não aceito para materiais$',
  ),
  pendingStep(r'^o envio do arquivo falhará nesta tentativa$'),
  pendingStep(
    r'^o envio anterior de "([^"]*)" para publicar "([^"]*)" no CA ativo falhou$',
  ),
  pendingStep(
    r'^o integrante solicita a publicação de "([^"]*)" em "([^"]*)" com o link "([^"]*)"$',
  ),
  pendingStep(
    r'^o integrante solicita a publicação de "([^"]*)" em "([^"]*)" com o arquivo "([^"]*)"$',
  ),
  pendingStep(
    r'^um estudante sem permissão de publicação encontra "([^"]*)" em "([^"]*)" no "([^"]*)"$',
  ),
  pendingStep(
    r'^o estudante consegue acessar o endereço "([^"]*)" a partir do material publicado$',
  ),
  pendingStep(
    r'^o estudante consegue acessar o PDF "([^"]*)" a partir do material publicado$',
  ),
  pendingStep(
    r'^o material "([^"]*)" não aparece entre os materiais publicados do "([^"]*)"$',
  ),
  pendingStep(
    r'^a publicação do material é recusada por (endereço externo inválido|falta de permissão|disciplina incompatível com o CA ativo|formato de arquivo não aceito|tamanho de arquivo acima do limite)$',
  ),
  pendingStep(r'^o integrante é informado de que o envio do arquivo falhou$'),
  pendingStep(
    r'^um estudante sem permissão de publicação encontra uma única publicação de "([^"]*)" em "([^"]*)" no "([^"]*)"$',
  ),
];
