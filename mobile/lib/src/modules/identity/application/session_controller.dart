/// Coordenação da sessão, provisionamento e seleção de contas do CApp.
///
/// Uma sessão Firebase, um perfil interno ativo e uma confirmação institucional
/// são fatos distintos. Esta biblioteca coordena suas portas sem conhecer SDKs,
/// DTOs, HTTP, preferências ou widgets. SessionGate apresenta o destino calculado;
/// SessionLifecycleObserver informa retomadas do processo.
///
/// ## Restauração e entrada
///
/// start aguarda o UID selecionado pelo adaptador. Sem seleção, publica a entrada
/// com contas salvas. Uma seleção só autentica se houver sessão SDK utilizável ou
/// senha aceita; a referência local, por si só, não autentica.
///
/// Com conta, consulta o backend por AccountGateway, sem Microsoft. Perfil e
/// evidência vigentes substituem as fotografias locais. Em falha temporária,
/// fotografias válidas podem apresentar a mesma conta autenticada. Recusa de
/// credencial, conta inativa e resposta incompatível não são substituídas por
/// uma permissão do cache. O backend autoriza suas operações independentemente.
///
/// Perfil incompleto ou ausência da credencial interna seleciona profileRequired.
/// completeProfile vincula e confirma a senha no mesmo UID, envia os dados
/// acadêmicos e só publica prontidão após o backend aceitar a ativação. A senha
/// é transitória e não participa de estados ou persistência.
///
/// A opção Microsoft autentica no Firebase antes de consultar AccountGateway.
/// Uma conta com evidência vigente é reutilizada, inclusive se estiver incompleta.
/// Somente conta ausente ou evidência sem validade exige credencial Graph para
/// confrontar as identidades e provisionar ou sincronizar a conta. O prazo de
/// 120 dias é da evidência, independente dos tokens Firebase e Microsoft.
///
/// ## Sincronização e tentativas controladas
///
/// Evidência ausente, expirada ou a até cinco minutos do vencimento provoca uma
/// tentativa silenciosa. Enquanto vigente, permite continuar com perfil ativo;
/// vencida, exige confirmação antes de voltar à área principal. Somente uma
/// necessidade explícita de interação permite que uma ação do usuário abra
/// Microsoft. Uma falha de rede não abre a autenticação.
///
/// Falhas temporárias aguardam 30 segundos, 1, 2, 4 e depois 5 minutos. O intervalo
/// começa na conclusão da tentativa; retomadas não antecipam nem duplicam I/O.
/// A restauração da sessão repete a consulta Firebase/backend; a sincronização
/// repete MSAL/backend. Interação, cancelamento e recusas suspendem a repetição.
/// Nenhuma falha prorroga a validade. Exemplo: falha às 12:00 permite tentar às
/// 12:00:30; outra falha às 12:00:32 permite tentar às 12:01:32.
///
/// O timer só funciona durante a execução do processo e verifica prazos longos
/// em intervalos de até um dia, sem consultar Microsoft diariamente. A agenda
/// de tentativas pertence à seleção atual e não é persistida.
///
/// ## Saída e concorrência
///
/// signOut permite conservar sessão para seleção futura, exigir senha ou
/// esquecer a conta. As duas primeiras opções preservam o perfil/verificação;
/// esquecer remove somente os dados daquele UID, depois de encerrar credenciais.
/// A flag de senha e a persistência SDK pertencem ao adaptador de autenticação.
/// Falhas de limpeza permanecem explícitas e não fabricam sucesso.
///
/// Revisões e contextos descartam resultados de outra operação/conta, inclusive
/// em trocas A/B/A. I/O local é serializado para que esquecer não seja seguido
/// por uma escrita antiga que recrie o cache. I/O já iniciado não é revertido.
/// Falhas operacionais preservam causa e pilha, além de uma classificação pública;
/// não se deve imprimir esses diagnósticos, pois podem conter dados sensíveis.
///
/// dispose cancela inscrições e timers, sem executar logout. O proprietário deve
/// aguardar seu Future. A composição fornece portas que compartilham a mesma
/// seleção de identidade; construir o controlador não inicia efeitos externos.
library;

import 'dart:async';

import 'errors/identity_exception.dart';
import 'errors/microsoft_interaction_required_exception.dart';
import 'errors/session_cache_removal_exception.dart';
import 'errors/session_cache_write_exception.dart';
import 'errors/session_identity_changed_exception.dart';
import 'institutional_auth_service.dart';
import 'errors/account_password_required_exception.dart';
import 'models/internal_user.dart';
import 'models/saved_account.dart';
import 'ports/account_authentication.dart';
import 'ports/account_gateway.dart';
import 'ports/account_profile_store.dart';
import 'models/account_session_result.dart';
import 'ports/auth_session.dart';
import 'ports/session_store.dart';
import 'session_cache_result.dart';
import 'session_state.dart';

/// Fonte do instante atual usada na avaliação de validade.
///
/// Deve representar o relógio da aplicação. A injeção permite testes
/// determinísticos; não altera o relógio dos serviços externos.
typedef SessionClock = DateTime Function();

/// Mantém a sessão e publica o destino de apresentação.
///
/// [start] registra a autenticação. Um UID inicia restauração de seu cache;
/// ausência de UID publica [SessionSignedOut], sem consultar registros locais.
///
/// [signIn] coordena Microsoft e backend. Só uma resposta válida permite
/// publicar SessionAccess.ready; estabelecer o UID Firebase não basta.
///
/// Contextos e revisões impedem aplicar conclusões de outra sessão, inclusive
/// numa sequência A/B/A. Operações locais são serializadas, mas I/O já iniciado
/// não é cancelado ou revertido por troca de conta.
///
/// [signOut] aplica a forma de saída escolhida somente à conta selecionada.
/// O proprietário encerra a instância com [dispose], sem executar logout.
/// O backend é a autoridade para provisionamento e estado do perfil.
final class SessionController {
  /// Conecta contratos referentes à mesma autenticação.
  ///
  /// authSession informa identidade e mudanças; sessionStore mantém o cache;
  /// authService executa a verificação. now fornece o instante para a política
  /// local, inicialmente o relógio do sistema.
  ///
  /// Construir não inicia inscrição, timer ou I/O e não transfere a propriedade
  /// dos adaptadores para o controlador.
  SessionController({
    required this._authSession,
    required this._sessionStore,
    required this._authService,
    this._now = DateTime.now,
    required this._accountAuthentication,
    required this._accountGateway,
    required this._accountProfileStore,
  });

