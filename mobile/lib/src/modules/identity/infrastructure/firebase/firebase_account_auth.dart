/// Sessões Firebase independentes para o seletor de contas do CApp.
///
/// Cada conta usa um FirebaseApp nomeado do mesmo projeto. O SDK conserva seus
/// próprios tokens; preferências guardam somente nome do contêiner, UID, rótulos
/// e a escolha de saída. Desativar a conta atual não encerra as outras sessões.
///
/// A restauração respeita a última escolha ativa, sem selecionar automaticamente
/// uma conta após logout. O índice legado é importado uma vez da instância padrão.
/// Ações são serializadas; a identificação retornada pelo SDK é conferida antes
/// de selecionar uma referência salva. Login com senha e vínculo de credencial
/// usam o mesmo UID, sem criar uma segunda conta Firebase.
///
/// A marca de senha é persistida antes de signOut. Mesmo se o processo morrer
/// durante a saída, selecionar essa referência exigirá senha. Esquecer não exclui
/// a conta remota nem altera sessões de outros dispositivos. A evidência e o
/// perfil institucional são gerenciados pelo controlador e pelo backend.
library;

import 'dart:async';

import 'package:firebase_auth/firebase_auth.dart';
import 'package:uuid/uuid.dart';

import '../../application/errors/identity_exception.dart';
import '../../application/errors/account_password_required_exception.dart';
import '../../application/errors/session_identity_changed_exception.dart';
import '../../application/models/saved_account.dart';
import '../../application/ports/account_authentication.dart';
import '../../application/ports/auth_session.dart';
import '../../application/ports/microsoft_sign_in.dart';
import '../../application/ports/microsoft_token_provider.dart';
import '../preferences/account_registry.dart';
import 'firebase_microsoft_auth.dart';
import 'firebase_auth_failure.dart';
import 'firebase_account_reference.dart';

/// Fábrica injetável de contêineres, compartilhando a configuração do projeto.
typedef AccountAuthFactory = Future<FirebaseAuth> Function(String name);

typedef AccountTokensFactory = MicrosoftTokenProvider Function(
  FirebaseAuth auth, {
  String? microsoftObjectId,
});

