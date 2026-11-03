/// Integração da ligação visual de prontidão com sessão e navegação reais.
///
/// Confere que uma consulta pendente ou falha não bloqueia cache e login, e que
/// uma nova tentativa não descarta a rota visitada. Todos os serviços são locais.
library;

import 'dart:async';

import 'package:capp/src/app/app.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../support/identity_fakes.dart';

/// Registra o comportamento global do aviso sem configurar plugins ou rede.
void main() {
  testWidgets(
    'cache válido entra durante consulta e preserva rota ao repetir',
    (tester) async {
      final h = IdentityHarness(uid: 'user-a');
      h.store.results['user-a'] = SessionCacheFound(testVerification());
      final response = Completer<void>();
      final cause = Exception('diagnóstico privado do transporte');
      var calls = 0;
      final readiness = BackendReadinessController(
        checkReadiness: () {
          calls++;
          return calls == 1 ? response.future : Future<void>.value();
        },
      );
      try {
        h.controller.start();
        readiness.start();
        await tester.pumpWidget(
          CApp(
            sessionController: h.controller,
            backendReadinessController: readiness,
          ),
        );
        await tester.pump();
        expect(find.text('Pessoa Exemplo'), findsOneWidget);
        expect(
          find.text('Verificando disponibilidade do servidor...'),
          findsOneWidget,
        );
        expect(h.gateway.calls, 0);
        expect(h.microsoft.signInCalls, 0);

        response.completeError(cause);
        await tester.pumpAndSettle();
        expect(
          find.textContaining('Não foi possível confirmar'),
          findsOneWidget,
        );
        expect(find.textContaining('diagnóstico privado'), findsNothing);
        expect(h.controller.state.access, SessionAccess.ready);
        expect((readiness.state as BackendReadinessFailed).cause, same(cause));

        final context = tester.element(find.text('Pessoa Exemplo'));
        unawaited(
          Navigator.of(context).push(
            MaterialPageRoute<void>(
              builder: (_) => const Scaffold(body: Text('Rota preservada')),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('Tentar novamente'), findsOneWidget);
        await tester.tap(find.text('Tentar novamente'));
        await tester.pumpAndSettle();
        expect(calls, 2);
        expect(readiness.state, isA<BackendReadinessReady>());
        expect(find.textContaining('Não foi possível confirmar'), findsNothing);
        expect(find.text('Rota preservada'), findsOneWidget);
        expect(tester.takeException(), isNull);
      } finally {
        if (!response.isCompleted) response.complete();
        await tester.pumpWidget(const SizedBox.shrink());
        readiness.dispose();
        await tester.runAsync(h.dispose);
      }
    },
  );

  testWidgets('login não aguarda a consulta nem a repete ao trocar de tela', (
    tester,
  ) async {
    final h = IdentityHarness();
    final response = Completer<void>();
    var calls = 0;
    final readiness = BackendReadinessController(
      checkReadiness: () {
        calls++;
        return response.future;
      },
    );
    try {
      h.controller.start();
      readiness.start();
      await tester.pumpWidget(
        CApp(
          sessionController: h.controller,
          backendReadinessController: readiness,
        ),
      );
      await tester.pump();
      await tester.tap(find.text('Entrar com Microsoft'));
      await tester.pump();
      await tester.pump();
      expect(find.text('Pessoa Exemplo'), findsOneWidget);
      expect(readiness.state, isA<BackendReadinessChecking>());
      expect(h.gateway.calls, 1);
      expect(calls, 1);

      response.complete();
      await tester.pumpAndSettle();
      expect(
        find.text('Verificando disponibilidade do servidor...'),
        findsNothing,
      );
      expect(find.text('Pessoa Exemplo'), findsOneWidget);
      expect(tester.takeException(), isNull);
    } finally {
      if (!response.isCompleted) response.complete();
      await tester.pumpWidget(const SizedBox.shrink());
      readiness.dispose();
      await tester.runAsync(h.dispose);
    }
  });
}
