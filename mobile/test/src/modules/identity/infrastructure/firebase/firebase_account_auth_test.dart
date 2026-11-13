/// Contratos de isolamento e proteção das sessões Firebase por conta.
///
/// Exercita o adaptador real com SDKs controlados e o índice persistente real.
/// Recriar o adaptador simula o reinício do processo sem perder a persistência SDK.
/// Não substitui o teste OAuth/MSAL em dispositivo Android.
library;

import 'dart:async';

import 'package:capp/src/modules/identity/application/errors/account_password_required_exception.dart';
import 'package:capp/src/modules/identity/application/errors/identity_exception.dart';
import 'package:capp/src/modules/identity/application/errors/session_identity_changed_exception.dart';
import 'package:capp/src/modules/identity/application/models/saved_account.dart';
import 'package:capp/src/modules/identity/application/ports/microsoft_token_provider.dart';
import 'package:capp/src/modules/identity/infrastructure/firebase/firebase_account_auth.dart';
import 'package:capp/src/modules/identity/infrastructure/preferences/account_registry.dart';
import 'package:firebase_auth/firebase_auth.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../../../support/preferences_fake.dart';

void main() {
  late FakePreferences preferences;
  late AccountRegistry registry;
  late Map<String, _Auth> apps;
  late _Tokens tokens;
  late Map<String, _User> users;
  Future<void>? beforePassword;

  Future<FirebaseAccountAuth> create() => FirebaseAccountAuth.create(
    registry: registry,
    factory: (name) async {
      final auth = apps.putIfAbsent(name, () => _Auth(name, users));

      auth.beforePassword = beforePassword;

      return auth;
    },
    tokensFor: (_, {microsoftObjectId}) {
      tokens.objectId = microsoftObjectId;

      return tokens;
    },
  );

  setUp(() {
    preferences = FakePreferences();
    registry = AccountRegistry(preferences);
    apps = {};
    tokens = _Tokens();
    beforePassword = null;
    users = {
      'a@test.example': _User('a', 'a@test.example'),
      'b@test.example': _User('b', 'b@test.example'),
    };
  });

  tearDown(() async {
    for (final app in apps.values) {
      await app.events.close();
    }

    preferences.dispose();
  });

  Future<void> login(FirebaseAccountAuth auth, String account) =>
      auth.signInWithPassword(
        email: '$account@test.example',
        password: 'valid-password',
      );

  test(
    'quota Firebase não encerra credenciais nem passa a exigir senha',
    () async {
      final auth = await create();

      addTearDown(auth.dispose);
      await login(auth, 'a');
      await auth.signOut();

      users['a@test.example']!.tokenFailure = FirebaseAuthException(
        code: 'quota-exceeded',
      );

      await expectLater(
        auth.selectAccount('a'),
        throwsA(
          isA<IdentityException>().having(
            (e) => e.kind,
            'categoria',
            IdentityFailureKind.serviceUnavailable,
          ),
        ),
      );

      final reference = (await registry.read())!.accounts.single;

      expect(reference.requiresPassword, isFalse);
      expect(apps[reference.authAppName]!.currentUser?.uid, 'a');

      users['a@test.example']!.tokenFailure = null;

      await auth.selectAccount('a');

      expect(auth.currentUserId, 'a');
    },
  );

  test(
    'saída comum conserva credenciais e seleção não pede Microsoft',
    () async {
      final auth = await create();

      addTearDown(auth.dispose);
      await login(auth, 'a');

      final account = (await registry.read())!.accounts.single;

      await auth.leaveAccount(AccountSignOutMode.keepSession);

      expect(auth.currentUserId, isNull);
      expect(apps[account.authAppName]!.currentUser?.uid, 'a');

      await auth.selectAccount('a');

      expect(auth.currentUserId, 'a');
      expect(tokens.silentCalls, 0);
      expect(tokens.clearCalls, 0);
      expect(apps[account.authAppName]!.passwordCalls, 1);
    },
  );

  test(
    'reinício após sair não escolhe automaticamente uma conta salva',
    () async {
      var auth = await create();

      await login(auth, 'a');
      await auth.signOut();
      await auth.dispose();

      auth = await create();

      addTearDown(auth.dispose);

      expect(await auth.watchUserId().first, isNull);
      expect(await auth.savedAccounts(), hasLength(1));

      await auth.selectAccount('a');

      expect(auth.currentUserId, 'a');
    },
  );

  test('restaura somente a última conta ativa após reinício', () async {
    var auth = await create();

    await login(auth, 'a');
    await auth.signOut();
    await login(auth, 'b');
    await auth.dispose();

    auth = await create();

    addTearDown(auth.dispose);

    expect(await auth.watchUserId().first, 'b');
    expect(await auth.savedAccounts(), hasLength(2));
  });

  test('exigir senha remove sessão SDK e bloqueia seleção silenciosa após reinício', () async {
    var auth = await create();

    await login(auth, 'a');

    final account = (await registry.read())!.accounts.single;

    await auth.leaveAccount(AccountSignOutMode.requirePassword);

    expect(apps[account.authAppName]!.currentUser, isNull);
    expect(tokens.clearCalls, 1);

    await auth.dispose();

    auth = await create();

    addTearDown(auth.dispose);
    await expectLater(
      auth.selectAccount('a'),
      throwsA(isA<AccountPasswordRequiredException>()),
    );

    expect(auth.currentUserId, isNull);

    await auth.selectAccount('a', password: 'valid-password');

    expect(auth.currentUserId, 'a');
    expect((await auth.savedAccounts()).single.requiresPassword, isFalse);
  });

  test('senha incorreta não ativa conta nem altera a proteção', () async {
    final auth = await create();

    addTearDown(auth.dispose);
    await login(auth, 'a');
    await auth.leaveAccount(AccountSignOutMode.requirePassword);
    await expectLater(
      auth.selectAccount('a', password: 'wrong'),
      throwsException,
    );

    expect(auth.currentUserId, isNull);
    expect((await auth.savedAccounts()).single.requiresPassword, isTrue);
  });

  test('esquecer uma conta não encerra ou remove as outras', () async {
    final auth = await create();

    addTearDown(auth.dispose);
    await login(auth, 'a');

    final a = (await registry.read())!.accounts.single;

    await auth.signOut();
    await login(auth, 'b');

    final b = (await registry.read())!.accounts.last;

    await auth.leaveAccount(AccountSignOutMode.forgetAccount);

    expect((await auth.savedAccounts()).map((a) => a.firebaseUid), ['a']);
    expect(apps[b.authAppName]!.currentUser, isNull);
    expect(apps[a.authAppName]!.currentUser?.uid, 'a');

    await auth.selectAccount('a');

    expect(auth.currentUserId, 'a');
  });

  test(
    'sessão SDK perdida exige senha mesmo com referência não protegida',
    () async {
      final auth = await create();

      addTearDown(auth.dispose);
      await login(auth, 'a');

      final a = (await registry.read())!.accounts.single;

      await auth.signOut();
      await apps[a.authAppName]!.signOut();
      await expectLater(
        auth.selectAccount('a'),
        throwsA(isA<AccountPasswordRequiredException>()),
      );

      expect(auth.currentUserId, isNull);
    },
  );

  test('vincula e-mail escolhido e senha ao mesmo UID', () async {
    final legacy = _Auth('[DEFAULT]', users)
      ..user = _User(
        'institutional-uid',
        'institucional@test.example',
        password: false,
      );

    apps['[DEFAULT]'] = legacy;

    final auth = await create();

    addTearDown(auth.dispose);
    await auth.setUpPassword(
      email: 'pessoal@test.example',
      password: 'valid-password',
    );

    expect(auth.currentUserId, 'institutional-uid');
    expect(auth.currentEmail, 'pessoal@test.example');
    expect(auth.hasPassword, isTrue);
    expect(legacy.user!.linkedEmail, 'pessoal@test.example');
    expect(legacy.user!.forcedRefreshes, 1);
    expect((await auth.savedAccounts()).single.email, 'pessoal@test.example');
  });

  test(
    'falha MSAL mantém a saída Firebase e a marca persistida que exige senha',
    () async {
      final auth = await create();

      await login(auth, 'a');

      tokens.failClear = true;

      await expectLater(
        auth.leaveAccount(AccountSignOutMode.requirePassword),
        throwsStateError,
      );

      expect(auth.currentUserId, isNull);
      expect(apps.values.every((app) => app.currentUser == null), isTrue);
      expect((await registry.read())!.accounts.single.requiresPassword, isTrue);

      await auth.dispose();

      final restored = await create();

      addTearDown(restored.dispose);

      expect(restored.currentUserId, isNull);

      await expectLater(
        restored.selectAccount('a'),
        throwsA(isA<AccountPasswordRequiredException>()),
      );
    },
  );

  test('falha ao gravar proteção não executa logout parcial', () async {
    final auth = await create();

    addTearDown(auth.dispose);
    await login(auth, 'a');

    preferences.writeFailure.value = Exception('storage');

    await expectLater(
      auth.leaveAccount(AccountSignOutMode.requirePassword),
      throwsException,
    );

    expect(auth.currentUserId, 'a');
    expect(tokens.clearCalls, 0);
  });

  test('esquece conta protegida diretamente do seletor sem senha', () async {
    final auth = await create();

    addTearDown(auth.dispose);
    await login(auth, 'a');
    await auth.leaveAccount(AccountSignOutMode.requirePassword);
    await auth.forgetSavedAccount('a');

    expect(await auth.savedAccounts(), isEmpty);
    expect(auth.currentUserId, isNull);
  });

  test(
    'sessão Firebase perdida ainda permite limpar a conta Microsoft correta',
    () async {
      final auth = await create();

      addTearDown(auth.dispose);
      await login(auth, 'a');

      final account = (await registry.read())!.accounts.single;

      await auth.signOut();
      await login(auth, 'b');

      final other = (await registry.read())!.accounts.last;

      await auth.signOut();
      await apps[account.authAppName]!.signOut();
      await auth.forgetSavedAccount('a');

      expect(tokens.objectId, account.microsoftObjectId!.uuid);
      expect(apps[other.authAppName]!.currentUser?.uid, 'b');
      expect((await auth.savedAccounts()).single.firebaseUid, 'b');
    },
  );

  test(
    'senha explicitamente informada é validada mesmo com sessão SDK existente',
    () async {
      final auth = await create();

      addTearDown(auth.dispose);
      await login(auth, 'a');

      final saved = (await registry.read())!.accounts.single;

      await auth.signOut();
      await expectLater(
        auth.selectAccount('a', password: 'wrong'),
        throwsException,
      );

      expect(auth.currentUserId, isNull);
      expect(apps[saved.authAppName]!.passwordCalls, 2);
    },
  );

  test(
    'sessão revogada passa a pedir senha e permite autenticação novamente',
    () async {
      final auth = await create();

      addTearDown(auth.dispose);
      await login(auth, 'a');
      await auth.signOut();

      users['a@test.example']!.tokenFailure = FirebaseAuthException(
        code: 'user-token-expired',
      );

      await expectLater(
        auth.selectAccount('a'),
        throwsA(isA<AccountPasswordRequiredException>()),
      );

      expect((await auth.savedAccounts()).single.requiresPassword, isTrue);

      users['a@test.example']!.tokenFailure = null;

      await auth.selectAccount('a', password: 'valid-password');

      expect(auth.currentUserId, 'a');
    },
  );

  test(
    'login concluído após descarte não reabre a seleção nem grava referências',
    () async {
      final gate = Completer<void>();

      beforePassword = gate.future;

      final auth = await create();
      final login = auth.signInWithPassword(
        email: 'a@test.example',
        password: 'valid-password',
      );
      final rejected = expectLater(
        login,
        throwsA(isA<SessionIdentityChangedException>()),
      );

      await Future<void>.delayed(Duration.zero);
      await auth.dispose();
      gate.complete();
      await rejected;

      expect((await registry.read())!.accounts, isEmpty);
      expect(auth.currentUserId, isNull);
      expect(apps.values.every((app) => app.currentUser == null), isTrue);
    },
  );

  test('índice não contém senha ou tokens', () async {
    final auth = await create();

    addTearDown(auth.dispose);
    await login(auth, 'a');

    final json = preferences.values[AccountRegistry.key] as String;

    expect(json, isNot(contains('valid-password')));
    expect(json, isNot(contains('firebase-token')));
    expect(json, isNot(contains('graph-token')));
  });
}