  final AccountAuthentication _accountAuthentication;
  final AccountGateway _accountGateway;
  final AccountProfileStore _accountProfileStore;
  final Map<String, InternalUser> _users = {};

  final AuthSession _authSession;
  final SessionStore _sessionStore;
  final InstitutionalAuthService _authService;
  final SessionClock _now;
  static const _automaticRenewalLead = Duration(minutes: 5);
  static const _retryDelays = [
    Duration(seconds: 30),
    Duration(minutes: 1),
    Duration(minutes: 2),
    Duration(minutes: 4),
    Duration(minutes: 5),
  ];

  /// Agenda da conta/evidência atual; nulo permite a primeira tentativa.
  _RenewalSchedule? _renewalSchedule;
  final StreamController<SessionState> _changes =
      StreamController<SessionState>.broadcast();
  StreamSubscription<String?>? _authSubscription;
  Timer? _refreshTimer;
  SessionState _state = const SessionRestoring();
  _EntryOperation? _activeEntry;

  /// Contexto de saída, preservado durante a emissão esperada de UID nulo.
  _SignOutOperation? _activeSignOut;

  /// Fila de disponibilidade do armazenamento; cada falha fica no seu Future.
  Future<void> _storeTurn = Future<void>.value();

  /// Encerramento compartilhado, inclusive se a primeira solicitação falhar.
  Future<void>? _disposeFuture;

  String? _currentUid;
  bool _receivedUser = false;
  bool _started = false;
  bool _disposed = false;

  /// Geração local que distingue identidades e operações sucessivas.
  int _revision = 0;
  bool _sessionRestorePending = false;
  int _sessionRestoreFailures = 0;

  /// Fotografia atual, incluindo o destino local de apresentação.
  SessionState get state => _state;

  /// Transições posteriores à inscrição, sem repetição do estado inicial.
  ///
  /// É broadcast e assíncrono. Use [state] na primeira construção.
  /// Falhas operacionais esperadas são dados do estado, não erros deste stream.
  Stream<SessionState> get states => _changes.stream;

  /// Registra o acompanhamento uma única vez enquanto estiver ativo.
  ///
  /// Não aguarda o primeiro evento ou a leitura do cache. Exceções de inscrição
  /// publicam [SessionRestorationFailed]; erros de programação propagam.
  /// Lança [StateError] depois de iniciado o encerramento.
  void start() {
    if (_disposed) throw StateError('O controlador já foi encerrado');

    if (_started) return;

    _started = true;

    try {
      _authSubscription = _authSession.watchUserId().listen(
        _onUserChanged,
        onError: _onAuthenticationError,
        onDone: _onAuthenticationDone,
      );
    } on Exception catch (error, stackTrace) {
      _started = false;

      _onAuthenticationError(error, stackTrace);
    }
  }

  /// Executa a operação única de entrada.
  ///
  /// Sem conta autenticada, faz login Microsoft e consulta a conta antes de
  /// decidir se precisa de sincronização. Com conta, retoma a restauração
  /// pendente ou renova a confirmação institucional quando necessário.
  /// Na conta existente, só permite autorização interativa após uma falha que
  /// exija participação do usuário ou após o cancelamento dessa autorização.
  /// Falha de rede oferece repetição silenciosa, sem executar novo login Firebase.
  ///
  /// Retorna UID, perfil e verificação após persistir. Falhas externas propagam
  /// e ficam no estado da sessão correspondente. Falha exclusiva de gravação lança
  /// [SessionCacheWriteException] e mantém a entrada permitida em memória.
  ///
  /// Exige restauração concluída, ausência de operação e entrada pendente.
  /// Violações lançam [StateError]; troca de conta pode lançar
  /// [SessionIdentityChangedException].
  Future<AccountSessionResult> signIn() {
    if (_state.access == SessionAccess.ready) {
      throw StateError('A sessão já está pronta para entrar no aplicativo');
    }

    return _runEntry(
      renewal: _state is SessionSignedIn,
      allowInteraction:
          _state is! SessionSignedIn || _state.requiresMicrosoftInteraction,
    );
  }

  /// Obtém outra verificação da conta atual, preservando o perfil anterior.
  ///
  /// Exige autenticação e ociosidade. A tentativa automática usa
  /// [allowInteraction] falso; uma ação explícita pode manter o padrão verdadeiro
  /// para autorizar a conta Microsoft se a obtenção silenciosa não for possível.
  ///
  /// Retorno e falhas seguem [signIn], mas uma sessão já pronta também pode
  /// solicitar esta operação explicitamente.
  /// A validade só muda com uma nova resposta válida do backend. Falhas mantêm
  /// a evidência anterior e seu prazo, mesmo quando esse prazo já venceu.
  Future<AccountSessionResult> renewVerification({
    bool allowInteraction = true,
  }) {
    return _runEntry(renewal: true, allowInteraction: allowInteraction);
  }

  /// Desativa a seleção com a forma de saída escolhida.
  ///
  /// O padrão conserva a sessão SDK e as fotografias para entrada rápida. Exigir
  /// senha encerra credenciais mantendo a referência. Esquecer encerra credenciais
  /// e remove perfil/verificação somente deste UID. A saída permanece ocupada até
  /// concluir a limpeza, inclusive depois do evento de UID nulo.
  ///
  /// Falhas do adaptador propagam e ficam no estado. Falha de remoção do cache
  /// lança SessionCacheRemovalException; troca de contexto descarta a publicação.
  /// Não exclui conta remota nem revoga sessões de outros dispositivos.
  Future<void> signOut({
    AccountSignOutMode mode = AccountSignOutMode.keepSession,
  }) async {
    _requireReadyForOperation(requiresAuthentication: true);

    final previous = _state as SessionSignedIn;
    final operation = _SignOutOperation(
      firebaseUid: previous.firebaseUid,
      revision: ++_revision,
    );

    _activeSignOut = operation;

    _emit(
      SessionSignedIn(
        firebaseUid: previous.firebaseUid,
        verification: previous.verification,
        cacheLoadResult: previous.cacheLoadResult,
        operation: SessionOperation.signingOut,
      ),
    );

    try {
      await _accountAuthentication.leaveAccount(mode);
      _requireSignedOutOperation(operation);

      if (mode == AccountSignOutMode.forgetAccount) {
        await _removeForgottenData(
          operation.firebaseUid,
          ensureCurrent: () => _requireSignedOutOperation(operation),
        );
      } else {
        await _withStore(() async => _requireSignedOutOperation(operation));
      }

      _requireSignedOutOperation(operation);
      _emit(const SessionSignedOut());
    } on Exception catch (error, stackTrace) {
      _publishSignOutFailure(operation, error, stackTrace);
      rethrow;
    } finally {
      if (identical(_activeSignOut, operation)) {
        _activeSignOut = null;

        _scheduleRefresh();
      }
    }
  }

