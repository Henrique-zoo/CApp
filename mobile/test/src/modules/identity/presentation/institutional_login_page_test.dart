/// Testes da tela de entrada com a coordenação real e contratos simulados.
///
/// A hospedagem limita a tela ao seu estado aceito, sem testar a navegação
/// da aplicação. Verifica mensagens e interações, sem SDK ou cliente HTTP.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/identity_widgets.dart';

import '../../../../support/identity_fakes.dart';

/// Fornece atualizações à tela somente enquanto a entrada está pendente.
///
/// Os demais destinos não possuem interface nesta fixture: sua seleção
/// pertence aos testes específicos do gate.
Widget _entryHost(IdentityHarness h) => MaterialApp(
  home: StreamBuilder<SessionState>(
    stream: h.controller.states,
    initialData: h.controller.state,
    builder: (_, _) {
      final state = h.controller.state;
      if (state.access != SessionAccess.entryRequired) {
        return const SizedBox.shrink();
      }
      return InstitutionalLoginPage(controller: h.controller, state: state);
    },
  ),
);

/// Registra espera, mensagem pública, cancelamento e bloqueio de cliques.
void main() {
  testWidgets('backend pendente permanece na entrada e mostra sua recusa', (
    tester,
  ) async {
    final h = IdentityHarness();
    try {
      final backend = Completer<InstitutionalVerification>();
      h.gateway.onVerify = () => backend.future;
      addTearDown(() {
        if (!backend.isCompleted) backend.complete(testVerification());
      });
      h.controller.start();
      await tester.pumpWidget(_entryHost(h));
      await tester.pump();
      await tester.tap(find.text('Entrar com Microsoft'));
      await tester.pump();
      expect(h.auth.currentUserId, 'user-a');
      expect(find.text('Entrando...'), findsOneWidget);
      expect(
        tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNull,
      );
      backend.completeError(
        IdentityException(
          kind: IdentityFailureKind.institutionNotSupported,
          cause: Exception('recusa simulada'),
          stackTrace: StackTrace.current,
          publicMessage: 'Instituição de teste não cadastrada',
        ),
      );
      await tester.pump();
      expect(find.text('Instituição de teste não cadastrada'), findsOneWidget);
      expect(find.text('Entrar com Microsoft'), findsOneWidget);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets('cancelamento apresenta mensagem sem apagar o diagnóstico', (
    tester,
  ) async {
    final h = IdentityHarness();
    final failure = IdentityException(
      kind: IdentityFailureKind.authenticationCancelled,
      cause: Exception('cancelamento simulado'),
      stackTrace: StackTrace.current,
    );
    h.microsoft.onSignIn = () => Future.error(failure, failure.stackTrace);
    try {
      h.controller.start();
      await tester.pumpWidget(_entryHost(h));
      await tester.pump();
      await tester.tap(find.text('Entrar com Microsoft'));
      await tester.pump();
      expect(
        find.text('A autenticação foi cancelada. Você pode tentar novamente.'),
        findsOneWidget,
      );
      expect(h.controller.state.failureDetails?.cause, same(failure));
      expect(h.gateway.calls, 0);
      expect(
        tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNotNull,
      );
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets('cliques antes da reconstrução não duplicam a entrada', (
    tester,
  ) async {
    final h = IdentityHarness();
    final backend = Completer<InstitutionalVerification>();
    h.gateway.onVerify = () => backend.future;
    addTearDown(() {
      if (!backend.isCompleted) backend.complete(testVerification());
    });
    try {
      h.controller.start();
      await tester.pumpWidget(_entryHost(h));
      await tester.pump();
      final press = tester
          .widget<FilledButton>(find.byType(FilledButton))
          .onPressed!;
      press();
      press();
      await tester.pumpAndSettle();
      expect(h.microsoft.signInCalls, 1);
      expect(h.gateway.calls, 1);
      expect(find.text('Entrando...'), findsOneWidget);
      backend.complete(testVerification());
      await tester.pump();
      expect(h.store.saveCalls, 1);
      expect(tester.takeException(), isNull);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });
}