class _App extends Fake implements FirebaseApp {
  _App(this.name);

  @override
  final String name;
}

class _Info extends Fake implements UserInfo {
  _Info(this.providerId);

  @override
  String? get uid => "00000000-0000-0000-0000-000000000001";

  @override
  final String providerId;
}

class _User extends Fake implements User {
  _User(this.uid, this.address, {this.password = true});

  @override
  final String uid;
  String address;
  bool password;
  String? linkedEmail;
  int forcedRefreshes = 0;
  FirebaseAuthException? tokenFailure;
  _Auth? auth;

  @override
  String get email => address;

  @override
  String? get displayName => 'Conta $uid';

  @override
  List<UserInfo> get providerData => [
    _Info('microsoft.com'),
    if (password) _Info('password'),
  ];

  @override
  Future<String?> getIdToken([bool forceRefresh = false]) async {
    if (tokenFailure != null) throw tokenFailure!;

    if (forceRefresh) forcedRefreshes++;

    return 'firebase-token';
  }

  @override
  Future<UserCredential> linkWithCredential(AuthCredential credential) async {
    final email = credential as EmailAuthCredential;

    linkedEmail = email.email;
    address = email.email;
    password = true;
    auth!.users[email.email] = this;

    return _Credential(this);
  }
}

class _Credential extends Fake implements UserCredential {
  _Credential(this.user);