  /// Contas lembradas pelo dispositivo, sem credenciais no resultado.
  Future<List<SavedAccount>> savedAccounts() =>
      _accountAuthentication.savedAccounts();

  bool get hasInternalPassword => _accountAuthentication.hasPassword;

  String? get internalEmail => _accountAuthentication.currentEmail;

  Future<void> selectAccount(String uid, {String? password}) => _enterInternal(
    () => _accountAuthentication.selectAccount(uid, password: password),
  );

  Future<void> signInWithPassword({
    required String email,
    required String password,
  }) => _enterInternal(
    () => _accountAuthentication.signInWithPassword(
      email: email,
      password: password,
    ),
  );

  Future<void> sendPasswordReset(String email) =>
      _accountAuthentication.sendPasswordReset(email);

  Future<void> _enterInternal(Future<void> Function() operation) async {
    _requireReadyForOperation(requiresAuthentication: false);

    if (_authSession.currentUserId != null) {
      throw StateError('Entrada interna indisponível');
    }

    final revision = ++_revision;

    _emit(const SessionSignedOut(operation: SessionOperation.signingIn));

    try {
      await operation();
    } on AccountPasswordRequiredException {
      if (_isSignedOutRevision(revision)) _emit(const SessionSignedOut());

      rethrow;
    } on Exception catch (error, stack) {
      if (_isSignedOutRevision(revision)) {
        _emit(
          SessionSignedOut(
            failure: _classifyFailure(error, _state),
            failureDetails: SessionFailureDetails(
              cause: error,
              stackTrace: stack,
            ),
          ),
        );
      }

      rethrow;
    }
  }

  /// Remove uma conta lembrada sem selecionar ou autenticar sua sessão.
  Future<void> forgetSavedAccount(String uid) async {
    _requireReadyForOperation(requiresAuthentication: false);

    if (_authSession.currentUserId != null) {
      throw StateError('A seleção precisa estar desativada');
    }

    final revision = ++_revision;

    _emit(const SessionSignedOut(operation: SessionOperation.signingOut));

    try {
      await _accountAuthentication.forgetSavedAccount(uid);
      await _removeForgottenData(
        uid,
        ensureCurrent: () => _requireSignedOutRevision(revision),
      );

      if (!_isSignedOutRevision(revision)) {
        throw const SessionIdentityChangedException();
      }

      _emit(const SessionSignedOut());
    } on Exception catch (error, stack) {
      if (_isSignedOutRevision(revision)) {
        _emit(
          SessionSignedOut(
            failure: error is SessionCacheRemovalException
                ? SessionFailure.cacheRemovalFailed
                : SessionFailure.signOutFailed,
            failureDetails: SessionFailureDetails(
              cause: error,
              stackTrace: stack,
            ),
          ),
        );
      }

      rethrow;
    }
  }

  bool _isSignedOutRevision(int revision) =>
      !_disposed && _revision == revision && _authSession.currentUserId == null;

  void _requireSignedOutRevision(int revision) {
    if (!_isSignedOutRevision(revision)) {
      throw const SessionIdentityChangedException();
    }
  }

  /// Remove as duas fotografias sob a mesma fila, conferindo o contexto entre I/Os.
  Future<void> _removeForgottenData(
    String uid, {
    required void Function() ensureCurrent,
  }) async {
    try {
      await _withStore(() async {
        ensureCurrent();
        await _sessionStore.remove(uid);

        ensureCurrent();
        await _accountProfileStore.removeUser(uid);

        ensureCurrent();
        _users.remove(uid);
      });
    } on SessionIdentityChangedException {
      rethrow;
    } on Exception catch (error, stack) {
      Error.throwWithStackTrace(
        SessionCacheRemovalException(
          firebaseUid: uid,
          cause: error,
          causeStackTrace: stack,
        ),
        stack,
      );
    }
  }

  /// Repete somente a limpeza que falhou após encerrar a autenticação.
  Future<void> retryForgottenAccountCleanup() async {
    _requireReadyForOperation(requiresAuthentication: false);

    final cause = _state.failureDetails?.cause;

    if (_authSession.currentUserId != null ||
        cause is! SessionCacheRemovalException) {
      throw StateError('Não existe limpeza pendente');
    }

    final revision = ++_revision;

    _emit(const SessionSignedOut(operation: SessionOperation.signingOut));

    try {
      await _removeForgottenData(
        cause.firebaseUid,
        ensureCurrent: () => _requireSignedOutRevision(revision),
      );

      if (!_isSignedOutRevision(revision)) {
        throw const SessionIdentityChangedException();
      }

      _emit(const SessionSignedOut());
    } on Exception catch (error, stack) {
      if (_isSignedOutRevision(revision)) {
        _emit(
          SessionSignedOut(
            failure: SessionFailure.cacheRemovalFailed,
            failureDetails: SessionFailureDetails(
              cause: error,
              stackTrace: stack,
            ),
          ),
        );
      }

      rethrow;
    }
  }

  /// Repete a consulta da conta sem solicitar token Microsoft.
  Future<void> retryAccountSession() async {
    _requireReadyForOperation(requiresAuthentication: true);

    final uid = _authSession.currentUserId!;
    final previous = _state as SessionSignedIn;
    final revision = ++_revision;

    _emit(
      SessionSignedIn(
        firebaseUid: uid,
        verification: previous.verification,
        operation: SessionOperation.restoringCache,
      ),
    );

    await _restoreCache(firebaseUid: uid, revision: revision);
  }

