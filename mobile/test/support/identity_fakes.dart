/// Implementações controladas dos contratos de identidade para testes.
///
/// Não inicializam Firebase, plugins ou conexões. A fila assíncrona e os
/// Futures controláveis permitem exercitar restauração, falhas e troca de conta.
/// FakeMicrosoftSignIn registra separadamente login e obtenção de tokens da
/// conta atual, inclusive a permissão de interação. Assim, os testes detectam
/// renovação automática que abra autorização ou repita operações indevidamente.
library;

import 'dart:async';

import 'package:capp/src/modules/identity/application/institutional_auth_service.dart';
import 'package:capp/src/modules/identity/application/errors/microsoft_interaction_required_exception.dart';
import 'package:capp/src/modules/identity/application/ports/auth_session.dart';
import 'package:capp/src/modules/identity/application/ports/institutional_identity_gateway.dart';
import 'package:capp/src/modules/identity/application/ports/microsoft_sign_in.dart';
import 'package:capp/src/modules/identity/application/ports/session_store.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:uuid/uuid.dart';
import 'package:capp/src/modules/identity/application/models/internal_user.dart';
import 'package:capp/src/modules/identity/application/ports/account_authentication.dart';
import 'package:capp/src/modules/identity/application/ports/account_gateway.dart';
import 'package:capp/src/modules/identity/application/ports/account_profile_store.dart';

/// Produz dados fixos, independentes do relógio do computador.
///
/// A validade padrão é uma hora após o instante inicial do harness.
/// Os UUIDs da pessoa e da instituição são distintos. [institutionAcronym]
/// permite exercitar o cadastro com e sem sigla.
InstitutionalVerification testVerification({
  DateTime? expiresAt,
  bool student = false,
  String? institutionAcronym = 'IEX',
  String? email,
  String? name = 'Pessoa Exemplo',
  String? courseName,
}) => InstitutionalVerification(
  profile: InstitutionalProfile(
    microsoftObjectId: const UuidValue.raw(
      '00000000-0000-0000-0000-000000000001',
    ),
    name: name,
    email: email,
    affiliation: student
        ? StudentAffiliation(courseName: courseName)
        : const NonStudentAffiliation(),
  ),
  institution: InstitutionSummary(
    id: const UuidValue.raw('00000000-0000-0000-0000-000000000002'),
    name: 'Instituição Exemplo',
    acronym: institutionAcronym,
  ),
  expiresAt: expiresAt ?? DateTime.utc(2026, 1, 1, 13),
);

/// Autenticação cujo UID consultado e eventos podem ser controlados separados.
final class FakeAuthSession implements AuthSession {
  /// Define a identidade que será informada na primeira observação.
  FakeAuthSession(this._uid);

  String? _uid;
  final _changes = StreamController<String?>.broadcast(sync: true);

  /// Quantidade de solicitações explícitas de logout ao provedor.
  int signOutCalls = 0;

  /// Substitui a saída para simular demora, falha ou eventos atrasados.
  Future<void> Function()? onSignOut;

  /// Identidade consultada sincronamente pela operação.
  @override
  String? get currentUserId => _uid;

  /// Publica o estado inicial e encaminha eventos e falhas posteriores.
  @override
  Stream<String?> watchUserId() => Stream<String?>.multi((events) {
    final subscription = _changes.stream.listen(
      events.add,
      onError: events.addError,
      onDone: events.close,
    );

    events.add(_uid);

    events.onCancel = subscription.cancel;
  });

  /// Altera o UID; [notify] permite simular um evento ainda não entregue.
  void setUserId(String? uid, {bool notify = true}) {
    _uid = uid;

    if (notify) _changes.add(uid);
  }

  /// Publica uma falha de observação com sua pilha.
  void fail(Exception error, StackTrace stack) =>
      _changes.addError(error, stack);

  /// Encerra a identidade simulada e publica ausência de conta.
  @override
  Future<void> signOut() async {
    signOutCalls++;

    if (onSignOut != null) {
      await onSignOut!();

      return;
    }

    setUserId(null);
  }