  @override
  final User user;
}

class _Auth extends Fake implements FirebaseAuth {
  _Auth(String name, this.users) : app = _App(name);

  @override
  final FirebaseApp app;
  final Map<String, _User> users;
  _User? user;
  int passwordCalls = 0;
  Future<void>? beforePassword;
  final events = StreamController<User?>.broadcast();

  @override
  User? get currentUser {
    user?.auth = this;

    return user;
  }

  @override
  Stream<User?> authStateChanges() => Stream<User?>.multi((sink) {
    final sub = events.stream.listen(sink.add);

    sink.add(currentUser);

    sink.onCancel = sub.cancel;
  });

  @override
  Future<void> signOut() async {
    user = null;

    events.add(null);
  }

  @override
  Future<UserCredential> signInWithEmailAndPassword({
    required String email,
    required String password,
  }) async {
    passwordCalls++;

    if (beforePassword != null) {
      await beforePassword;
    }

    if (password != 'valid-password' || users[email] == null) {
      throw FirebaseAuthException(code: 'invalid-credential');
    }

    user = users[email];
    user!.auth = this;

    events.add(user);

    return _Credential(user!);
  }
}

class _Tokens implements MicrosoftTokenProvider {
  String? objectId;
  int silentCalls = 0;
  int clearCalls = 0;
  bool failClear = false;

  @override
  Future<String> getAccessTokenSilently() async {
    silentCalls++;

    return 'graph-token';
  }

  @override
  Future<String> getAccessTokenInteractively({String? loginHint}) async =>
      'graph-token';

  @override
  Future<void> clearSession() async {
    clearCalls++;

    if (failClear) throw StateError('clear');
  }
}