  /// Vincula a senha ao UID atual e conclui o perfil somente após o backend aceitar.
  Future<void> completeProfile({
    required String email,
    required String password,
    required ProfileCompletion profile,
  }) async {
    _requireReadyForOperation(requiresAuthentication: true);

    final previous = _state as SessionSignedIn;
    final revision = ++_revision;

    if (!previous.hasUnexpiredVerificationAt(_now())) {
      throw StateError('Perfil indisponível');
    }

    _emit(
      SessionSignedIn(
        firebaseUid: previous.firebaseUid,
        verification: previous.verification,
        operation: SessionOperation.completingProfile,
      ),
    );

    try {
      await _accountAuthentication.setUpPassword(
        email: email,
        password: password,
      );

      if (!_isCurrent(previous.firebaseUid, revision)) {
        throw const SessionIdentityChangedException();
      }

      final user = await _accountGateway.completeProfile(profile);

      if (!_isCurrent(previous.firebaseUid, revision)) {
        throw const SessionIdentityChangedException();
      }

      _users[previous.firebaseUid] = user;

      await _withStore(() async {
        if (!_isCurrent(previous.firebaseUid, revision)) {
          throw const SessionIdentityChangedException();
        }

        await _accountProfileStore.saveUser(previous.firebaseUid, user);
      });

      if (!_isCurrent(previous.firebaseUid, revision)) {
        throw const SessionIdentityChangedException();
      }

      _emit(
        SessionSignedIn(
          firebaseUid: previous.firebaseUid,
          verification: previous.verification,
        ),
      );
    } on Exception catch (error, stack) {
      if (_isCurrent(previous.firebaseUid, revision)) {
        _emit(
          SessionSignedIn(
            firebaseUid: previous.firebaseUid,
            verification: previous.verification,
            failure:
                error is IdentityException &&
                    (error.kind == IdentityFailureKind.communicationFailed ||
                        error.kind == IdentityFailureKind.serviceUnavailable)
                ? _classifyFailure(error, previous)
                : SessionFailure.profileCompletionFailed,
            failureDetails: SessionFailureDetails(
              cause: error,
              stackTrace: stack,
            ),
          ),
        );
      }

      rethrow;
    }
  }

  /// Reavalia identidade e validade ao retomar a aplicação.
  ///
  /// Pode iniciar recuperação de evidência ausente ou renovação de evidência
  /// expirada ou próxima do vencimento, sempre sem abrir autorização Microsoft.
  /// Repetições após falhas temporárias respeitam a espera progressiva.
  /// Antes da primeira identidade restaurada ou depois do descarte, não age.
  /// Uma divergência de UID inicia a restauração da conta atual. Retorna sem
  /// aguardar a renovação; atividade, resultado e falhas aparecem em [states].
  void refresh() {
    if (_disposed || !_started || !_receivedUser) return;

    final actualUid = _authSession.currentUserId;

    if (actualUid != _currentUid) {
      _onUserChanged(actualUid);
    } else {
      _emit(_state);
      _maybeRenewAutomatically();
    }
  }

  /// Recupera ou renova a evidência respeitando a agenda da sessão atual.
  ///
  /// Ausência de evidência também permite obtenção silenciosa. Uma operação
  /// pendente, uma falha definitiva ou a espera de nova tentativa impedem I/O.
  void _maybeRenewAutomatically() {
    final current = _state;

    if (_disposed ||
        !_started ||
        !_receivedUser ||
        current is! SessionSignedIn ||
        current.isBusy ||
        _activeEntry != null ||
        _activeSignOut != null ||
        _authSession.currentUserId != current.firebaseUid) {
      return;
    }

    final dueAt = _automaticRenewalAt(current);

    if (dueAt == null || dueAt.isAfter(_now())) return;

    unawaited(_runAutomaticRenewal());
  }

  /// Consome falhas já publicadas sem abrir autorização ou perder diagnóstico.
  Future<void> _runAutomaticRenewal() async {
    try {
      if (_sessionRestorePending) {
        await retryAccountSession();
      } else {
        await renewVerification(allowInteraction: false);
      }
    } on Exception {
      // Falhas da operação ficam no estado. Conclusões de outra sessão
      // são descartadas pelas guardas já existentes no controlador.
    }
  }

  /// Mantém o contexto de entrada ou renovação até concluir a persistência.
  ///
  /// Publica sucesso em memória antes de gravar. Só o próprio contexto é
  /// liberado ao terminar; uma operação antiga não encerra a de outra conta.
  Future<AccountSessionResult> _runEntry({
    required bool renewal,
    bool allowInteraction = true,
  }) async {
    _requireReadyForOperation(requiresAuthentication: renewal);

    final previous = _state;
    final operation = _EntryOperation(
      renewal: renewal,
      firebaseUid: renewal ? _currentUid : null,
      revision: ++_revision,
    );

    _activeEntry = operation;

    if (previous is SessionSignedIn) {
      final schedule = _scheduleFor(previous);

      // Bloqueia repetições até esta operação concluir e definir outra agenda.
      schedule.nextAttemptAt = null;

      _emit(
        SessionSignedIn(
          firebaseUid: previous.firebaseUid,
          verification: previous.verification,
          cacheLoadResult: previous.cacheLoadResult,
          operation: SessionOperation.verifyingInstitution,
        ),
      );
    } else {
      _emit(const SessionSignedOut(operation: SessionOperation.signingIn));
    }

    try {
      AccountSessionResult result;
      late final InternalUser accountUser;

      try {
        if (!renewal) {
          await _authService.signIn();
        }

        result = !renewal || _sessionRestorePending
            ? await _restoreOrVerify(
                operation,
                allowInteraction: allowInteraction,
              )
            : await _authService.renewVerification(
                allowInteraction: allowInteraction,
              );

        _requireActiveOperation(operation, result.firebaseUid);

        accountUser = result.user;
        _users[result.firebaseUid] = accountUser;

        if (!result.verification.isValidAt(_now())) {
          final cause = const FormatException(
            'A verificação recebida já está expirada no relógio local',
          );

          throw IdentityException(
            kind: IdentityFailureKind.invalidResponse,
            cause: cause,
            stackTrace: StackTrace.current,
          );
        }
      } on Exception catch (error, stackTrace) {
        _publishEntryFailure(operation, error, stackTrace);
        rethrow;
      }

      _requireActiveOperation(operation, result.firebaseUid);

      final now = _now();
      final anticipated = result.verification.expiresAt.subtract(
        _automaticRenewalLead,
      );
      final earliest = now.add(_retryDelays.first);

      _renewalSchedule = _RenewalSchedule(
        key: (result.firebaseUid, result.verification.expiresAt),
        nextAttemptAt: anticipated.isAfter(earliest) ? anticipated : earliest,
      );

      _emit(
        SessionSignedIn(
          firebaseUid: result.firebaseUid,
          verification: result.verification,
          operation: SessionOperation.verifyingInstitution,
        ),
      );

      try {
        await _withStore(() async {
          _requireActiveOperation(operation, result.firebaseUid);
          await _accountProfileStore.saveUser(result.firebaseUid, accountUser);
          await _sessionStore.save(
            firebaseUid: result.firebaseUid,
            verification: result.verification,
          );
        });
      } on SessionIdentityChangedException {
        rethrow;
      } on Exception catch (error, stackTrace) {
        final failure = SessionCacheWriteException(
          result: result,
          cause: error,
          causeStackTrace: stackTrace,
        );

        if (_canPublish(operation, result.firebaseUid)) {
          _emit(
            SessionSignedIn(
              firebaseUid: result.firebaseUid,
              verification: result.verification,
              failure: SessionFailure.cacheWriteFailed,
              failureDetails: SessionFailureDetails(
                cause: failure,
                stackTrace: stackTrace,
              ),
            ),
          );
        }

        Error.throwWithStackTrace(failure, stackTrace);
      }

      _requireActiveOperation(operation, result.firebaseUid);
      _emit(
        SessionSignedIn(
          firebaseUid: result.firebaseUid,
          verification: result.verification,
        ),
      );

      return result;
    } finally {
      if (identical(_activeEntry, operation)) {
        _activeEntry = null;

        _scheduleRefresh();
      }
    }
  }

