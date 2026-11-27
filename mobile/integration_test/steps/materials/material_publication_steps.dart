import 'dart:typed_data';

import 'package:capp/src/modules/materials/materials.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../support/materials_scenario.dart';
import '../../support/step_registry.dart';

const _uploadFailure =
    'Armazenamento de materiais temporariamente indisponível.';
const _denied = 'Acesso a materiais não permitido neste Centro Acadêmico.';
const _invalidDiscipline = 'Disciplina incompatível com o Centro Acadêmico.';

Finder _formField(String label) => find.widgetWithText(TextFormField, label);
Finder _publish() => find.widgetWithText(FilledButton, 'Publicar');
Finder _openForm() => find.widgetWithText(FilledButton, 'Publicar material');

Future<void> _submit(
  MaterialsScenario scenario,
  String title,
  String discipline, {
  String? url,
  String? filename,
}) async {
  await scenario.openMaterials();
  final ca = await scenario.center(scenario.activeCenter);
  final choices = await scenario.gateway.disciplines(ca.id);
  final isRetry = _publish().evaluate().isNotEmpty;
  if (!ca.canPublish) {
    expect(_openForm(), findsNothing);
    final disciplineId = choices
        .singleWhere((choice) => choice.name == discipline)
        .id;
    final draft = (title: title, description: '', disciplineId: disciplineId);
    final requestId = const Uuid().v4();
    final attempt = url == null
        ? scenario.gateway.publishPdf(
            ca.id,
            draft,
            scenario.device.input!,
            requestId,
          )
        : scenario.gateway.publishLink(ca.id, draft, url, requestId);
    await expectLater(attempt, _failureMessage(_denied));
    return;
  }

  if (!isRetry) {
    await scenario.tap(_openForm());
    await scenario.tester.enterText(_formField('Título'), title);
    if (!choices.any((choice) => choice.name == discipline)) {
      await scenario.tap(scenario.disciplineMenu);
      expect(find.text(discipline), findsNothing);
      final foreign = await scenario.discipline(
        discipline,
        centerName: mathCenter,
      );
      await expectLater(
        scenario.gateway.publishLink(
          ca.id,
          (title: title, description: '', disciplineId: foreign.id),
          url!,
          const Uuid().v4(),
        ),
        _failureMessage(_invalidDiscipline),
      );
      await scenario.tester.tapAt(const Offset(5, 5));
      await scenario.tester.pumpAndSettle();
      return;
    }
    await scenario.selectOption(scenario.disciplineMenu, discipline);
    if (url != null) {
      await scenario.tester.enterText(_formField('Endereço do link'), url);
    } else {
      expect(scenario.device.input?.name, filename);
      await scenario.selectOption(
        find.byWidgetPredicate(
          (widget) =>
              widget is InputDecorator &&
              widget.decoration.labelText == 'Tipo de conteúdo',
        ),
        'PDF',
      );
      await scenario.tap(find.text('Selecionar PDF (até 10 MiB)'));
      expect(find.text(filename!), findsOneWidget);
    }
  }
  await scenario.tap(_publish());
}

Matcher _failureMessage(String message) => throwsA(
  isA<MaterialFailure>().having((error) => error.message, 'mensagem', message),
);