final class FirebaseAccountAuth
    implements AuthSession, MicrosoftSignIn, AccountAuthentication {
  FirebaseAccountAuth._(this._registry, this._factory, this._tokensFor);

  final AccountRegistry _registry;
  final AccountAuthFactory _factory;
  final AccountTokensFactory _tokensFor;
  final _events = StreamController<String?>.broadcast();
  List<FirebaseAccountReference> _accounts = [];
  FirebaseAuth? _active;
  StreamSubscription<User?>? _subscription;
  Future<void> _turn = Future.value();
  int _generation = 0;
  bool _disposed = false;
  Future<void>? _disposeFuture;

  /// Restaura a seleção antes de iniciar o controlador da aplicação.
  /// [factory] permite testar sessões sem inicializar plugins nativos.
  static Future<FirebaseAccountAuth> create({
    required AccountRegistry registry,
    required AccountTokensFactory tokensFor,
    required AccountAuthFactory factory,
  }) async {
    final adapter = FirebaseAccountAuth._(registry, factory, tokensFor);
    final stored = await registry.read();

    if (stored == null) {
      final legacy = await adapter._factory('[DEFAULT]');
      final user = legacy.currentUser;

      if (user != null) {
        adapter._accounts = [adapter._reference(legacy, user)];

        await registry.write(adapter._accounts, user.uid);
        adapter._activate(legacy);
      } else {
        await registry.write([], null);
      }
    } else {
      adapter._accounts = stored.accounts;

      final selected = stored.accounts.where(
        (a) => a.firebaseUid == stored.activeUid,
      );

      if (selected.isNotEmpty && !selected.first.requiresPassword) {
        final auth = await adapter._factory(selected.first.authAppName);

        if (auth.currentUser?.uid == selected.first.firebaseUid) {
          adapter._activate(auth);
        } else {
          await registry.write(adapter._accounts, null);
        }
      } else if (stored.activeUid != null) {
        await registry.write(adapter._accounts, null);
      }
    }

    return adapter;
  }

  @override
  String? get currentUserId => _disposed ? null : _active?.currentUser?.uid;

  @override
  String? get currentEmail => _active?.currentUser?.email;

  @override
  bool get hasPassword =>
      _active?.currentUser?.providerData.any(
        (p) => p.providerId == 'password',
      ) ??
      false;

  @override
  Stream<String?> watchUserId() => Stream<String?>.multi((sink) {
    final subscription = _events.stream.listen(
      sink.add,
      onError: sink.addError,
    );

    sink.add(currentUserId);

    sink.onCancel = subscription.cancel;
  }).distinct();

  @override
  Future<List<SavedAccount>> savedAccounts() async {
    _requireOpen();

    return List.unmodifiable(_accounts.map((reference) => reference.account));
  }

  FirebaseMicrosoftAuth _delegate(FirebaseAuth auth) =>
      FirebaseMicrosoftAuth(auth, microsoftTokenProvider: _tokensFor(auth));

  Future<String?> getFirebaseIdToken() async {
    _requireOpen();

    final auth = _active;

    if (auth == null) return null;

    final uid = auth.currentUser?.uid;
    final token = await _delegate(auth).getFirebaseIdToken();

    if (_disposed || _active != auth || auth.currentUser?.uid != uid) {
      throw const SessionIdentityChangedException();
    }

    return token;
  }

  @override
  Future<String> getAccessTokenForCurrentUser({
    bool allowInteraction = true,
  }) async {
    _requireOpen();

    final auth = _active;

    if (auth == null) throw const SessionIdentityChangedException();

    final token = await _delegate(auth)
        .getAccessTokenForCurrentUser(allowInteraction: allowInteraction);

    _requireOpen();

    return token;
  }

  @override
  Future<void> signIn() => _serial(() async {
    final auth = await _newContainer();

    try {
      await _delegate(auth).signIn();
      await _rememberAndActivate(auth, protectExisting: true);
    } on Object {
      if (_active != auth) await auth.signOut();

      rethrow;
    }
  });

  @override
  Future<void> signInWithPassword({
    required String email,
    required String password,
  }) => _serial(() async {
    final auth = await _newContainer();

    try {
      await _firebase(
        () => auth.signInWithEmailAndPassword(
          email: email.trim(),
          password: password,
        ),
      );

      await _rememberAndActivate(auth);
    } on Object {
      if (_active != auth) await auth.signOut();

      rethrow;
    }
  });

  @override
  Future<void> selectAccount(String firebaseUid, {String? password}) =>
      _serial(() async {
        final account = _accounts
            .where((a) => a.firebaseUid == firebaseUid)
            .firstOrNull;

        if (account == null) throw StateError('Conta não salva');

        final auth = await _factory(account.authAppName);

        if (password != null ||
            account.requiresPassword ||
            auth.currentUser?.uid != firebaseUid) {
          if (password == null || account.email == null) {
            throw const AccountPasswordRequiredException();
          }

          await _firebase(
            () => auth.signInWithEmailAndPassword(
              email: account.email!,
              password: password,
            ),
          );
        }

        if (auth.currentUser?.uid != firebaseUid) {
          await auth.signOut();
          throw const SessionIdentityChangedException();
        }

        try {
          await _firebase(() => auth.currentUser!.getIdToken());
        } on IdentityException catch (error) {
          if (error.kind != IdentityFailureKind.credentialsRejected) rethrow;

          final protected = _accounts
              .map(
                (a) => a.firebaseUid != firebaseUid ? a : a.requiringPassword(),
              )
              .toList();

          await _writeRegistry(protected, null);

          _accounts = protected;

          await _firebase(auth.signOut);
          throw const AccountPasswordRequiredException();
        }

        await _rememberAndActivate(auth);
      });

  @override
  Future<void> setUpPassword({
    required String email,
    required String password,
  }) => _serial(() async {
    final auth = _active;
    final user = auth?.currentUser;

    if (auth == null || user == null) {
      throw const SessionIdentityChangedException();
    }

    if (hasPassword &&
        email.trim().toLowerCase() != user.email?.toLowerCase()) {
      throw IdentityException(
        kind: IdentityFailureKind.credentialsRejected,
        cause: const FormatException('E-mail interno divergente'),
        stackTrace: StackTrace.current,
        publicMessage:
            'Use o e-mail da credencial interna já vinculada a esta conta.',
      );
    }

    final credential = EmailAuthProvider.credential(
      email: email.trim(),
      password: password,
    );

    await _firebase(() async {
      if (!hasPassword) await user.linkWithCredential(credential);

      // Confirmar pelo provedor password também fornece o método no próximo JWT.
      final result = await auth.signInWithEmailAndPassword(
        email: email.trim(),
        password: password,
      );

      if (result.user?.uid != user.uid) {
        await auth.signOut();
        throw const SessionIdentityChangedException();
      }

      await result.user!.getIdToken(true);
    });

    await _rememberAndActivate(auth);
  });

  @override
  Future<void> sendPasswordReset(String email) => _serial(() async {
    final auth = _active ?? await _factory('[DEFAULT]');

    await _firebase(() => auth.sendPasswordResetEmail(email: email.trim()));
  });

  @override
  Future<void> signOut() => leaveAccount(AccountSignOutMode.keepSession);

  @override
  Future<void> leaveAccount(AccountSignOutMode mode) => _serial(() async {
    final auth = _active;
    final user = auth?.currentUser;

    if (auth == null || user == null) {
      throw const SessionIdentityChangedException();
    }

    if (mode == AccountSignOutMode.requirePassword && !hasPassword) {
      throw StateError('Configure a senha antes de exigir entrada interna');
    }

    if (mode != AccountSignOutMode.keepSession) {
      final protected = _accounts
          .map(
            (a) => a.firebaseUid == user.uid
                ? a.requiringPassword(email: user.email, name: user.displayName)
                : a,
          )
          .toList();

      await _writeRegistry(protected, user.uid);

      _accounts = protected;

      final saved = _accounts
          .where((a) => a.firebaseUid == user.uid)
          .firstOrNull;
      final tokens = _tokensFor(
        auth,
        microsoftObjectId: saved?.microsoftObjectId?.uuid,
      );

      try {
        // A proteção local não depende do sucesso da limpeza Microsoft.
        await _firebase(auth.signOut);
        await tokens.clearSession();

        final remaining = mode == AccountSignOutMode.forgetAccount
            ? _accounts.where((a) => a.firebaseUid != user.uid).toList()
            : _accounts;

        await _writeRegistry(remaining, null);

        _accounts = remaining;
      } finally {
        if (auth.currentUser == null) await _deactivate();
      }

      return;
    }

    await _writeRegistry(_accounts, null);
    await _deactivate();
  });

  Future<void> _deactivate() async {
    await _subscription?.cancel();

    _subscription = null;
    _active = null;

    ++_generation;

    if (!_disposed) _events.add(null);
  }

  FirebaseAccountReference _reference(FirebaseAuth auth, User user) =>
      FirebaseAccountReference(
        account: SavedAccount(
          firebaseUid: user.uid,
          requiresPassword: false,
          email: user.email,
          name: user.displayName,
        ),
        authAppName: auth.app.name,
        microsoftObjectId: _microsoftObjectId(user),
      );

  UuidValue? _microsoftObjectId(User user) {
    final provider = user.providerData
        .where((p) => p.providerId == 'microsoft.com')
        .firstOrNull;

    if (provider == null) return null;

    try {
      final value = provider.uid;

      if (value == null) {
        throw const FormatException('Objeto Microsoft ausente');
      }

      return UuidValue.withFormatValidation(value);
    } on FormatException catch (error, stack) {
      Error.throwWithStackTrace(
        IdentityException(
          kind: IdentityFailureKind.invalidResponse,
          cause: error,
          stackTrace: stack,
        ),
        stack,
      );
    }
  }

  @override
  Future<void> forgetSavedAccount(String firebaseUid) => _serial(() async {
    if (currentUserId != null) {
      throw StateError('Desative a seleção antes de esquecer outra conta');
    }

    final account = _accounts
        .where((a) => a.firebaseUid == firebaseUid)
        .firstOrNull;

    if (account == null) return;

    final auth = await _factory(account.authAppName);

    if (auth.currentUser != null && auth.currentUser!.uid != firebaseUid) {
      throw const SessionIdentityChangedException();
    }

    final protected = _accounts
        .map((a) => a.firebaseUid != firebaseUid ? a : a.requiringPassword())
        .toList();

    await _writeRegistry(protected, null);

    _accounts = protected;

    final tokens = _tokensFor(
      auth,
      microsoftObjectId: account.microsoftObjectId?.uuid,
    );

    await _firebase(auth.signOut);
    await tokens.clearSession();

    final remaining = _accounts
        .where((a) => a.firebaseUid != firebaseUid)
        .toList();

    await _writeRegistry(remaining, null);

    _accounts = remaining;
  });

  Future<FirebaseAuth> _newContainer() => _factory('capp-${const Uuid().v4()}');

  Future<void> _rememberAndActivate(
    FirebaseAuth auth, {
    bool protectExisting = false,
  }) async {
    _requireOpen();

    final user = auth.currentUser;

    if (user == null) throw const SessionIdentityChangedException();

    final previous = _accounts
        .where((a) => a.firebaseUid == user.uid)
        .firstOrNull;

    if (protectExisting && previous?.requiresPassword == true) {
      throw const AccountPasswordRequiredException();
    }

    final updated = [
      ..._accounts.where((a) => a.firebaseUid != user.uid),
      _reference(auth, user),
    ];

    // Encerrar a cópia substituída antes de trocar o índice evita credenciais
    // órfãs se a limpeza falhar. Falha de persistência mantém a referência antiga;
    // a autenticação recém-criada é encerrada pelo chamador.
    if (previous != null && previous.authAppName != auth.app.name) {
      final replaced = await _factory(previous.authAppName);

      await replaced.signOut();
    }

    await _writeRegistry(updated, user.uid);

    _accounts = updated;

    _activate(auth);
  }

  void _activate(FirebaseAuth auth) {
    _requireOpen();
    unawaited(_subscription?.cancel());

    _active = auth;

    final generation = ++_generation;
    final uid = auth.currentUser?.uid;

    _subscription = auth.authStateChanges().listen(
      (user) {
        if (generation != _generation || user?.uid == uid) return;

        if (user == null) {
          _events.add(null);
        } else {
          _events.addError(
            const SessionIdentityChangedException(),
            StackTrace.current,
          );
        }
      },
      onError: (Object error, StackTrace stack) {
        _events.addError(
          error is FirebaseAuthException
              ? translateFirebaseAuthFailure(error, stack)
              : error,
          stack,
        );
      },
    );

    _events.add(uid);
  }

  Future<T> _serial<T>(Future<T> Function() operation) async {
    final before = _turn;
    final released = Completer<void>();

    _turn = released.future;

    await before;

    try {
      _requireOpen();

      return await operation();
    } finally {
      released.complete();
    }
  }

  void _requireOpen() {
    if (_disposed) throw const SessionIdentityChangedException();
  }

  Future<void> _writeRegistry(
    List<FirebaseAccountReference> accounts,
    String? activeUid,
  ) async {
    _requireOpen();
    await _registry.write(accounts, activeUid);
    _requireOpen();
  }

  Future<T> _firebase<T>(Future<T> Function() operation) async {
    try {
      return await operation();
    } on FirebaseAuthException catch (error, stack) {
      Error.throwWithStackTrace(
        translateFirebaseAuthFailure(error, stack),
        stack,
      );
    }
  }

  /// Cancela apenas a observação; sair do aplicativo não encerra contas salvas.
  Future<void> dispose() => _disposeFuture ??= _disposeResources();

  Future<void> _disposeResources() async {
    _disposed = true;

    ++_generation;

    try {
      await _subscription?.cancel();
    } finally {
      await _events.close();
    }
  }
}