  /// Recupera a conta antes de decidir se precisa de uma consulta institucional.
  ///
  /// Uma evidência válida é reutilizada sem obter token Microsoft nem prolongar
  /// seu prazo. Conta ausente ou evidência vencida exige verificação. Outras
  /// recusas propagam; uma falha de rede mantém a restauração pendente para que
  /// a próxima tentativa repita a consulta da sessão, em vez de chamar o Graph.
  /// As guardas protegem também as etapas entre autenticação e restauração.
  Future<AccountSessionResult> _restoreOrVerify(
    _EntryOperation operation, {
    required bool allowInteraction,
  }) async {
    final firebaseUid = _authSession.currentUserId;

    if (firebaseUid == null) {
      throw const SessionIdentityChangedException();
    }

    _requireActiveOperation(operation, firebaseUid);

    _sessionRestorePending = true;

    AccountSession? restored;

    try {
      restored = await _accountGateway.restoreSession();
    } on IdentityException catch (error) {
      if (error.kind != IdentityFailureKind.accountNotProvisioned) rethrow;
    }

    _requireActiveOperation(operation, firebaseUid);

    _sessionRestorePending = false;

    final proof = restored?.verification;

    if (restored != null && proof != null && proof.isValidAt(_now())) {
      return AccountSessionResult(
        firebaseUid: firebaseUid,
        user: restored.user,
        verification: proof,
      );
    }

    return _authService.renewVerification(allowInteraction: allowInteraction);
  }

  /// Confere restauração, exclusão mútua e identidade antes de uma operação.
  ///
  /// [requiresAuthentication] exige conta conhecida para renovação ou saída.
  /// Divergência de UID é sincronizada antes de sinalizar contexto alterado.
  void _requireReadyForOperation({required bool requiresAuthentication}) {
    if (_disposed) throw StateError('O controlador já foi encerrado');

    if (!_started ||
        !_receivedUser ||
        _state.access == SessionAccess.restorationFailed) {
      throw StateError('A autenticação inicial ainda não foi restaurada');
    }

    if (_activeEntry != null || _activeSignOut != null || _state.isBusy) {
      throw StateError('Já existe uma operação de sessão em andamento');
    }

    final actualUid = _authSession.currentUserId;

    if (actualUid != _currentUid) {
      _onUserChanged(actualUid);
      throw const SessionIdentityChangedException();
    }

    if (requiresAuthentication && _state is! SessionSignedIn) {
      throw StateError('A operação exige uma conta autenticada');
    }
  }

  /// Processa mudanças de identidade e restaura apenas o cache da conta atual.
  ///
  /// A primeira identidade estabelecida durante login é esperada: vincula-se
  /// ao contexto em curso e mantém a entrada pendente até a resposta do backend.
  /// Outras mudanças invalidam a tentativa anterior.
  void _onUserChanged(String? firebaseUid) {
    if (_disposed) return;

    if (_receivedUser && firebaseUid == _currentUid) return;

    final signOut = _activeSignOut;

    if (signOut != null &&
        firebaseUid == null &&
        _currentUid == signOut.firebaseUid &&
        _revision == signOut.revision) {
      _receivedUser = true;
      _currentUid = null;

      _emit(const SessionSignedOut(operation: SessionOperation.signingOut));

      return;
    }

    _receivedUser = true;
    _currentUid = firebaseUid;
    _renewalSchedule = null;
    _sessionRestorePending = false;
    _sessionRestoreFailures = 0;

    final revision = ++_revision;
    final operation = _activeEntry;

    if (operation != null &&
        !operation.renewal &&
        operation.firebaseUid == null &&
        firebaseUid != null) {
      operation.firebaseUid = firebaseUid;
      operation.revision = revision;

      _emit(
        SessionSignedIn(
          firebaseUid: firebaseUid,
          operation: SessionOperation.verifyingInstitution,
        ),
      );

      return;
    }

    _activeEntry = null;
    _activeSignOut = null;

    if (firebaseUid == null) {
      _emit(const SessionSignedOut());

      return;
    }

    _emit(
      SessionSignedIn(
        firebaseUid: firebaseUid,
        operation: SessionOperation.restoringCache,
      ),
    );

    unawaited(_restoreCache(firebaseUid: firebaseUid, revision: revision));
  }