  /// Libera o emissor pertencente ao teste.
  Future<void> close() => _changes.close();
}

/// Microsoft simulado que registra qual caminho foi solicitado.
final class FakeMicrosoftSignIn implements MicrosoftSignIn {
  /// Compartilha a autenticação utilizada pelo controlador.
  FakeMicrosoftSignIn(this.auth);

  /// Autenticação atualizada pelo login simulado.
  final FakeAuthSession auth;

  /// Permissões de interação nas solicitações de token da conta atual.
  final interactionPermissions = <bool>[];

  /// Quantidade de logins iniciais.
  int signInCalls = 0;

  /// Quantidade de solicitações de token para a conta já autenticada.
  int currentUserTokenCalls = 0;

  /// Simula uma credencial que precisa de autorização antes do uso silencioso.
  /// Uma chamada com interação permitida resolve essa necessidade no fake.
  bool interactionRequired = false;

  /// Substitui a interação inicial, inclusive seus eventos e falhas.
  Future<void> Function()? onSignIn;

  /// Substitui a obtenção da credencial, incluindo falha ou troca de conta.
  Future<String> Function()? onGetAccessTokenForCurrentUser;

  /// Estabelece a conta de teste sem obter credenciais para o Graph.
  @override
  Future<void> signIn() async {
    signInCalls++;

    if (onSignIn != null) return onSignIn!();

    auth.setUserId('user-a');
  }

  /// Mantém a identidade e retorna outra credencial fictícia.
  @override
  Future<String> getAccessTokenForCurrentUser({
    bool allowInteraction = true,
  }) async {
    currentUserTokenCalls++;
    interactionPermissions.add(allowInteraction);

    if (onGetAccessTokenForCurrentUser != null) {
      return onGetAccessTokenForCurrentUser!();
    }

    if (interactionRequired && !allowInteraction) {
      throw MicrosoftInteractionRequiredException(
        cause: Exception('Autorização pendente no provedor de teste'),
        stackTrace: StackTrace.current,
      );
    }

    interactionRequired = false;

    return 'microsoft-test-token';
  }
}

/// Gateway sem rede, com resposta ou falha controlada pelo teste.
final class FakeIdentityGateway implements InstitutionalIdentityGateway {
  /// Usa a verificação fixa do teste como resposta padrão.
  FakeIdentityGateway(this.verification);

  /// Resultado padrão da operação.
  final InstitutionalVerification verification;

  /// Quantidade de verificações solicitadas.
  int calls = 0;

  /// Última credencial fictícia entregue ao contrato de verificação.
  String? lastMicrosoftAccessToken;

  /// Permite atrasar ou rejeitar uma verificação.
  Future<InstitutionalVerification> Function()? onVerify;

  /// Ledger do backend simulado, separado do cache mobile e associado ao UID.
  String? Function()? resolveUid;
  final verifiedByUid = <String, InstitutionalVerification>{};

  InternalUser Function()? userProvider;

  /// Registra a chamada e entrega o resultado controlado.
  @override
  Future<AccountSession> verify({required String microsoftAccessToken}) async {
    calls++;

    lastMicrosoftAccessToken = microsoftAccessToken;

    final owner = resolveUid?.call();
    final result = onVerify == null ? verification : await onVerify!();

    if (owner != null) verifiedByUid[owner] = result;

    return AccountSession(
      user: userProvider?.call() ?? testInternalUser(),
      verification: result,
    );
  }
}

/// Armazenamento em memória com operações de I/O controláveis.
final class FakeSessionStore implements SessionStore {
  /// Inicia sem registros.
  FakeSessionStore();

  /// Resultados associados aos UIDs.
  final results = <String, SessionCacheLoadResult>{};

  /// Identidades efetivamente solicitadas pelo controlador.
  final loadedUids = <String>[];

  /// Quantidade de tentativas de gravação.
  int saveCalls = 0;

  /// Registros cuja remoção foi solicitada, inclusive tentativas malsucedidas.
  final removedUids = <String>[];

