import 'package:capp/src/modules/materials/application/materials.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../support/materials_scenario.dart';
import '../../support/step_registry.dart';

final materialViewingSteps = <StepDefinition>[
  materialsStep(
    r'^que o CA ativo para consulta de materiais é "([^"]*)"$',
    (scenario, match) => scenario.reset(match[1]!),
  ),
  materialsStep(
    r'^altero o CA ativo da consulta de materiais para "([^"]*)"$',
    (scenario, match) => scenario.selectCenter(match[1]!),
  ),
  materialsStep(r'^vejo os materiais "([^"]*)" e "([^"]*)"$', (
    scenario,
    match,
  ) {
    scenario.expectListed(match[1]!);
    scenario.expectListed(match[2]!);
  }),
  materialsStep(
    r'^vejo o material "([^"]*)"$',
    (scenario, match) => scenario.expectListed(match[1]!),
  ),
  materialsStep(
    r'^não vejo o material "([^"]*)"$',
    (scenario, match) => scenario.expectListed(match[1]!, present: false),
  ),
  materialsStep(
    r'^vejo o título do material "([^"]*)"$',
    (_, match) => expect(find.text(match[1]!), findsOneWidget),
  ),
  materialsStep(r'^a descrição do material é "([^"]*)"$', (
    scenario,
    match,
  ) async {
    await scenario.change('describe', {'description': match[1]!});
  }),
  materialsStep(
    r'^a listagem informa que não há materiais publicados$',
    (_, _) => expect(
      find.text('Nenhum material publicado nesta seleção.'),
      findsOneWidget,
    ),
  ),
  materialsStep(
    r'^consulto (?:novamente )?a lista de materiais$',
    (scenario, _) => scenario.openMaterials(),
  ),
  materialsStep(
    r'^consulto os detalhes do material "([^"]*)"$',
    (scenario, match) => scenario.openDetails(match[1]!),
  ),
  materialsStep(
    r'^existe o material publicado "([^"]*)" no CA "([^"]*)"$',
    (scenario, match) => scenario.seed(match[1]!, center: match[2]!),
  ),
  materialsStep(
    r'^existe o material "([^"]*)" arquivado nesse CA$',
    (scenario, match) => scenario.seed(match[1]!, pdf: true, archived: true),
  ),
  materialsStep(
    r'^existem os materiais publicados "([^"]*)" e "([^"]*)" nesse CA$',
    (scenario, match) async {
      await scenario.seed(match[1]!);
      await scenario.seed(match[2]!);
    },
  ),
  materialsStep(
    r'^não existem materiais publicados nesse CA$',
    (scenario, _) async => expect(await scenario.published(), isEmpty),
  ),
  materialsStep(
    r'^o aplicativo informa que não foi possível carregar os materiais$',
    (_, _) => expect(
      find.text('Ocorreu uma falha interna ao processar o material.'),
      findsOneWidget,
    ),
  ),
  materialsStep(
    r'^que não foi possível obter os materiais publicados do CA ativo$',
    (scenario, _) async {
      await scenario.reset(computingCenter);
      await scenario.change('fail_list');
    },
  ),
  materialsStep(
    r'^vejo a descrição "([^"]*)"$',
    (_, match) => expect(find.text(match[1]!), findsOneWidget),
  ),
  materialsStep(
    r'^o material possui conteúdo disponível no formato "([^"]*)"$',
    (scenario, match) async {
      switch (match[1]) {
        case 'PDF':
          await scenario.change('convert_last_to_pdf');
        case 'link externo':
          break;
        default:
          throw StateError('Formato de material indisponível: ${match[1]}');
      }
    },
  ),
  materialsStep(r'^acesso o conteúdo do material "([^"]*)"$', (
    scenario,
    match,
  ) async {
    await scenario.openDetails(match[1]!);
    final save = find.text('Salvar PDF');
    await scenario.tap(
      save.evaluate().isEmpty ? find.text('Abrir link') : save,
    );
  }),
  materialsStep(
    r'^o conteúdo publicado é disponibilizado no formato "([^"]*)"$',
    (scenario, match) async {
      switch (match[1]) {
        case 'PDF':
          expect(find.text('PDF salvo.'), findsOneWidget);
          expect(scenario.device.saved?.name, 'referencias.pdf');
          expect(
            scenario.device.saved?.bytes,
            orderedEquals(await scenario.pdf()),
          );
        case 'link externo':
          expect(
            scenario.device.opened,
            Uri.parse('https://exemplo.edu.br/referencias'),
          );
        default:
          throw StateError('Formato de material indisponível: ${match[1]}');
      }
    },
  ),
  materialsStep(r'^tento acessar o conteúdo do material "([^"]*)"$', (
    scenario,
    match,
  ) async {
    await scenario.openMaterials();
    scenario.expectListed(match[1]!, present: false);
    final ca = await scenario.center(scenario.activeCenter);
    final id = scenario.archivedIds[match[1]!]!;
    final notFound = throwsA(
      isA<MaterialFailure>().having(
        (failure) => failure.message,
        'mensagem',
        'Material publicado não encontrado neste Centro Acadêmico.',
      ),
    );
    await expectLater(scenario.gateway.details(ca.id, id), notFound);
    expect(await scenario.downloadStatus(ca.id, id), 404);
  }),
  materialsStep(r'^o conteúdo do material não é disponibilizado$', (
    scenario,
    _,
  ) {
    expect(find.text('Abrir link'), findsNothing);
    expect(find.text('Salvar PDF'), findsNothing);
    expect(scenario.device.opened, isNull);
    expect(scenario.device.saved, isNull);
  }),
];
