/// Contratos do adaptador composto de Firebase e credenciais Microsoft.
///
/// Uma renovação institucional não deve executar novo login Firebase nem abrir
/// autorização por falha de rede. Esta suíte controla FirebaseAuth e a porta de
/// tokens para verificar essas fronteiras, mudanças de UID e ordem de limpeza.
/// Não inicializa plugins, navegador ou serviços remotos; o cadastro externo e
/// a autenticação nativa ainda dependem de validação em dispositivo.
library;

import 'package:firebase_auth/firebase_auth.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/application/errors/identity_exception.dart';
import 'package:capp/src/modules/identity/application/errors/microsoft_interaction_required_exception.dart';
import 'package:capp/src/modules/identity/application/errors/session_identity_changed_exception.dart';
import 'package:capp/src/modules/identity/application/ports/microsoft_token_provider.dart';
import 'package:capp/src/modules/identity/infrastructure/firebase/firebase_microsoft_auth.dart';

/// Exercita os efeitos de autenticação e os limites de interação do adaptador.
void main() {
  late _Auth auth;
  late _Tokens tokens;
  late FirebaseMicrosoftAuth adapter;
  final requiredUi = MicrosoftInteractionRequiredException(
    cause: StateError('Interaction required'),
    stackTrace: StackTrace.current,
  );

  setUp(() {
    auth = _Auth()..user = _User('a');
    tokens = _Tokens();
    adapter = FirebaseMicrosoftAuth(auth, microsoftTokenProvider: tokens);
  });

  test('obtém token da conta atual sem novo login Firebase', () async {
    expect(await adapter.getAccessTokenForCurrentUser(), 'graph-silent');
    expect(auth.signInCalls, 0);
    expect(tokens.interactiveCalls, 0);
  });

  test('conta ausente impede solicitar tokens', () async {
    auth.user = null;

    expect(
      () => adapter.getAccessTokenForCurrentUser(),
      throwsA(isA<SessionIdentityChangedException>()),
    );
    expect(tokens.silentCalls, 0);
    expect(tokens.interactiveCalls, 0);
  });

  test(
    'modo silencioso propaga necessidade de interação sem abrir UI',
    () async {
      tokens.silent = () async => throw requiredUi;

      await expectLater(
        adapter.getAccessTokenForCurrentUser(allowInteraction: false),
        throwsA(same(requiredUi)),
      );

      expect(tokens.interactiveCalls, 0);
    },
  );

  test('ação interativa usa fallback MSAL e hint da conta Firebase', () async {
    tokens.silent = () async => throw requiredUi;

    expect(await adapter.getAccessTokenForCurrentUser(), 'graph-interactive');
    expect(tokens.interactiveCalls, 1);
    expect(tokens.loginHint, 'a@example.test');
    expect(auth.signInCalls, 0);
  });

  test('falha de rede não abre autenticação', () async {
    final failure = IdentityException(
      kind: IdentityFailureKind.communicationFailed,
      cause: StateError('Network'),
      stackTrace: StackTrace.current,
    );

    tokens.silent = () async => throw failure;

    await expectLater(
      adapter.getAccessTokenForCurrentUser(),
      throwsA(same(failure)),
    );

    expect(tokens.interactiveCalls, 0);
  });

  test(
    'mudança do UID durante obtenção do token descarta o resultado',
    () async {
      tokens.silent = () async {
        auth.user = _User('b');

        return 'graph-a';
      };

      await expectLater(
        adapter.getAccessTokenForCurrentUser(),
        throwsA(isA<SessionIdentityChangedException>()),
      );
    },
  );

  test('falha ao limpar MSAL preserva autenticação Firebase', () async {
    tokens.clear = () async => throw StateError('Clear failed');

    await expectLater(adapter.signOut(), throwsStateError);

    expect(auth.user?.uid, 'a');
    expect(auth.signOutCalls, 0);
  });

  test(
    'mudança de conta antes da interação impede abrir autorização',
    () async {
      tokens.silent = () async {
        auth.user = _User('b');

        throw requiredUi;
      };

      await expectLater(
        adapter.getAccessTokenForCurrentUser(),
        throwsA(isA<SessionIdentityChangedException>()),
      );

      expect(tokens.interactiveCalls, 0);
    },
  );

  test('mudança de conta durante autorização descarta o token', () async {
    tokens.silent = () async => throw requiredUi;
    tokens.onInteractive = () async {
      auth.user = _User('b');

      return 'graph-a';
    };

    await expectLater(
      adapter.getAccessTokenForCurrentUser(),
      throwsA(isA<SessionIdentityChangedException>()),
    );
  });

  test('saída limpa MSAL antes do Firebase', () async {
    final events = <String>[];

    tokens.clear = () async => events.add('msal');
    auth.onSignOut = () => events.add('firebase');

    await adapter.signOut();

    expect(events, ['msal', 'firebase']);
    expect(auth.currentUser, isNull);
  });

  test(
    'login autentica no Firebase sem limpar cache ou solicitar token MSAL',
    () async {
      auth.user = null;

      final events = <String>[];

      tokens.clear = () async => events.add('clear');
      auth.onSignIn = () => events.add('firebase');
      tokens.silent = () async {
        events.add('msal');

        return 'graph-msal';
      };

      await adapter.signIn();

      expect(events, ['firebase']);
      expect(tokens.interactiveCalls, 0);
      expect(auth.currentUser?.uid, 'a');
      expect(auth.parameters['prompt'], 'login');
    },
  );
}

/// Usuário Firebase mínimo, sem acesso ao plugin de plataforma.
class _User extends Fake implements User {
  _User(this.uid);

  @override
  final String uid;

  @override
  String? get email => '$uid@example.test';

  @override
  List<UserInfo> get providerData => [];
}

/// Resultado do login sem credencial Microsoft: ela vem da outra porta.
class _Credential extends Fake implements UserCredential {
  _Credential(this.user);

  @override
  final User? user;
}

/// Registra login e saída; demais operações inesperadas falham pelo Fake.
class _Auth extends Fake implements FirebaseAuth {
  User? user;
  Map<String, String> parameters = {};
  int signInCalls = 0;
  int signOutCalls = 0;
  void Function()? onSignIn;
  void Function()? onSignOut;

  @override
  User? get currentUser => user;

  @override
  Future<UserCredential> signInWithProvider(AuthProvider provider) async {
    signInCalls++;

    parameters = (provider as MicrosoftAuthProvider).parameters;

    onSignIn?.call();

    user = _User('a');

    return _Credential(user);
  }

  @override
  Future<void> signOut() async {
    signOutCalls++;
    onSignOut?.call();

    user = null;
  }
}

/// Controla falhas e mudanças de conta nas etapas de obtenção e limpeza.
class _Tokens implements MicrosoftTokenProvider {
  Future<String> Function()? silent;
  Future<String> Function()? onInteractive;
  Future<void> Function()? clear;
  int silentCalls = 0;
  int interactiveCalls = 0;
  String? loginHint;

  @override
  Future<String> getAccessTokenSilently() async {
    silentCalls++;

    return silent == null ? 'graph-silent' : await silent!();
  }

  @override
  Future<String> getAccessTokenInteractively({String? loginHint}) async {
    interactiveCalls++;

    this.loginHint = loginHint;

    return onInteractive == null ? 'graph-interactive' : await onInteractive!();
  }

  @override
  Future<void> clearSession() async {
    await clear?.call();
  }
}