  /// Substitui a leitura para simular I/O pendente.
  Future<SessionCacheLoadResult> Function(String uid)? onLoad;

  /// Substitui a etapa de gravação antes de atualizar o registro.
  Future<void> Function(String uid, InstitutionalVerification value)? onSave;

  /// Substitui a remoção antes de alterar o registro em memória.
  Future<void> Function(String uid)? onRemove;

  /// Registra o UID e mantém distintos os resultados da leitura.
  @override
  Future<SessionCacheLoadResult> load(String firebaseUid) async {
    loadedUids.add(firebaseUid);

    if (onLoad != null) return onLoad!(firebaseUid);

    return results[firebaseUid] ?? const SessionCacheMissing();
  }

  /// Aguarda a ação injetada; falhas propagam sem alterar o registro.
  @override
  Future<void> save({
    required String firebaseUid,
    required InstitutionalVerification verification,
  }) async {
    saveCalls++;
    await onSave?.call(firebaseUid, verification);

    results[firebaseUid] = SessionCacheFound(verification);
  }

  /// Remove somente o registro solicitado.
  @override
  Future<void> remove(String firebaseUid) async {
    removedUids.add(firebaseUid);
    await onRemove?.call(firebaseUid);
    results.remove(firebaseUid);
  }
}

/// Composição real da aplicação ligada a contratos simulados.
final class IdentityHarness {
  /// Recebe a conta inicial e, opcionalmente, um armazenamento controlado.
  IdentityHarness({String? uid, FakeSessionStore? sessionStore})
    : auth = FakeAuthSession(uid),
      store = sessionStore ?? FakeSessionStore() {
    microsoft = FakeMicrosoftSignIn(auth);
    accounts = FakeAccountAuthentication(auth);
    gateway.resolveUid = () => auth.currentUserId;
    users.defaultUser = testInternalUser();
    gateway.userProvider = () => accountGateway.user;
    accountGateway.onRestore = () async {
      final local = store.results[auth.currentUserId];

      return AccountSession(
        user: testInternalUser(),
        verification:
            gateway.verifiedByUid[auth.currentUserId] ??
            (local is SessionCacheFound && local.verification.isValidAt(instant)
                ? local.verification
                : null),
      );
    };
    service = InstitutionalAuthService(
      identityGateway: gateway,
      microsoftSignIn: microsoft,
      authSession: auth,
    );
    controller = SessionController(
      authSession: auth,
      sessionStore: store,
      authService: service,
      now: () => instant,
      accountAuthentication: accounts,
      accountGateway: accountGateway,
      accountProfileStore: users,
    );
  }

  /// Contratos internos compartilham a identidade e simulam um perfil já completo.
  late final FakeAccountAuthentication accounts;
  final users = FakeAccountProfileStore();
  final accountGateway = FakeAccountGateway(testInternalUser(), null);

  /// Relógio mutável para reavaliação determinística.
  DateTime instant = DateTime.utc(2026, 1, 1, 12);

  /// Identidade simulada.
  final FakeAuthSession auth;

  /// Cache simulado.
  final FakeSessionStore store;

  /// Backend controlado sem transporte.
  final gateway = FakeIdentityGateway(testVerification());

  /// Interação Microsoft simulada.
  late final FakeMicrosoftSignIn microsoft;

  /// Serviço real compartilhado pelos testes de aplicação.
  late final InstitutionalAuthService service;

  /// Controlador utilizado sem substituição de sua lógica.
  late final SessionController controller;

  /// Encerra a coordenação antes do emissor de autenticação.
  Future<void> dispose() async {
    await controller.dispose();
    await auth.close();
  }
}

/// Aguarda as microtarefas em testes comuns; widgets utilizam tester.pump.
Future<void> flushSessionEvents() => Future<void>.delayed(Duration.zero);

InternalUser testInternalUser({bool complete = true}) => InternalUser(
  id: const UuidValue.raw('00000000-0000-0000-0000-000000000010'),
  status: complete ? InternalUserStatus.active : InternalUserStatus.incomplete,
  nickname: complete ? 'Apelido' : null,
  email: complete ? 'pessoal@example.test' : null,
  currentSemester: complete ? 4 : null,
  institutionEntrySemester: complete ? '2023.2' : null,
);

