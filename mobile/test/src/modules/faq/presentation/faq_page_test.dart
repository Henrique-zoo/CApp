/// Testes de widget da tela [FaqPage] e sua integração na navegação principal.
library;

import 'package:capp/src/app/app.dart';
import 'package:capp/src/app/theme.dart';
import 'package:capp/src/modules/faq/faq.dart';
import 'package:capp/src/modules/faq/faq_widgets.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../support/identity_fakes.dart';
import '../application/faq_controller_test.dart';

void main() {
  const caComp = UuidValue.raw('00000000-0000-0000-0000-000000000001');
  const caMath = UuidValue.raw('00000000-0000-0000-0000-000000000002');

  late FakeFaqGateway gateway;

  setUp(() {
    gateway = FakeFaqGateway();
    gateway.entriesByCa[caComp] = [
      FaqEntry(
        id: const UuidValue.raw('00000000-0000-0000-0000-000000000011'),
        academicCenterId: caComp,
        question: 'Como solicitar apoio acadêmico?',
        answer: 'Entre em contato com a gestão do CA.',
        category: 'Apoio Acadêmico',
        createdBy: const UuidValue.raw('00000000-0000-0000-0000-000000000020'),
        updatedBy: const UuidValue.raw('00000000-0000-0000-0000-000000000020'),
        createdAt: DateTime.utc(2026, 11, 22, 13),
        updatedAt: DateTime.utc(2026, 11, 22, 14, 30),
      ),
      FaqEntry(
        id: const UuidValue.raw('00000000-0000-0000-0000-000000000012'),
        academicCenterId: caComp,
        question: 'Como solicito a carteirinha de matrícula?',
        answer: 'Preencha o formulário disponível no início do semestre.',
        category: 'Carteirinhas',
        createdBy: const UuidValue.raw('00000000-0000-0000-0000-000000000020'),
        updatedBy: const UuidValue.raw('00000000-0000-0000-0000-000000000020'),
        createdAt: DateTime.utc(2026, 11, 20, 10),
        updatedAt: DateTime.utc(2026, 11, 20, 11),
      ),
    ];
    gateway.entriesByCa[caMath] = [
      FaqEntry(
        id: const UuidValue.raw('00000000-0000-0000-0000-000000000021'),
        academicCenterId: caMath,
        question: 'Qual o horário da biblioteca?',
        answer: 'Funciona das 8h às 20h.',
        category: 'Geral',
        createdBy: const UuidValue.raw('00000000-0000-0000-0000-000000000020'),
        updatedBy: const UuidValue.raw('00000000-0000-0000-0000-000000000020'),
        createdAt: DateTime.utc(2026, 11, 21, 9),
        updatedAt: DateTime.utc(2026, 11, 21, 9),
      ),
    ];
  });

  FaqController createController({
    UuidValue? caId = caComp,
    String? caName = 'CA de Computação',
    bool canManage = false,
  }) => FaqController(
    gateway: gateway,
    academicCenterId: caId,
    academicCenterName: caName,
    canManage: canManage,
    searchDebounce: Duration.zero,
  );

  Future<void> pumpFaqPage(
    WidgetTester tester,
    FaqController controller,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: createDarkTheme(),
        home: Scaffold(body: FaqPage(controller: controller)),
      ),
    );
    await tester.pumpAndSettle();
  }

  testWidgets(
    'lista perguntas do CA ativo, expande resposta e oculta perguntas de outro CA',
    (tester) async {
      final controller = createController();
      addTearDown(controller.dispose);

      await pumpFaqPage(tester, controller);

      expect(find.text('CA de Computação'), findsOneWidget);
      expect(find.text('Como solicitar apoio acadêmico?'), findsOneWidget);
      expect(
        find.text('Como solicito a carteirinha de matrícula?'),
        findsOneWidget,
      );
      expect(find.text('Qual o horário da biblioteca?'), findsNothing);
      expect(
        find.text('Entre em contato com a gestão do CA.'),
        findsNothing,
      );

      await tester.tap(find.text('Como solicitar apoio acadêmico?'));
      await tester.pumpAndSettle();

      expect(
        find.text('Entre em contato com a gestão do CA.'),
        findsOneWidget,
      );
      expect(find.textContaining('Atualizado em'), findsOneWidget);

      await tester.tap(find.text('Como solicitar apoio acadêmico?'));
      await tester.pumpAndSettle();

      expect(
        find.text('Entre em contato com a gestão do CA.'),
        findsNothing,
      );
    },
  );

  testWidgets('filtra por categoria via chips horizontais e restaura em Todas', (
    tester,
  ) async {
    final controller = createController();
    addTearDown(controller.dispose);

    await pumpFaqPage(tester, controller);

    await tester.tap(find.widgetWithText(ChoiceChip, 'Carteirinhas'));
    await tester.pumpAndSettle();

    expect(
      find.text('Como solicito a carteirinha de matrícula?'),
      findsOneWidget,
    );
    expect(find.text('Como solicitar apoio acadêmico?'), findsNothing);

    await tester.tap(find.widgetWithText(ChoiceChip, 'Todas'));
    await tester.pumpAndSettle();

    expect(find.text('Como solicitar apoio acadêmico?'), findsOneWidget);
    expect(
      find.text('Como solicito a carteirinha de matrícula?'),
      findsOneWidget,
    );
  });

  testWidgets(
    'busca textual, exibe estado vazio sem correspondência e limpa filtros',
    (tester) async {
      final controller = createController();
      addTearDown(controller.dispose);

      await pumpFaqPage(tester, controller);

      await tester.enterText(find.byType(TextField), 'matrícula');
      await tester.pumpAndSettle();

      expect(
        find.text('Como solicito a carteirinha de matrícula?'),
        findsOneWidget,
      );
      expect(find.text('Como solicitar apoio acadêmico?'), findsNothing);
      expect(find.byTooltip('Limpar busca'), findsOneWidget);

      await tester.enterText(find.byType(TextField), 'termo sem correspondência');
      await tester.pumpAndSettle();

      expect(find.text('Nenhuma pergunta encontrada'), findsOneWidget);
      expect(find.text('Limpar filtros'), findsOneWidget);

      await tester.tap(find.text('Limpar filtros'));
      await tester.pumpAndSettle();

      expect(find.text('Como solicitar apoio acadêmico?'), findsOneWidget);
      expect(
        find.text('Como solicito a carteirinha de matrícula?'),
        findsOneWidget,
      );
    },
  );

  testWidgets('exibe estado vazio quando CA não possui perguntas cadastradas', (
    tester,
  ) async {
    const emptyCa = UuidValue.raw('00000000-0000-0000-0000-000000000009');
    final controller = createController(caId: emptyCa, caName: 'CA Vazio');
    addTearDown(controller.dispose);

    await pumpFaqPage(tester, controller);

    expect(find.text('Nenhuma pergunta frequente disponível'), findsOneWidget);
  });

  testWidgets('permite tentar novamente após erro de carregamento', (
    tester,
  ) async {
    gateway.nextError = FaqException(
      kind: FaqFailureKind.serviceUnavailable,
      cause: Exception('503'),
      stackTrace: StackTrace.current,
    );
    final controller = createController();
    addTearDown(controller.dispose);

    await pumpFaqPage(tester, controller);

    expect(find.textContaining('temporariamente indisponível'), findsOneWidget);

    await tester.tap(find.text('Tentar novamente'));
    await tester.pumpAndSettle();

    expect(find.text('Como solicitar apoio acadêmico?'), findsOneWidget);
  });

  testWidgets(
    'estudante comum não vê botões de gestão; gestor consegue criar, editar e excluir',
    (tester) async {
      final studentController = createController(canManage: false);
      addTearDown(studentController.dispose);

      await pumpFaqPage(tester, studentController);
      expect(find.text('Nova pergunta'), findsNothing);

      final managerController = createController(canManage: true);
      addTearDown(managerController.dispose);

      await pumpFaqPage(tester, managerController);
      expect(find.text('Nova pergunta'), findsOneWidget);

      await tester.tap(find.text('Nova pergunta'));
      await tester.pumpAndSettle();

      await tester.enterText(
        find.widgetWithText(TextFormField, 'Pergunta'),
        'Como acessar a sala de estudos?',
      );
      await tester.enterText(
        find.widgetWithText(TextFormField, 'Resposta'),
        'Solicite acesso na secretaria do CA.',
      );
      await tester.enterText(
        find.widgetWithText(TextFormField, 'Categoria (opcional)'),
        'Espaço Físico',
      );
      await tester.tap(find.widgetWithText(FilledButton, 'Salvar'));
      await tester.pumpAndSettle();

      expect(find.text('Como acessar a sala de estudos?'), findsOneWidget);

      await tester.tap(find.text('Como acessar a sala de estudos?'));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Editar'));
      await tester.pumpAndSettle();

      await tester.enterText(
        find.widgetWithText(TextFormField, 'Resposta'),
        'Solicite acesso na secretaria do CA no bloco B.',
      );
      await tester.tap(find.widgetWithText(FilledButton, 'Salvar'));
      await tester.pumpAndSettle();

      expect(
        find.text('Solicite acesso na secretaria do CA no bloco B.'),
        findsOneWidget,
      );

      await tester.tap(find.text('Excluir'));
      await tester.pumpAndSettle();
      await tester.tap(find.widgetWithText(FilledButton, 'Remover'));
      await tester.pumpAndSettle();

      expect(find.text('Como acessar a sala de estudos?'), findsNothing);
    },
  );

  testWidgets('integra a aba FAQ na navegação principal do CApp', (
    tester,
  ) async {
    final h = IdentityHarness(uid: 'user-a');
    h.store.results['user-a'] = SessionCacheFound(
      testVerification(student: true),
    );
    final readiness = BackendReadinessController(checkReadiness: () async {});
    final faqController = createController();

    try {
      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
          electionHome: const Text('Início do CApp'),
          faqHome: FaqPage(controller: faqController),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Início do CApp'), findsOneWidget);
      expect(find.byType(FaqPage), findsNothing);

      await tester.tap(find.byIcon(Icons.help_outline));
      await tester.pumpAndSettle();

      expect(find.byType(FaqPage), findsOneWidget);
      expect(find.text('Como solicitar apoio acadêmico?'), findsOneWidget);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
      await tester.runAsync(faqController.dispose);
      readiness.dispose();
    }
  });
}
