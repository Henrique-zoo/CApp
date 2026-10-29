/// Testes da composição visual completa do CApp.
///
/// As responsabilidades de gate e tela de entrada são verificadas em suas
/// suítes específicas. Esta composição verifica entrada, avisos de persistência
/// e saída pela Home, incluindo falhas do provedor e da limpeza local.
/// Os contratos controlados não inicializam SDKs ou conexões externas.
library;

import 'dart:async';

import 'package:capp/src/app/app.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../support/identity_fakes.dart';

/// Registra os cenários que atravessam a composição de widgets da aplicação.
void main() {
  late BackendReadinessController readiness;

  setUp(() {
    readiness = BackendReadinessController(checkReadiness: () async {});
  });
  tearDown(() => readiness.dispose());

  testWidgets('não estudante entra sem uma etapa manual de verificação', (
    tester,
  ) async {
    final h = IdentityHarness();
    try {
      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Entrar com Microsoft'));
      await tester.pumpAndSettle();
      expect(find.text('Pessoa Exemplo'), findsOneWidget);
      expect(
        find.textContaining('Você não foi reconhecido como estudante'),
        findsOneWidget,
      );
      expect(find.text('Verificar instituição'), findsNothing);
      expect(find.text('Atualizar verificação'), findsNothing);
      expect(h.store.saveCalls, 1);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets('falha exclusiva de cache não desfaz a entrada', (tester) async {
    final h = IdentityHarness();
    try {
      h.store.onSave = (_, _) async => throw Exception('gravação indisponível');
      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Entrar com Microsoft'));
      await tester.pumpAndSettle();
      expect(find.text('Pessoa Exemplo'), findsOneWidget);
      expect(
        find.textContaining('não foi possível salvar os dados'),
        findsOneWidget,
      );
      expect(find.byType(SnackBar), findsOneWidget);
      expect(tester.takeException(), isNull);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets('botão Sair encerra a sessão e retorna à tela de entrada', (
    tester,
  ) async {
    final h = IdentityHarness(uid: 'user-a');
    h.store.results['user-a'] = SessionCacheFound(testVerification());
    try {
      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Pessoa Exemplo'), findsOneWidget);

      await tester.tap(find.text('Sair'));
      await tester.pumpAndSettle();

      expect(find.text('Entrar com Microsoft'), findsOneWidget);
      expect(find.text('Dados institucionais'), findsNothing);
      expect(h.auth.currentUserId, isNull);
      expect(h.auth.signOutCalls, 1);
      expect(h.store.results.containsKey('user-a'), isFalse);
      expect(tester.takeException(), isNull);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets(
    'saída impede cliques repetidos e apresenta falha sem perder a conta',
    (tester) async {
      final h = IdentityHarness(uid: 'user-a');
      h.store.results['user-a'] = SessionCacheFound(testVerification());
      final provider = Completer<void>();
      final cause = IdentityException(
        kind: IdentityFailureKind.credentialsRejected,
        cause: Exception('falha do provedor ao sair'),
        stackTrace: StackTrace.current,
      );
      h.auth.onSignOut = () async {
        await provider.future;
        throw cause;
      };
      try {
        h.controller.start();
        await tester.pumpWidget(
          CApp(
            sessionController: h.controller,
            backendReadinessController: readiness,
          ),
        );
        await tester.pumpAndSettle();
        final button = tester.widget<TextButton>(
          find.widgetWithText(TextButton, 'Sair'),
        );
        button.onPressed!();
        button.onPressed!();
        await tester.pumpAndSettle();
        expect(h.auth.signOutCalls, 1);
        final busy = tester.widget<TextButton>(
          find.widgetWithText(TextButton, 'Saindo...'),
        );
        expect(busy.onPressed, isNull);

        provider.complete();
        await tester.pumpAndSettle();

        expect(find.text('Pessoa Exemplo'), findsOneWidget);
        expect(
          find.text('Não foi possível sair da conta. Tente novamente.'),
          findsOneWidget,
        );
        expect(find.byType(SnackBar), findsOneWidget);
        expect(h.controller.state.failureDetails?.cause, same(cause));
        expect(
          tester
              .widget<TextButton>(find.widgetWithText(TextButton, 'Sair'))
              .onPressed,
          isNotNull,
        );
        expect(tester.takeException(), isNull);
      } finally {
        if (!provider.isCompleted) provider.complete();
        await tester.pumpWidget(const SizedBox.shrink());
        await tester.runAsync(h.dispose);
      }
    },
  );

  testWidgets('falha de limpeza é informada na entrada depois do logout', (
    tester,
  ) async {
    final h = IdentityHarness(uid: 'user-a');
    h.store.results['user-a'] = SessionCacheFound(testVerification());
    h.store.onRemove = (_) async => throw Exception('remoção indisponível');
    try {
      h.controller.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Sair'));
      await tester.pumpAndSettle();
      expect(h.auth.currentUserId, isNull);
      expect(find.text('Entrar com Microsoft'), findsOneWidget);
      expect(
        find.textContaining('Você saiu da conta, mas não foi possível remover'),
        findsOneWidget,
      );
      expect(
        h.controller.state.failureDetails?.cause,
        isA<SessionCacheRemovalException>(),
      );
      expect(tester.takeException(), isNull);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });
}
