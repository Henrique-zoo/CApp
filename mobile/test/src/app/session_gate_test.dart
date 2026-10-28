/// Testes do gate e da navegação pertencente aos seus ramos.
///
/// Os cenários isolados verificam os destinos publicados pelo controlador.
/// A retomada usa a composição real para conferir descarte do Navigator.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/app/app.dart';
import 'package:capp/src/app/session_gate.dart';
import 'package:capp/src/app/session_lifecycle_observer.dart';
import 'package:capp/src/modules/identity/identity.dart';

import '../../support/identity_fakes.dart';

/// Apresenta um marcador dentro da navegação própria de um ramo.
Widget _branch(String marker) =>
    MaterialApp(home: Scaffold(body: Text(marker)));

/// Monta o gate real com marcadores que identificam os builders selecionados.
Widget _gateHost(IdentityHarness h) => SessionGate(
  controller: h.controller,
  restoringBuilder: (_) => _branch('restoring'),
  entryBuilder: (_, _) => _branch('entry'),
  signedInBuilder: (_, state) {
    final uid = state.firebaseUid;
    return _branch('ready: $uid');
  },
  restorationFailedBuilder: (_, _) => _branch('restoration-failed'),
);

/// Registra seleção, substituição de controlador e descarte de navegação.
void main() {
  testWidgets(
    'aguarda autenticação e cache antes de selecionar a área principal',
    (tester) async {
      final h = IdentityHarness(uid: 'user-a');
      final read = Completer<SessionCacheLoadResult>();
      h.store.onLoad = (_) => read.future;
      addTearDown(() {
        if (!read.isCompleted) read.complete(const SessionCacheMissing());
      });
      try {
        await tester.pumpWidget(_gateHost(h));
        expect(find.text('restoring'), findsOneWidget);
        h.controller.start();
        await tester.pump();
        expect(find.text('restoring'), findsOneWidget);
        read.complete(SessionCacheFound(testVerification()));
        await tester.pumpAndSettle();
        expect(find.text('ready: user-a'), findsOneWidget);
        expect(h.gateway.calls, 0);
        expect(h.microsoft.signInCalls, 0);
      } finally {
        await tester.pumpWidget(const SizedBox.shrink());
        await tester.runAsync(h.dispose);
      }
    },
  );

  testWidgets('cache não seleciona a área principal sem autenticação', (
    tester,
  ) async {
    final h = IdentityHarness();
    try {
      h.store.results['user-a'] = SessionCacheFound(testVerification());
      h.controller.start();
      await tester.pumpWidget(_gateHost(h));
      await tester.pump();
      expect(find.text('entry'), findsOneWidget);
      expect(find.text('ready: user-a'), findsNothing);
      expect(h.store.loadedUids, isEmpty);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets('autenticação com perfil expirado seleciona a mesma entrada', (
    tester,
  ) async {
    final h = IdentityHarness(uid: 'user-a');
    try {
      h.store.results['user-a'] = SessionCacheFound(
        testVerification(expiresAt: h.instant),
      );
      h.controller.start();
      await tester.pumpWidget(_gateHost(h));
      await tester.pump();
      expect(h.controller.state, isA<SessionSignedIn>());
      expect(find.text('entry'), findsOneWidget);
      expect(find.text('ready: user-a'), findsNothing);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets('falha de acompanhamento possui ramo próprio', (tester) async {
    final h = IdentityHarness(uid: 'user-a');
    try {
      h.controller.start();
      await tester.pumpWidget(_gateHost(h));
      await tester.pump();
      final cause = Exception('observação indisponível');
      h.auth.fail(cause, StackTrace.current);
      await tester.pumpAndSettle();
      expect(find.text('restoration-failed'), findsOneWidget);
      expect(find.text('entry'), findsNothing);
      expect(
        (h.controller.state as SessionRestorationFailed).cause,
        same(cause),
      );
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });

  testWidgets('trocar controlador não reutiliza o ramo pronto anterior', (
    tester,
  ) async {
    final first = IdentityHarness(uid: 'user-a');
    final second = IdentityHarness(uid: 'user-b');
    try {
      first.store.results['user-a'] = SessionCacheFound(testVerification());
      first.controller.start();
      await tester.pumpWidget(_gateHost(first));
      await tester.pump();
      expect(find.text('ready: user-a'), findsOneWidget);
      second.controller.start();
      await tester.pumpWidget(_gateHost(second));
      await tester.pump();
      expect(find.text('ready: user-a'), findsNothing);
      expect(find.text('entry'), findsOneWidget);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(first.dispose);
      await tester.runAsync(second.dispose);
    }
  });

  testWidgets('retomada expirada descarta também a navegação anterior', (
    tester,
  ) async {
    final h = IdentityHarness(uid: 'user-a');
    try {
      h.store.results['user-a'] = SessionCacheFound(testVerification());
      h.controller.start();
      await tester.pumpWidget(
        SessionLifecycleObserver(
          controller: h.controller,
          child: CApp(sessionController: h.controller),
        ),
      );
      await tester.pump();
      final context = tester.element(find.text('Olá, Pessoa Exemplo'));
      unawaited(
        Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => const Scaffold(body: Text('Tela interna anterior')),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Tela interna anterior'), findsOneWidget);
      h.instant = h.instant.add(const Duration(hours: 2));
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pump();
      expect(find.text('Tela interna anterior'), findsNothing);
      expect(find.text('Entrar com Microsoft'), findsOneWidget);
      expect(h.gateway.calls, 0);
      expect(
        Navigator.of(tester.element(find.text('Entrar com Microsoft')))
            .canPop(),
        isFalse,
      );
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(h.dispose);
    }
  });
}