final materialPublicationSteps = <StepDefinition>[
  materialsStep(
    r'^que o CA ativo para publicação de materiais é "([^"]*)"$',
    (scenario, match) => scenario.reset(match[1]!, publication: true),
  ),
  materialsStep(
    r'^a disciplina "([^"]*)" está disponível para publicar materiais no CA ativo$',
    (scenario, match) async =>
        expect((await scenario.discipline(match[1]!)).id, isNotEmpty),
  ),
  materialsStep(
    r'^a disciplina "([^"]*)" está disponível para publicar materiais somente em outro CA$',
    (scenario, match) async {
      final computing = await scenario.center(scenario.activeCenter);
      final catalog = await scenario.gateway.disciplines(computing.id);
      expect(catalog.map((item) => item.name), isNot(contains(match[1]!)));
      expect(
        (await scenario.discipline(match[1]!, centerName: mathCenter)).id,
        isNotEmpty,
      );
    },
  ),
  materialsStep(
    r'^em cada um dos CAs "([^"]*)" e "([^"]*)" há um estudante com acesso à consulta e sem permissão de publicação$',
    (scenario, match) async {
      await scenario.withReader(() async {
        for (final name in [match[1]!, match[2]!]) {
          expect((await scenario.center(name)).canPublish, isFalse);
          expect(await scenario.published(centerName: name), isEmpty);
        }
      });
    },
  ),
  materialsStep(r'^que o integrante pode gerenciar materiais no CA ativo$', (
    scenario,
    _,
  ) async {
    await scenario.change('set_publisher', {'center': scenario.activeCenter});
    expect((await scenario.center(scenario.activeCenter)).canPublish, isTrue);
  }),
  materialsStep(
    r'^que o integrante não pode gerenciar materiais no CA ativo$',
    (scenario, _) async => expect(
      (await scenario.center(scenario.activeCenter)).canPublish,
      isFalse,
    ),
  ),
  materialsStep(
    r'^que a permissão "([^"]*)" do integrante vale somente no "([^"]*)"$',
    (scenario, match) async {
      expect(match[1], 'Gerenciar materiais');
      await scenario.change('set_publisher', {'center': match[2]!});
      expect((await scenario.center(match[2]!)).canPublish, isTrue);
      expect(
        (await scenario.center(scenario.activeCenter)).canPublish,
        isFalse,
      );
    },
  ),
  materialsStep(
    r'^o arquivo "([^"]*)" é um PDF válido (abaixo do limite de tamanho|exatamente no limite de tamanho|1 byte acima do limite de tamanho)$',
    (scenario, match) async {
      final size = switch (match[2]) {
        'exatamente no limite de tamanho' => maxMaterialPdfBytes,
        '1 byte acima do limite de tamanho' => maxMaterialPdfBytes + 1,
        _ => null,
      };
      scenario.device.input = (
        name: match[1]!,
        bytes: await scenario.pdf(size),
      );
    },
  ),
  materialsStep(
    r'^o arquivo "([^"]*)" tem tamanho abaixo do limite e formato não aceito para materiais$',
    (scenario, match) {
      scenario.device.input = (
        name: match[1]!,
        bytes: Uint8List.fromList([1, 2, 3]),
      );
    },
  ),
  materialsStep(r'^o envio do arquivo falhará nesta tentativa$', (
    scenario,
    _,
  ) async {
    await scenario.change('fail_upload');
  }),
  materialsStep(
    r'^o envio anterior de "([^"]*)" para publicar "([^"]*)" no CA ativo falhou$',
    (scenario, match) async {
      await scenario.change('fail_upload');
      await _submit(scenario, match[2]!, 'Cálculo I', filename: match[1]!);
      expect(find.text(_uploadFailure), findsOneWidget);
      final items = await scenario.published();
      expect(items.map((item) => item.title), isNot(contains(match[2]!)));
      expect(scenario.publicationKeys, hasLength(1));
    },
  ),
  materialsStep(
    r'^o integrante solicita a publicação de "([^"]*)" em "([^"]*)" com o link "([^"]*)"$',
    (scenario, match) =>
        _submit(scenario, match[1]!, match[2]!, url: match[3]!),
  ),
  materialsStep(
    r'^o integrante solicita a publicação de "([^"]*)" em "([^"]*)" com o arquivo "([^"]*)"$',
    (scenario, match) =>
        _submit(scenario, match[1]!, match[2]!, filename: match[3]!),
  ),
  materialsStep(
    r'^um estudante sem permissão de publicação encontra(?: uma única publicação de)? "([^"]*)" em "([^"]*)" no "([^"]*)"$',
    (scenario, match) async {
      final title = match[1]!;
      final discipline = match[2]!;
      final ca = match[3]!;
      if (match[0]!.contains('uma única publicação')) {
        expect(scenario.publicationKeys, hasLength(2));
        expect(scenario.publicationKeys.toSet(), hasLength(1));
      }
      await scenario.assumeReader();
      await scenario.openMaterials();
      await scenario.selectCenter(ca);
      await scenario.selectDiscipline(discipline);
      expect((await scenario.center(ca)).canPublish, isFalse);
      scenario.expectListed(title);
      final items = await scenario.published(
        centerName: ca,
        disciplineName: discipline,
      );
      expect(items.where((item) => item.title == title), hasLength(1));
      scenario.lastTitle = title;
    },
  ),
  materialsStep(
    r'^o estudante consegue acessar o endereço "([^"]*)" a partir do material publicado$',
    (scenario, match) async {
      await scenario.openContent(scenario.lastTitle!, pdf: false);
      expect(scenario.device.opened, Uri.parse(match[1]!));
    },
  ),
  materialsStep(
    r'^o estudante consegue acessar o PDF "([^"]*)" a partir do material publicado$',
    (scenario, match) async {
      await scenario.openContent(scenario.lastTitle!, pdf: true);
      expect(find.text('PDF salvo.'), findsOneWidget);
      expect(scenario.device.saved?.name, match[1]!);
      expect(
        scenario.device.saved?.bytes,
        orderedEquals(scenario.device.input!.bytes),
      );
    },
  ),
  materialsStep(
    r'^o material "([^"]*)" não aparece entre os materiais publicados do "([^"]*)"$',
    (scenario, match) async {
      await scenario.assumeReader();
      await scenario.backToList();
      await scenario.selectCenter(match[2]!);
      scenario.expectListed(match[1]!, present: false);
      final items = await scenario.published();
      expect(items.map((item) => item.title), isNot(contains(match[1]!)));
    },
  ),
  materialsStep(
    r'^a publicação do material é recusada por (endereço externo inválido|falta de permissão|disciplina incompatível com o CA ativo|formato de arquivo não aceito|tamanho de arquivo acima do limite)$',
    (_, match) {
      switch (match[1]) {
        case 'falta de permissão':
          expect(_openForm(), findsNothing);
        case 'disciplina incompatível com o CA ativo':
          expect(find.text('Direito Constitucional'), findsNothing);
          expect(_publish(), findsOneWidget);
        case 'endereço externo inválido':
          expect(
            find.text('Informe um endereço HTTP ou HTTPS válido.'),
            findsOneWidget,
          );
        case 'formato de arquivo não aceito':
          expect(
            find.text('O arquivo deve ser um PDF válido.'),
            findsOneWidget,
          );
        case 'tamanho de arquivo acima do limite':
          expect(
            find.text('O PDF ultrapassa o limite de 10 MiB.'),
            findsOneWidget,
          );
      }
    },
  ),
  materialsStep(
    r'^o integrante é informado de que o envio do arquivo falhou$',
    (_, _) => expect(find.text(_uploadFailure), findsOneWidget),
  ),
];