  /// Publica o resultado explícito da leitura somente no contexto que a iniciou.
  ///
  /// Ausência, rejeição e falha de acesso mantêm a autenticação, mas não concedem
  /// entrada. Evidência expirada também é preservada, com acesso pendente.
  /// Falhas inesperadas da implementação não são convertidas em cache ausente.
  Future<void> _restoreCache({
    required String firebaseUid,
    required int revision,
  }) async {
    var result = await _withStore(() => _sessionStore.load(firebaseUid));

    if (!_isCurrent(firebaseUid, revision)) return;

    Exception? restorationError;
    StackTrace? restorationStack;
    bool receivedBackendResponse = false;
    bool cacheWriteFailed = false;

    try {
      final cached = await _accountProfileStore.loadUser(firebaseUid);

      if (!_isCurrent(firebaseUid, revision)) return;

      if (cached != null) _users[firebaseUid] = cached;
    } on Exception {
      /* Cache inválido não impede buscar a fonte de verdade. */
    }

    try {
      final restored = await _accountGateway.restoreSession();

      if (!_isCurrent(firebaseUid, revision)) return;

      receivedBackendResponse = true;
      _users[firebaseUid] = restored.user;

      final proof = restored.verification;
      final local = result;
      final invalidateLocal =
          proof == null &&
          local is SessionCacheFound &&
          local.verification.isValidAt(_now());

      if (proof != null) {
        result = SessionCacheFound(proof);
      } else if (invalidateLocal) {
        result = const SessionCacheMissing();
      }

      await _withStore(() async {
        if (!_isCurrent(firebaseUid, revision)) return;

        await _accountProfileStore.saveUser(firebaseUid, restored.user);

        if (proof != null &&
            (local is! SessionCacheFound || local.verification != proof)) {
          await _sessionStore.save(
            firebaseUid: firebaseUid,
            verification: proof,
          );
        } else if (invalidateLocal) {
          await _sessionStore.remove(firebaseUid);
        }
      });
    } on IdentityException catch (error, stack) {
      if (!_isCurrent(firebaseUid, revision)) return;

      restorationError = error;
      restorationStack = stack;

      if (error.kind == IdentityFailureKind.appVerificationFailed) {
        // A origem do app foi recusada, não o usuário: bloqueia o acesso desta
        // tentativa sem apagar as fotografias da conta para uma recuperação.
        result = const SessionCacheMissing();
      } else if (error.kind != IdentityFailureKind.communicationFailed &&
          error.kind != IdentityFailureKind.serviceUnavailable) {
        _users.remove(firebaseUid);

        result = const SessionCacheMissing();

        try {
          await _withStore(() async {
            if (!_isCurrent(firebaseUid, revision)) return;

            await _sessionStore.remove(firebaseUid);
            await _accountProfileStore.removeUser(firebaseUid);
          });

          if (error.kind == IdentityFailureKind.accountNotProvisioned) {
            // A primeira confirmação ainda não ocorreu. Permite a tentativa
            // silenciosa de provisionamento, sem tratar ausência como recusa.
            restorationError = null;
            restorationStack = null;
          }
        } on Exception catch (cleanup, cleanupStack) {
          restorationError = IdentityException(
            kind: error.kind,
            cause: (
              refusal: error,
              cleanup: cleanup,
              cleanupStack: cleanupStack,
            ),
            stackTrace: stack,
            backendCode: error.backendCode,
            publicMessage: error.publicMessage,
          );
        }
      }
    } on Exception catch (error, stack) {
      restorationError = error;
      restorationStack = stack;
      cacheWriteFailed = receivedBackendResponse;
    }

    if (!_isCurrent(firebaseUid, revision)) return;

    _sessionRestorePending =
        restorationError is IdentityException &&
        (restorationError.kind == IdentityFailureKind.communicationFailed ||
            restorationError.kind == IdentityFailureKind.serviceUnavailable);

    if (restorationError != null && !cacheWriteFailed) {
      final delay = _retryDelays[_sessionRestoreFailures];

      if (_sessionRestorePending &&
          _sessionRestoreFailures < _retryDelays.length - 1) {
        _sessionRestoreFailures++;
      }

      _renewalSchedule = _RenewalSchedule(
        key: (
          firebaseUid,
          result is SessionCacheFound ? result.verification.expiresAt : null,
        ),
        nextAttemptAt: _sessionRestorePending ? _now().add(delay) : null,
      );
    } else {
      _sessionRestoreFailures = 0;
      _renewalSchedule = null;
    }

    _emit(
      SessionSignedIn(
        firebaseUid: firebaseUid,
        verification: switch (result) {
          SessionCacheFound(:final verification) => verification,
          _ => null,
        },
        cacheLoadResult: result,
        failure: cacheWriteFailed
            ? SessionFailure.cacheWriteFailed
            : restorationError != null
            ? _classifyFailure(restorationError, _state)
            : switch (result) {
                SessionCacheRejected() => SessionFailure.cacheRejected,
                SessionCacheReadFailed() => SessionFailure.cacheReadFailed,
                _ => null,
              },
        failureDetails: restorationError == null
            ? null
            : SessionFailureDetails(
                cause: restorationError,
                stackTrace: restorationStack!,
              ),
      ),
    );

    _maybeRenewAutomatically();
  }

  /// Preserva a falha e seu diagnóstico somente na sessão correspondente.
  ///
  /// Uma exceção de contexto antigo continua no Future original, mas não
  /// substitui os dados da conta atual.
  void _publishEntryFailure(
    _EntryOperation operation,
    Exception error,
    StackTrace stackTrace,
  ) {
    if (_disposed || !identical(_activeEntry, operation)) return;

    final actualUid = _authSession.currentUserId;

    if (actualUid != _currentUid) _onUserChanged(actualUid);

    if (!identical(_activeEntry, operation)) return;

    final current = _state;
    final details = SessionFailureDetails(cause: error, stackTrace: stackTrace);
    final failure = _classifyFailure(error, current);

    if (current is SessionSignedIn) {
      final schedule = _scheduleFor(current);

      if (error is IdentityException &&
          (error.kind == IdentityFailureKind.communicationFailed ||
              error.kind == IdentityFailureKind.serviceUnavailable)) {
        final index = schedule.transientFailures;
        final delay = _retryDelays[index];

        if (index < _retryDelays.length - 1) schedule.transientFailures++;

        schedule.nextAttemptAt = _now().add(delay);
      } else {
        // Interação, cancelamento, recusa e falhas não classificadas exigem
        // ação explícita. O diagnóstico não é convertido em falha de rede.
        schedule.nextAttemptAt = null;
      }

      _emit(
        SessionSignedIn(
          firebaseUid: current.firebaseUid,
          verification: current.verification,
          cacheLoadResult: current.cacheLoadResult,
          failure: failure,
          failureDetails: details,
        ),
      );
    } else if (current is SessionSignedOut) {
      _emit(SessionSignedOut(failure: failure, failureDetails: details));
    }
  }

