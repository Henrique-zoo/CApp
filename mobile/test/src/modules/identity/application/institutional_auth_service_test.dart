/// Testes da operação institucional sem transporte, SDK ou armazenamento.
///
/// Verifica a associação entre credencial, UID e resposta, incluindo mudanças
/// de conta durante interações assíncronas. A sessão compartilhada não é iniciada.
library;

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/application/errors/microsoft_interaction_required_exception.dart';
import 'package:capp/src/modules/identity/identity.dart';

import '../../../../support/identity_fakes.dart';

/// Cria a composição controlada e registra seu encerramento.
IdentityHarness _harness({String? uid}) {
  final harness = IdentityHarness(uid: uid);

  addTearDown(harness.dispose);

  return harness;
}

/// Registra o contrato de identidade antes e depois da verificação externa.
void main() {
  test(
    'login estabelece UID sem solicitar token Microsoft ou verificar vínculo',
    () async {
      final h = _harness();

      h.microsoft.onSignIn = () async {
        h.auth.setUserId('user-a');
      };

      await h.service.signIn();

      expect(h.auth.currentUserId, 'user-a');
      expect(h.gateway.calls, 0);
      expect(h.microsoft.signInCalls, 1);
      expect(h.microsoft.currentUserTokenCalls, 0);
      expect(h.store.saveCalls, 0);
      expect(h.controller.state, isA<SessionRestoring>());
    },
  );

  test('login sem identidade não consulta o backend', () async {
    final h = _harness();

    h.microsoft.onSignIn = () async {};

    await expectLater(
      h.service.signIn(),
      throwsA(isA<SessionIdentityChangedException>()),
    );

    expect(h.gateway.calls, 0);
  });

  test('renovação sem conta falha antes de solicitar credenciais', () async {
    final h = _harness();

    await expectLater(
      h.service.renewVerification(),
      throwsA(isA<SessionIdentityChangedException>()),
    );

    expect(h.microsoft.currentUserTokenCalls, 0);
    expect(h.gateway.calls, 0);
  });

  test('renovação silenciosa verifica a conta atual sem novo login', () async {
    final h = _harness(uid: 'user-a');
    final result = await h.service.renewVerification(allowInteraction: false);

    expect(result.firebaseUid, 'user-a');
    expect(result.verification, same(h.gateway.verification));
    expect(h.microsoft.interactionPermissions, [false]);
    expect(h.microsoft.signInCalls, 0);
    expect(h.gateway.calls, 1);
    expect(h.store.saveCalls, 0);
  });

  test(
    'necessidade de interação propaga sem verificar ou perder a conta',
    () async {
      final h = _harness(uid: 'user-a');
      final failure = MicrosoftInteractionRequiredException(
        cause: Exception('Consentimento pendente'),
        stackTrace: StackTrace.current,
      );

      h.microsoft.onGetAccessTokenForCurrentUser = () async => throw failure;

      await expectLater(
        h.service.renewVerification(allowInteraction: false),
        throwsA(same(failure)),
      );

      expect(h.microsoft.interactionPermissions, [false]);
      expect(h.gateway.calls, 0);
      expect(h.auth.currentUserId, 'user-a');
    },
  );

  test('troca de conta no provedor não é aceita como renovação', () async {
    final h = _harness(uid: 'user-a');

    h.microsoft.onGetAccessTokenForCurrentUser = () async {
      h.auth.setUserId('user-b');

      return 'delegated-test-token';
    };

    await expectLater(
      h.service.renewVerification(),
      throwsA(isA<SessionIdentityChangedException>()),
    );

    expect(h.gateway.calls, 0);
  });

  test('mudança de conta durante o backend invalida sua resposta', () async {
    final h = _harness(uid: 'user-a');
    final backend = Completer<InstitutionalVerification>();

    h.gateway.onVerify = () => backend.future;

    addTearDown(() {
      if (!backend.isCompleted) backend.complete(testVerification());
    });

    final rejected = expectLater(
      h.service.renewVerification(),
      throwsA(isA<SessionIdentityChangedException>()),
    );

    await flushSessionEvents();
    h.auth.setUserId('user-b');
    backend.complete(testVerification());
    await rejected;

    expect(h.gateway.calls, 1);
    expect(h.store.saveCalls, 0);
  });

  test('recusa externa mantém a exceção original e não grava cache', () async {
    final h = _harness(uid: 'user-a');
    final failure = IdentityException(
      kind: IdentityFailureKind.institutionNotSupported,
      cause: Exception('recusa simulada'),
      stackTrace: StackTrace.current,
    );

    h.gateway.onVerify = () => Future.error(failure, failure.stackTrace);

    await expectLater(h.service.renewVerification(), throwsA(same(failure)));

    expect(h.store.saveCalls, 0);
    expect(h.auth.currentUserId, 'user-a');
  });
}