class FakeAccountAuthentication implements AccountAuthentication {
  FakeAccountAuthentication(this.auth);

  final FakeAuthSession auth;
  @override
  bool hasPassword = true;

  @override
  String get currentEmail => 'pessoal@example.test';

  String? linkEmail;
  AccountSignOutMode? mode;

  @override
  Future<void> setUpPassword({
    required String email,
    required String password,
  }) async {
    linkEmail = email;
    hasPassword = true;
  }

  List<SavedAccount> accounts = [];
  bool requirePassword = false;
  String? selectedUid;
  int passwordCalls = 0;

  /// Suspende a limpeza depois do evento de logout e antes de atualizar o índice.
  /// Simula a persistência e a limpeza MSAL concluindo após a montagem da entrada.
  Future<void>? beforeSignOutCompletes;

  @override
  Future<List<SavedAccount>> savedAccounts() async => accounts;

  @override
  Future<void> selectAccount(String uid, {String? password}) async {
    selectedUid = uid;

    if (requirePassword && password == null) {
      throw const AccountPasswordRequiredException();
    }

    auth.setUserId(uid);
  }

  @override
  Future<void> signInWithPassword({
    required String email,
    required String password,
  }) async {
    passwordCalls++;
    auth.setUserId('a');
  }

  @override
  Future<void> leaveAccount(AccountSignOutMode mode) async {
    this.mode = mode;

    final uid = auth.currentUserId;

    await auth.signOut();
    await beforeSignOutCompletes;

    if (mode == AccountSignOutMode.forgetAccount) {
      accounts = accounts.where((a) => a.firebaseUid != uid).toList();
    }
  }

  @override
  Future<void> forgetSavedAccount(String uid) async {
    accounts.removeWhere((a) => a.firebaseUid == uid);
  }

  @override
  Future<void> sendPasswordReset(String email) async {}
}

class FakeAccountGateway implements AccountGateway {
  FakeAccountGateway(this.user, this.proof);

  InternalUser user;
  InstitutionalVerification? proof;
  Exception? restoreFailure;
  Exception? completeFailure;
  int restoreCalls = 0;
  Future<AccountSession> Function()? onRestore;
  InstitutionalVerification? Function()? proofProvider;
  ProfileCompletion? completedProfile;
  Future<InternalUser> Function()? onComplete;

  @override
  Future<AccountSession> restoreSession() async {
    restoreCalls++;

    if (onRestore != null) return onRestore!();

    if (restoreFailure != null) throw restoreFailure!;

    return AccountSession(
      user: user,
      verification: proofProvider?.call() ?? proof,
    );
  }

  @override
  Future<InternalUser> completeProfile(ProfileCompletion profile) async {
    completedProfile = profile;

    if (completeFailure != null) throw completeFailure!;

    return onComplete != null
        ? onComplete!()
        : user = InternalUser(
            id: user.id,
            status: InternalUserStatus.active,
            nickname: profile.nickname,
            email: 'pessoal@example.test',
            currentSemester: profile.currentSemester,
            institutionEntrySemester: profile.institutionEntrySemester,
            name: profile.name,
            institutionalEmail: profile.institutionalEmail,
            courseName: profile.courseName,
          );
  }
}

class FakeAccountProfileStore implements AccountProfileStore {
  final values = <String, InternalUser>{};
  InternalUser? defaultUser;

  /// Suspende ou faz falhar a remoção antes de alterar o perfil em memória.
  Future<void> Function(String uid)? onRemove;

  @override
  Future<InternalUser?> loadUser(String uid) async =>
      values[uid] ?? defaultUser;

  @override
  Future<void> saveUser(String uid, InternalUser user) async {
    values[uid] = user;
  }

  @override
  Future<void> removeUser(String uid) async {
    await onRemove?.call(uid);
    values.remove(uid);
  }
}
