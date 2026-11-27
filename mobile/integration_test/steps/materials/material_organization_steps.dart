import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../support/materials_scenario.dart';
import '../../support/step_registry.dart';

Future<void> _checkCatalog(
  MaterialsScenario scenario, {
  List<String> present = const [],
  List<String> absent = const [],
}) async {
  await scenario.tap(scenario.disciplineMenu);
  for (final name in present) {
    expect(find.text(name).hitTestable(), findsWidgets);
  }
  for (final name in absent) {
    expect(find.text(name), findsNothing);
  }
  await scenario.tester.sendKeyEvent(LogicalKeyboardKey.escape);
  await scenario.tester.pumpAndSettle();
}

final materialOrganizationSteps = <StepDefinition>[
  materialsStep(
    r'^a disciplina "([^"]*)" pertence à grade curricular do contexto acadêmico$',
    (scenario, match) => scenario.addDiscipline(match[1]!),
  ),
  materialsStep(
    r'^a grade curricular desse contexto possui a disciplina "([^"]*)"$',
    (scenario, match) => scenario.addDiscipline(match[1]!),
  ),
  materialsStep(
    r'^a grade curricular do contexto acadêmico possui as disciplinas "([^"]*)", "([^"]*)" e "([^"]*)"$',
    (scenario, match) async {
      for (final index in [1, 2, 3]) {
        await scenario.addDiscipline(match[index]!);
      }
    },
  ),
  materialsStep(r'^ambos estão associados à disciplina "([^"]*)"$', (
    scenario,
    match,
  ) async {
    expect(await scenario.published(disciplineName: match[1]!), hasLength(2));
  }),
  materialsStep(
    r'^existe a disciplina "([^"]*)" em outro contexto acadêmico$',
    (scenario, match) => scenario.addDiscipline(match[1]!, center: mathCenter),
  ),
  materialsStep(
    r'^existe o material publicado "([^"]*)" associado à disciplina "([^"]*)"(?: nesse CA)?$',
    (scenario, match) => scenario.seed(match[1]!, discipline: match[2]!),
  ),
  materialsStep(
    r'^existe o material publicado "([^"]*)" associado à disciplina "([^"]*)" no "([^"]*)"$',
    (scenario, match) =>
        scenario.seed(match[1]!, discipline: match[2]!, center: match[3]!),
  ),
  materialsStep(
    r'^existe o material publicado "([^"]*)" no formato (PDF|de link)$',
    (scenario, match) => scenario.seed(match[1]!, pdf: match[2] == 'PDF'),
  ),
  materialsStep(
    r'^existe o material arquivado "([^"]*)" associado à disciplina "([^"]*)"$',
    (scenario, match) =>
        scenario.seed(match[1]!, discipline: match[2]!, archived: true),
  ),
  materialsStep(
    r'^existem materiais publicados associados às disciplinas "([^"]*)" e "([^"]*)"$',
    (scenario, match) async {
      for (final name in [match[1]!, match[2]!]) {
        await scenario.seed('Resumo de $name', discipline: name);
      }
    },
  ),
  materialsStep(
    r'^existem os materiais publicados "([^"]*)" e "([^"]*)" associados à disciplina "([^"]*)"$',
    (scenario, match) async {
      await scenario.seed(match[1]!, discipline: match[3]!);
      await scenario.seed(match[2]!, discipline: match[3]!);
    },
  ),
  materialsStep(
    r'^não existem disciplinas cadastradas na grade curricular do contexto acadêmico$',
    (scenario, _) async {
      await scenario.change('empty_catalog');
    },
  ),
  materialsStep(
    r'^não existem materiais publicados associados à disciplina "([^"]*)"$',
    (scenario, match) async =>
        expect(await scenario.published(disciplineName: match[1]!), isEmpty),
  ),
  materialsStep(
    r'^o estudante está consultando os materiais de "([^"]*)"$',
    (scenario, match) => scenario.selectDiscipline(match[1]!),
  ),
  materialsStep(
    r'^o material publicado "([^"]*)" está associado à disciplina "([^"]*)"$',
    (scenario, match) => scenario.seed(match[1]!, discipline: match[2]!),
  ),
  materialsStep(
    r'^que o estudante está no "([^"]*)"$',
    (scenario, match) => scenario.reset(match[1]!),
  ),
  materialsStep(
    r'^a disciplina "([^"]*)" não é apresentada$',
    (scenario, match) => _checkCatalog(scenario, absent: [match[1]!]),
  ),
  materialsStep(
    r'^a disciplina "([^"]*)" é apresentada$',
    (scenario, match) => _checkCatalog(scenario, present: [match[1]!]),
  ),
  materialsStep(r'^a organização corresponde ao contexto do "([^"]*)"$', (
    scenario,
    match,
  ) async {
    final center = await scenario.center(match[1]!);
    expect(
      scenario.tester.widget<DropdownButton<String>>(scenario.centerMenu).value,
      center.id,
    );
    expect(scenario.selectedDiscipline, 'Cálculo I');
  }),
  materialsStep(r'^ambos ficam disponíveis na organização da disciplina$', (
    _,
    _,
  ) {
    expect(find.byIcon(Icons.link), findsOneWidget);
    expect(find.byIcon(Icons.picture_as_pdf_outlined), findsOneWidget);
  }),
  materialsStep(
    r'^as disciplinas da grade curricular são apresentadas para consulta$',
    (scenario, _) => _checkCatalog(
      scenario,
      present: ['Cálculo I', 'Álgebra Linear', 'Física I'],
    ),
  ),
  materialsStep(
    r'^nenhum material é apresentado$',
    (_, _) => expect(find.byType(ListTile), findsOneWidget),
  ),
  materialsStep(
    r'^nenhuma disciplina é apresentada para consulta$',
    (scenario, _) => expect(scenario.disciplineMenu, findsNothing),
  ),
  materialsStep(
    r'^o aplicativo informa que não existem disciplinas disponíveis para a organização dos materiais$',
    (_, _) => expect(
      find.text('Nenhuma disciplina disponível para organizar os materiais.'),
      findsOneWidget,
    ),
  ),
  materialsStep(
    r'^o aplicativo informa que não existem materiais disponíveis para essa disciplina$',
    (_, _) => expect(
      find.text('Nenhum material publicado nesta seleção.'),
      findsOneWidget,
    ),
  ),
  materialsStep(
    r'^o estudante pode selecionar uma disciplina para visualizar seus materiais$',
    (scenario, _) async {
      await scenario.selectDiscipline('Cálculo I');
      expect(find.text('Cálculo I').hitTestable(), findsOneWidget);
      expect(
        find.text('Nenhum material publicado nesta seleção.'),
        findsOneWidget,
      );
    },
  ),
  materialsStep(r'^o material selecionado fica disponível para acesso$', (
    scenario,
    _,
  ) {
    expect(find.text(scenario.lastTitle!), findsOneWidget);
    expect(find.text('Abrir link'), findsOneWidget);
  }),
  materialsStep(
    r'^o material "([^"]*)" (?:continua associado à disciplina|é apresentado)$',
    (scenario, match) {
      scenario.expectListed(match[1]!);
      final item = find.widgetWithText(ListTile, match[1]!);
      expect(
        find.descendant(
          of: item,
          matching: find.text(scenario.selectedDiscipline!),
        ),
        findsOneWidget,
      );
      scenario.lastTitle = match[1]!;
    },
  ),
  materialsStep(
    r'^o material "([^"]*)" não é apresentado(?: na consulta atual| nessa consulta)?$',
    (scenario, match) => scenario.expectListed(match[1]!, present: false),
  ),
  materialsStep(
    r'^os materiais de "([^"]*)" não são apresentados nessa consulta$',
    (scenario, match) async {
      final other = await scenario.published(disciplineName: match[1]!);
      expect(other, isNotEmpty);
      for (final item in other) {
        scenario.expectListed(item.title, present: false);
      }
    },
  ),
  materialsStep(r'^permanece disponível para consulta$', (scenario, _) async {
    await scenario.openDetails(scenario.lastTitle!);
    expect(find.text('Abrir link'), findsOneWidget);
  }),
  materialsStep(
    r'^sua associação com a disciplina "([^"]*)" é mantida$',
    (_, match) => expect(find.text(match[1]!), findsOneWidget),
  ),
  materialsStep(
    r'^são apresentados os materiais associados à disciplina "([^"]*)"$',
    (scenario, match) async {
      final items = await scenario.published(disciplineName: match[1]!);
      expect(items, isNotEmpty);
      for (final item in items) {
        scenario.expectListed(item.title);
      }
    },
  ),
  materialsStep(
    r'^(?:o estudante acessa|acessa novamente) a organização de materiais por disciplina$',
    (scenario, _) => scenario.openMaterials(),
  ),
  materialsStep(
    r'^(?:o estudante )?consulta os materiais da disciplina "([^"]*)"$',
    (scenario, match) => scenario.selectDiscipline(match[1]!),
  ),
  materialsStep(r'^o estudante altera o CA ativo para "([^"]*)"$', (
    scenario,
    match,
  ) async {
    await scenario.openMaterials();
    await scenario.selectCenter(match[1]!);
  }),
  materialsStep(r'^o estudante retorna à lista de disciplinas$', (
    scenario,
    _,
  ) async {
    await scenario.selectOption(
      scenario.disciplineMenu,
      'Todas as disciplinas',
    );
    scenario.selectedDiscipline = null;
  }),
  materialsStep(
    r'^o estudante sai da organização de materiais$',
    (scenario, _) => scenario.leaveMaterials(),
  ),
  materialsStep(
    r'^(?:o estudante )?seleciona(?: novamente)? a disciplina "([^"]*)"$',
    (scenario, match) => scenario.selectDiscipline(match[1]!),
  ),
  materialsStep(
    r'^seleciona o material "([^"]*)"$',
    (scenario, match) => scenario.openDetails(match[1]!),
  ),
  materialsStep(
    r'^seleciona "([^"]*)"$',
    (scenario, match) => scenario.selectDiscipline(match[1]!),
  ),
  pendingStep(r'^a associação do material com a disciplina é registrada$'),
  pendingStep(r'^a associação é recusada$'),
  pendingStep(r'^a associação é recusada por falta de permissão$'),
  pendingStep(r'^a organização anterior do material permanece inalterada$'),
  pendingStep(r'^esse integrante associa "([^"]*)" à disciplina "([^"]*)"$'),
  pendingStep(
    r'^esse integrante tenta associar "([^"]*)" à disciplina "([^"]*)"$',
  ),
  pendingStep(r'^existe um material publicado nesse CA$'),
  pendingStep(r'^o integrante não possui a permissão "([^"]*)" nesse CA$'),
  pendingStep(r'^o material fica disponível para consulta nessa disciplina$'),
  pendingStep(r'^o material não fica associado à disciplina incompatível$'),
  pendingStep(
    r'^o material passa a pertencer à organização da disciplina "([^"]*)"$',
  ),
  pendingStep(r'^que o integrante possui a permissão "([^"]*)" no CA ativo$'),
  pendingStep(r'^tenta associar o material a uma disciplina$'),
];