  /// Exige a mesma saída e confirma a ausência efetiva de conta no provedor.
  ///
  /// Sincroniza eventos atrasados pelo UID consultado. Revisões impedem que
  /// uma sequência A/B/A reaproveite conclusões do logout anterior.
  void _requireSignedOutOperation(_SignOutOperation operation) {
    if (_disposed || !identical(_activeSignOut, operation)) {
      throw const SessionIdentityChangedException();
    }

    final actualUid = _authSession.currentUserId;

    if (actualUid != _currentUid) _onUserChanged(actualUid);

    if (!identical(_activeSignOut, operation) ||
        _revision != operation.revision ||
        _currentUid != null ||
        actualUid != null) {
      throw const SessionIdentityChangedException();
    }
  }

  /// Preserva uma falha de saída na identidade efetivamente observada.
  ///
  /// Uma falha do provedor não fabrica logout. Uma falha exclusiva da limpeza
  /// conserva o estado deslogado. Conclusões de contextos antigos são ignoradas.
  void _publishSignOutFailure(
    _SignOutOperation operation,
    Exception error,
    StackTrace stackTrace,
  ) {
    if (_disposed || !identical(_activeSignOut, operation)) return;

    final actualUid = _authSession.currentUserId;

    if (actualUid != _currentUid) _onUserChanged(actualUid);

    if (!identical(_activeSignOut, operation) ||
        _revision != operation.revision) {
      return;
    }

    final failure = error is SessionCacheRemovalException
        ? SessionFailure.cacheRemovalFailed
        : SessionFailure.signOutFailed;
    final details = SessionFailureDetails(cause: error, stackTrace: stackTrace);
    final current = _state;

    if (current is SessionSignedIn) {
      _emit(
        SessionSignedIn(
          firebaseUid: current.firebaseUid,
          verification: current.verification,
          cacheLoadResult: current.cacheLoadResult,
          failure: failure,
          failureDetails: details,
        ),
      );
    } else if (current is SessionSignedOut) {
      _emit(SessionSignedOut(failure: failure, failureDetails: details));
    }
  }

  /// Resume a falha sem interpretar tipos de SDK ou status HTTP.
  ///
  /// A exceção original continua em SessionFailureDetails e no Future público.
  SessionFailure _classifyFailure(Exception error, SessionState current) {
    if (error is MicrosoftInteractionRequiredException) {
      return SessionFailure.microsoftInteractionRequired;
    }

    if (error is IdentityException) {
      return switch (error.kind) {
        IdentityFailureKind.authenticationCancelled =>
          SessionFailure.authenticationCancelled,
        IdentityFailureKind.operationCancelled =>
          SessionFailure.operationCancelled,
        IdentityFailureKind.communicationFailed =>
          SessionFailure.communicationFailed,
        IdentityFailureKind.serviceUnavailable =>
          SessionFailure.serviceUnavailable,
        IdentityFailureKind.credentialsRejected =>
          SessionFailure.authenticationFailed,
        _ => SessionFailure.institutionalVerificationFailed,
      };
    }

    return current is SessionSignedIn
        ? SessionFailure.institutionalVerificationFailed
        : SessionFailure.authenticationFailed;
  }

  /// Sincroniza eventual evento pendente e exige contexto, UID e revisão atuais.
  ///
  /// Divergências invalidam a tentativa e restauram a identidade consultada.
  /// Não desfaz operações externas já iniciadas.
  void _requireActiveOperation(_EntryOperation operation, String firebaseUid) {
    if (_disposed || !identical(_activeEntry, operation)) {
      throw const SessionIdentityChangedException();
    }

    final actualUid = _authSession.currentUserId;

    if (actualUid != _currentUid) _onUserChanged(actualUid);

    if (_canPublish(operation, firebaseUid)) return;

    if (identical(_activeEntry, operation)) {
      _activeEntry = null;
      _receivedUser = false;

      _onUserChanged(actualUid);
    }

    throw const SessionIdentityChangedException();
  }

  /// Confere o contexto e a identidade vinculados a uma publicação.
  bool _canPublish(_EntryOperation operation, String firebaseUid) {
    return !_disposed &&
        identical(_activeEntry, operation) &&
        operation.firebaseUid == firebaseUid &&
        _isCurrent(firebaseUid, operation.revision);
  }

  /// Distingue identidades e revisões, incluindo mudanças A/B/A.
  bool _isCurrent(String firebaseUid, int revision) {
    return !_disposed &&
        _currentUid == firebaseUid &&
        _authSession.currentUserId == firebaseUid &&
        _revision == revision;
  }

  /// Serializa I/O local sem consumir o resultado ou a falha de cada operação.
  ///
  /// O marcador de disponibilidade conclui sempre no finally, permitindo que
  /// uma operação malsucedida libere a fila.
  Future<T> _withStore<T>(Future<T> Function() operation) async {
    final previous = _storeTurn;
    final released = Completer<void>();

    _storeTurn = released.future;

    await previous;

    try {
      return await operation();
    } finally {
      released.complete();
    }
  }

  /// Invalida a sessão observada preservando a exceção de acompanhamento.
  ///
  /// Objetos que não implementam Exception continuam propagando com sua pilha.
  /// Não fabrica logout nem remove autenticação no provedor.
  void _onAuthenticationError(Object error, StackTrace stackTrace) {
    if (_disposed) return;

    if (error is! Exception) Error.throwWithStackTrace(error, stackTrace);

    ++_revision;

    _activeEntry = null;
    _activeSignOut = null;
    _currentUid = null;
    _receivedUser = false;

    _emit(SessionRestorationFailed(cause: error, stackTrace: stackTrace));
  }

