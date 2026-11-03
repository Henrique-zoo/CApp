/// Implementações controladas dos contratos de identidade para testes.
///
/// Não inicializam Firebase, plugins ou conexões. A fila assíncrona e os
/// Futures controláveis permitem exercitar restauração, falhas e troca de conta.
library;

import 'dart:async';

import 'package:capp/src/modules/identity/application/institutional_auth_service.dart';
import 'package:capp/src/modules/identity/application/ports/auth_session.dart';
import 'package:capp/src/modules/identity/application/ports/institutional_identity_gateway.dart';
import 'package:capp/src/modules/identity/application/ports/microsoft_sign_in.dart';
import 'package:capp/src/modules/identity/application/ports/session_store.dart';
import 'package:capp/src/modules/identity/identity.dart';

/// Produz dados fixos, independentes do relógio do computador.
///
/// A validade padrão é uma hora após o instante inicial do harness.
InstitutionalVerification testVerification({
  DateTime? expiresAt,
  bool student = false,
}) => InstitutionalVerification(
  profile: InstitutionalProfile(
    microsoftObjectId: '00000000-0000-0000-0000-000000000001',
    name: 'Pessoa Exemplo',
    email: null,
    affiliation: student
        ? const StudentAffiliation(courseName: null)
        : const NonStudentAffiliation(),
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

  /// Quantidade de logins iniciais.
  int signInCalls = 0;

  /// Quantidade de reautenticações.
  int reauthenticationCalls = 0;

  /// Substitui a interação inicial, inclusive seus eventos e falhas.
  Future<String> Function()? onSignIn;

  /// Substitui a reautenticação, incluindo troca de conta durante a interação.
  Future<String> Function()? onReauthenticate;

  /// Estabelece a conta de teste antes de retornar uma credencial fictícia.
  @override
  Future<String> signInAndGetAccessToken() async {
    signInCalls++;
    if (onSignIn != null) return onSignIn!();
    auth.setUserId('user-a');
    return 'microsoft-test-token';
  }

  /// Mantém a identidade e retorna outra credencial fictícia.
  @override
  Future<String> reauthenticateAndGetAccessToken() async {
    reauthenticationCalls++;
    if (onReauthenticate != null) return onReauthenticate!();
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

  /// Registra a chamada e entrega o resultado controlado.
  @override
  Future<InstitutionalVerification> verify({
    required String microsoftAccessToken,
  }) async {
    calls++;
    lastMicrosoftAccessToken = microsoftAccessToken;
    return onVerify == null ? verification : onVerify!();
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
    );
  }

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
