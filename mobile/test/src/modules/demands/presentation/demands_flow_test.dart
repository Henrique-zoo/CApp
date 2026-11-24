/// Testes de widget cobrindo fóruns, filtros, criação, detalhes e navegação.
library;

import 'package:capp/src/app/app.dart';
import 'package:capp/src/app/theme.dart';
import 'package:capp/src/modules/demands/demands.dart';
import 'package:capp/src/modules/demands/demands_widgets.dart';
import 'package:capp/src/presentation/presentation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../support/account_identity_fakes.dart';
import '../../../../support/demands_fakes.dart';

void main() {
  Widget wrapWithTheme(Widget child) =>
      MaterialApp(theme: createDarkTheme(), home: child);

  testWidgets(
    'alterna entre fórum institucional e espaço físico e filtra demandas',
    (tester) async {
    final gateway = FakeDemandsGateway(
      initialDemands: [
        sampleDemand(
          id: const UuidValue.raw('00000000-0000-0000-0000-000000000020'),
          forum: DemandForum.institutional,
          title: 'Revisão de pré-requisito',
          courseName: 'Engenharia de Software',
          status: DemandStatus.forwardedToCoordination,
          authorId: testUserId,
        ),
        sampleDemand(
          id: const UuidValue.raw('00000000-0000-0000-0000-000000000021'),
          forum: DemandForum.physicalSpace,
          title: 'Reparo da porta',
          status: DemandStatus.open,
          authorId: testUserId,
        ),
        sampleDemand(
          id: const UuidValue.raw('00000000-0000-0000-0000-000000000022'),
          forum: DemandForum.physicalSpace,
          title: 'Troca da lâmpada',
          status: DemandStatus.completed,
          authorId: testOtherUserId,
        ),
      ],
    );
    final controller = DemandsController(
      gateway: gateway,
      academicCenterId: testCaId,
      academicCenterName: 'CA de Computação',
      currentUserId: testUserId,
      userCourseName: 'Engenharia de Software',
    );
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      wrapWithTheme(DemandsHome(controller: controller)),
    );
    await tester.pumpAndSettle();

    expect(find.text('CA de Computação'), findsOneWidget);
    expect(find.text('Revisão de pré-requisito'), findsOneWidget);
    expect(find.text('Enviada à coordenação'), findsWidgets);

    await tester.tap(find.text('Espaço Físico'));
    await tester.pumpAndSettle();

    expect(find.text('Reparo da porta'), findsOneWidget);
    expect(find.text('Troca da lâmpada'), findsOneWidget);
    expect(find.text('Enviada à coordenação'), findsNothing);

    await tester.tap(find.text('Minhas demandas'));
    await tester.pumpAndSettle();

    expect(find.text('Reparo da porta'), findsOneWidget);
    expect(find.text('Troca da lâmpada'), findsNothing);

    await tester.tap(find.text('Minhas demandas'));
    await tester.pumpAndSettle();

    expect(find.text('Troca da lâmpada'), findsOneWidget);

    await tester.enterText(find.byType(TextField), 'lâmpada');
    await tester.pumpAndSettle();

    expect(find.text('Troca da lâmpada'), findsOneWidget);
    expect(find.text('Reparo da porta'), findsNothing);
  });

  testWidgets(
    'exibe aviso claro de 403 no institucional sem quebrar navegação',
    (tester) async {
    final gateway = FakeDemandsGateway(
      forbidInstitutional: true,
      initialDemands: [
        sampleDemand(
          forum: DemandForum.physicalSpace,
          title: 'Limpeza da sala',
        ),
      ],
    );
    final controller = DemandsController(
      gateway: gateway,
      academicCenterId: testCaId,
      currentUserId: testUserId,
    );
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      wrapWithTheme(DemandsHome(controller: controller)),
    );
    await tester.pumpAndSettle();

    expect(find.byType(WarningBox), findsOneWidget);
    expect(
      find.textContaining('O fórum institucional é restrito'),
      findsOneWidget,
    );

    await tester.tap(find.text('Ir para Espaço Físico'));
    await tester.pumpAndSettle();

    expect(find.byType(WarningBox), findsNothing);
    expect(find.text('Limpeza da sala'), findsOneWidget);
  });

  testWidgets('exibe estados vazios para filtro pessoal e erro de listagem', (
    tester,
  ) async {
    final gateway = FakeDemandsGateway(
      initialDemands: [
        sampleDemand(
          forum: DemandForum.physicalSpace,
          authorId: testOtherUserId,
          title: 'Demanda de colega',
        ),
      ],
    );
    final controller = DemandsController(
      gateway: gateway,
      academicCenterId: testCaId,
      currentUserId: testUserId,
      initialForum: DemandForum.physicalSpace,
    );
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      wrapWithTheme(DemandsHome(controller: controller)),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Minhas demandas'));
    await tester.pumpAndSettle();

    expect(
      find.text('Você ainda não registrou demandas neste fórum.'),
      findsOneWidget,
    );
    await tester.tap(find.text('Ver todas do fórum'));
    await tester.pumpAndSettle();

    expect(find.text('Demanda de colega'), findsOneWidget);

    gateway.listError = DemandException(
      kind: DemandFailureKind.communicationFailed,
      cause: Exception('offline'),
      stackTrace: StackTrace.current,
    );
    await controller.loadDemands();
    await tester.pumpAndSettle();

    expect(find.byType(WarningBox), findsOneWidget);
    expect(find.text('Tentar novamente'), findsOneWidget);
  });

  testWidgets(
    'cria demanda e navega para detalhes com histórico, comentário e status',
    (tester) async {
    final gateway = FakeDemandsGateway(actorIsManager: true);
    final controller = DemandsController(
      gateway: gateway,
      academicCenterId: testCaId,
      currentUserId: testUserId,
      userCourseName: 'Engenharia de Software',
      isCaManager: true,
      initialForum: DemandForum.physicalSpace,
    );
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      wrapWithTheme(DemandsHome(controller: controller)),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Nova demanda'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Registrar demanda'));
    await tester.pumpAndSettle();
    expect(find.text('Informe a descrição da demanda.'), findsOneWidget);

    await tester.enterText(
      find.widgetWithText(TextFormField, 'Título (opcional)'),
      'Porta com defeito',
    );
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Descrição'),
      'A fechadura da sala do CA não tranca.',
    );
    await tester.tap(find.text('Registrar demanda'));
    await tester.pumpAndSettle();

    expect(find.text('Porta com defeito'), findsOneWidget);

    await tester.tap(find.text('Porta com defeito'));
    await tester.pumpAndSettle();

    expect(find.byType(DemandDetailPage), findsOneWidget);
    expect(find.text('Histórico de status'), findsOneWidget);
    expect(find.text('Registro inicial: Aberta'), findsOneWidget);

    await tester.enterText(
      find.widgetWithText(TextFormField, 'Adicionar comentário'),
      'Equipe vai verificar ainda hoje.',
    );
    await tester.ensureVisible(find.text('Enviar comentário'));
    await tester.tap(find.text('Enviar comentário'));
    await tester.pumpAndSettle();

    expect(find.text('Equipe vai verificar ainda hoje.'), findsOneWidget);
    expect(find.text('Gestão do CA'), findsOneWidget);

    await tester.ensureVisible(find.text('Atualizar status'));
    await tester.tap(find.text('Atualizar status'));
    await tester.pumpAndSettle();

    expect(find.text('Enviada à coordenação'), findsNothing);
    await tester.tap(find.text('Em análise'));
    await tester.enterText(
      find.widgetWithText(TextFormField, 'Parecer / justificativa'),
      'Orçamento solicitado para troca da fechadura.',
    );
    await tester.tap(find.text('Confirmar alteração'));
    await tester.pumpAndSettle();

    expect(
      find.text('Orçamento solicitado para troca da fechadura.'),
      findsOneWidget,
    );
    expect(find.text('Aberta → Em análise'), findsOneWidget);
  });

  testWidgets('exibe aviso discreto quando comentários retornam 403', (
    tester,
  ) async {
    final gateway = FakeDemandsGateway(
      forbidComments: true,
      initialDemands: [
        sampleDemand(
          authorId: testOtherUserId,
          title: 'Tomada quebrada',
        ),
      ],
    );
    final controller = DemandsController(
      gateway: gateway,
      academicCenterId: testCaId,
      currentUserId: testUserId,
      initialForum: DemandForum.physicalSpace,
    );
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      wrapWithTheme(
        DemandDetailPage(
          controller: controller,
          demandId: const UuidValue.raw('00000000-0000-0000-0000-000000000020'),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Tomada quebrada'), findsOneWidget);
    expect(find.byType(WarningBox), findsOneWidget);
    expect(
      find.textContaining('Os comentários e interações desta demanda'),
      findsOneWidget,
    );
    expect(find.text('Enviar comentário'), findsNothing);
  });

  testWidgets('integra a aba Demandas na navegação principal do CApp', (
    tester,
  ) async {
    final h = AccountIdentityHarness(student: true);
    final readiness = BackendReadinessController(checkReadiness: () async {});
    final gateway = FakeDemandsGateway(
      initialDemands: [
        sampleDemand(
          forum: DemandForum.institutional,
          title: 'Demanda na navegação principal',
          courseName: 'Engenharia de Software',
        ),
      ],
    );
    final controller = DemandsController(
      gateway: gateway,
      academicCenterId: testCaId,
      currentUserId: testUserId,
      userCourseName: 'Engenharia de Software',
    );

    try {
      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
          demandsHome: DemandsHome(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byIcon(Icons.forum_outlined));
      await tester.pumpAndSettle();

      expect(find.text('Demanda na navegação principal'), findsOneWidget);
      expect(find.byType(DemandsHome), findsOneWidget);

      await tester.tap(find.byIcon(Icons.person_outline));
      await tester.pumpAndSettle();
      expect(find.text('Dados da conta'), findsOneWidget);
    } finally {
      controller.dispose();
      readiness.dispose();
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });
}