  /// Expõe encerramento inesperado da observação sem deixá-la congelada.
  ///
  /// Uma chamada posterior a start pode registrar outra observação.
  void _onAuthenticationDone() {
    if (_disposed) return;

    _started = false;
    _authSubscription = null;

    final stack = StackTrace.current;

    _onAuthenticationError(
      IdentityException(
        kind: IdentityFailureKind.unexpectedExternalFailure,
        cause: StateError('O acompanhamento da autenticação foi encerrado'),
        stackTrace: stack,
      ),
      stack,
    );
  }

  /// Calcula o acesso local e publica a fotografia antes de notificar ouvintes.
  ///
  /// O timer é rearmado para a expiração ou próxima tentativa. Dados e
  /// diagnósticos permanecem preservados quando a política muda por expiração.
  void _emit(SessionState next) {
    if (_disposed) return;

    if (next is SessionSignedIn) {
      next = SessionSignedIn(
        firebaseUid: next.firebaseUid,
        user: _users[next.firebaseUid],
        verification: next.verification,
        cacheLoadResult: next.cacheLoadResult,
        operation: next.operation,
        failure: next.failure,
        failureDetails: next.failureDetails,
        access: next.operation == SessionOperation.restoringCache
            ? SessionAccess.restoring
            : next.hasUnexpiredVerificationAt(_now())
            ? ((_users[next.firebaseUid]?.status != InternalUserStatus.active ||
                      !(_accountAuthentication.hasPassword))
                  ? SessionAccess.profileRequired
                  : SessionAccess.ready)
            : SessionAccess.entryRequired,
      );
    }

    _state = next;

    _changes.add(next);
    _scheduleRefresh();
  }

  /// Obtém a agenda pertencente à conta e à evidência desta fotografia.
  _RenewalSchedule _scheduleFor(SessionSignedIn current) {
    final key = (current.firebaseUid, current.verification?.expiresAt);

    if (_renewalSchedule?.key != key) {
      _renewalSchedule = _RenewalSchedule(key: key);
    }

    return _renewalSchedule!;
  }

  /// Próximo instante elegível; nulo representa espera por ação explícita.
  ///
  /// Sem agenda, a primeira tentativa ocorre na janela de antecipação ou
  /// imediatamente se o cache não forneceu evidência. A leitura é pura.
  DateTime? _automaticRenewalAt(SessionSignedIn current) {
    final key = (current.firebaseUid, current.verification?.expiresAt);

    if (_renewalSchedule?.key == key) return _renewalSchedule!.nextAttemptAt;

    return current.verification?.expiresAt.subtract(_automaticRenewalLead) ??
        _now();
  }

  /// Agenda o próximo vencimento ou tentativa, inclusive com acesso pendente.
  ///
  /// Uma operação em curso mantém apenas o timer de expiração. Seu finally
  /// rearma a agenda após liberar a exclusão mútua. Retomadas preservam os
  /// instantes absolutos, sem reiniciar ou antecipar a espera progressiva.
  void _scheduleRefresh() {
    _refreshTimer?.cancel();

    _refreshTimer = null;

    final current = _state;

    if (_disposed || current is! SessionSignedIn) return;

    final now = _now();
    DateTime? nextAt = current.hasUnexpiredVerificationAt(now)
        ? current.verification!.expiresAt
        : null;

    if (!current.isBusy && _activeEntry == null && _activeSignOut == null) {
      final renewalAt = _automaticRenewalAt(current);

      if (renewalAt != null && (nextAt == null || renewalAt.isBefore(nextAt))) {
        nextAt = renewalAt;
      }
    }

    if (nextAt == null) return;

    final remaining = nextAt.difference(now);
    final delay = remaining <= Duration.zero
        ? Duration.zero
        : remaining > const Duration(days: 1)
        ? const Duration(days: 1)
        : remaining;

    _refreshTimer = Timer(delay, refresh);
  }

  /// Solicita o encerramento e compartilha o mesmo Future entre chamadas.
  ///
  /// Invalida tentativas e timers, cancela a inscrição e fecha os estados.
  /// Não aguarda I/O dos adaptadores, executa logout ou remove cache. Leituras
  /// tardias não publicam e uma escrita já iniciada pode concluir depois.
  /// Falhas de encerramento propagam.
  Future<void> dispose() {
    return _disposeFuture ??= _disposeResources();
  }

  /// Fecha os recursos próprios mesmo quando cancelar a inscrição falha.
  Future<void> _disposeResources() async {
    _disposed = true;

    ++_revision;

    _activeEntry = null;
    _activeSignOut = null;

    _refreshTimer?.cancel();

    try {
      await _authSubscription?.cancel();
    } finally {
      await _changes.close();
    }
  }
}

/// Agenda efêmera de recuperação de uma conta e sua evidência conhecida.
///
/// Expiração nula identifica recuperação sem cache. Trocas de identidade
/// descartam a agenda; resultados obsoletos não podem alterá-la. O contador
/// satura no último intervalo, sem crescer indefinidamente durante falhas.
final class _RenewalSchedule {
  /// Inicia a agenda sem falhas transitórias para a conta/evidência informada.
  _RenewalSchedule({required this.key, this.nextAttemptAt});

  /// UID e vencimento conhecidos; vencimento nulo representa cache sem evidência.
  final (String, DateTime?) key;

  /// Próxima tentativa permitida; nulo suspende a repetição automática.
  DateTime? nextAttemptAt;

  /// Índice saturado da espera após falhas transitórias consecutivas.
  int transientFailures = 0;
}

/// Identidade e revisão de uma saída, incluindo a limpeza após UID nulo.
final class _SignOutOperation {
  /// Captura a conta cuja autenticação e cache serão encerrados.
  const _SignOutOperation({required this.firebaseUid, required this.revision});

  /// Conta cujo registro local será removido.
  final String firebaseUid;

  /// Geração que invalida conclusões após troca de conta ou descarte.
  final int revision;
}

/// Contexto de uma tentativa, distinto mesmo quando o UID se repete.
final class _EntryOperation {
  /// Captura a intenção, a identidade esperada e sua revisão inicial.
  _EntryOperation({
    required this.renewal,
    required this.firebaseUid,
    required this.revision,
  });

  /// Distingue renovação da primeira identidade esperada durante login.
  final bool renewal;

  /// UID esperado ou associado ao primeiro evento da autenticação.
  String? firebaseUid;

  /// Revisão atualizada quando o login estabelece a identidade esperada.
  int revision;
}
