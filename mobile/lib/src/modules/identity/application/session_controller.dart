/// Coordenação da sessão e da política local de entrada no aplicativo.
///
/// A autenticação do provedor, a verificação institucional e sua cópia persistida
/// possuem ciclos de vida diferentes. Esta biblioteca os coordena para que uma
/// identidade Firebase isolada ou um registro local de outra conta não sejam
/// interpretados como permissão para montar a área principal.
///
/// ## Papel na arquitetura
///
/// SessionController é a entrada das operações de sessão usadas pela apresentação.
/// Consome AuthSession, SessionStore e InstitutionalAuthService; publica modelos
/// de aplicação sem conhecer SDKs, transporte HTTP ou widgets.
/// SessionGate apresenta seu destino, e SessionLifecycleObserver informa retomadas.
///
/// ## Restauração inicial
///
/// 1. start registra a observação e aguarda o primeiro UID restaurado pelo SDK.
/// 2. Sem conta, publica SessionSignedOut e não consulta o cache.
/// 3. Com conta, mantém carregamento enquanto lê somente o registro daquele UID.
/// 4. Uma verificação encontrada e ainda válida permite entrada direta.
/// 5. Cache ausente, rejeitado, inacessível ou expirado mantém a entrada pendente.
///
/// A restauração não abre Microsoft nem consulta o backend. O resultado de cache
/// e seu diagnóstico permanecem no estado; continuar pela entrada não transforma
/// falha de leitura em registro ausente.
///
/// ## Entrada e persistência
///
/// signIn escolhe login quando não existe conta e reautenticação quando ela já é
/// conhecida. Microsoft e backend formam a mesma operação de entrada. Uma emissão
/// de UID durante esse processo não libera a área principal antes da resposta.
///
/// A verificação precisa pertencer à conta ativa e estar válida no relógio local.
/// O sucesso é publicado em memória antes da tentativa de gravação. Se somente
/// a persistência falhar, a entrada continua permitida e a operação lança
/// SessionCacheWriteException com o resultado confirmado e o diagnóstico original.
///
/// ## Saída e limpeza local
///
/// signOut encerra a autenticação antes de remover o cache da conta. A ausência
/// de UID já bloqueia a área principal, mas a entrada permanece ocupada até
/// concluir a tentativa de remoção. Falha do provedor preserva a identidade
/// efetivamente observada e seu diagnóstico, sem simular sucesso.
///
/// Se somente a limpeza falhar, publica SessionSignedOut com o aviso e lança
/// SessionCacheRemovalException. O cache remanescente não autentica a conta.
/// Eventos ou conclusões de outra sessão não podem atualizar o contexto atual.
///
/// ## Validade e concorrência
///
/// O controlador calcula SessionState.access e agenda a expiração. refresh
/// reavalia identidade e prazo na retomada, sem renovar pela rede. Ao expirar,
/// a entrada volta a ser exigida; o backend continua autorizando cada requisição
/// independentemente desse auxílio local.
///
/// Revisões e contextos de operação rejeitam conclusões obsoletas, inclusive
/// após uma sequência de contas A/B/A. O acesso ao armazenamento é serializado.
/// Essas guardas impedem publicação indevida, mas não revertem I/O já iniciado.
///
/// ## Falhas e encerramento
///
/// Falhas esperadas de login e verificação ficam no estado e também propagam
/// pelo Future. Problemas na observação da autenticação possuem um estado próprio,
/// sem fabricar logout. A interface usa a classificação pública e preserva a causa
/// técnica.
///
/// O proprietário chama dispose para invalidar operações, cancelar o timer e a
/// inscrição e fechar o stream. O encerramento é compartilhado e não aguarda
/// todo I/O externo ou local pendente. Descarte não substitui signOut e não
/// solicita logout ou limpeza de dados no provedor.
library;

import 'dart:async';

import 'errors/identity_exception.dart';
import 'errors/session_cache_removal_exception.dart';
import 'errors/session_cache_write_exception.dart';
import 'errors/session_identity_changed_exception.dart';
import 'institutional_auth_service.dart';
import 'models/institutional_verification_result.dart';
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
/// [signOut] encerra a autenticação e tenta remover o cache dessa conta.
/// O proprietário encerra a instância com [dispose], sem executar logout.
/// Provisionamento de usuário no backend não é implementado aqui.
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
  });

  final AuthSession _authSession;
  final SessionStore _sessionStore;
  final InstitutionalAuthService _authService;
  final SessionClock _now;
  final StreamController<SessionState> _changes =
      StreamController<SessionState>.broadcast();
  StreamSubscription<String?>? _authSubscription;
  Timer? _expirationTimer;
  SessionState _state = const SessionRestoring();
  _VerificationOperation? _activeVerification;

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
  /// Escolhe login Microsoft sem conta autenticada ou reautenticação quando
  /// a conta já foi restaurada, mas ainda precisa de uma verificação válida.
  ///
  /// Retorna UID e verificação após persistir. Falhas externas propagam e ficam
  /// no estado da sessão correspondente. Falha exclusiva de gravação lança
  /// [SessionCacheWriteException] e mantém a entrada permitida em memória.
  ///
  /// Exige restauração concluída, ausência de operação e entrada pendente.
  /// Violações lançam [StateError]; troca de conta pode lançar
  /// [SessionIdentityChangedException].
  Future<InstitutionalVerificationResult> signIn() {
    if (_state.access == SessionAccess.ready) {
      throw StateError('A sessão já está pronta para entrar no aplicativo');
    }
    return _runVerification(renewal: _state is SessionSignedIn);
  }

  /// Obtém outra verificação da conta atual, preservando o perfil anterior.
  ///
  /// Exige autenticação e ociosidade. Não é disparada automaticamente e não
  /// instala um botão de verificação na experiência de entrada.
  ///
  /// Retorno e falhas seguem [signIn], mas uma sessão já pronta também pode
  /// solicitar esta operação explicitamente.
  Future<InstitutionalVerificationResult> renewVerification() {
    return _runVerification(renewal: true);
  }

  /// Encerra a autenticação e remove o cache somente da conta correspondente.
  ///
  /// Exige restauração concluída, uma conta autenticada e ausência de outra
  /// operação; violações lançam [StateError]. Marca signingOut antes de aguardar
  /// o provedor, impedindo chamadas simultâneas e novos logins durante a limpeza.
  ///
  /// Primeiro solicita logout à [AuthSession]. Sua emissão de UID nulo mantém
  /// a saída em andamento. Depois serializa a remoção com o restante do I/O
  /// local, evitando que uma escrita anterior recrie o registro após a limpeza.
  ///
  /// Conclui sem valor quando ambas as etapas terminam. Falhas do provedor ficam
  /// no estado e propagam com causa e pilha. Se apenas a remoção falhar, mantém
  /// o usuário deslogado e lança [SessionCacheRemovalException] com o diagnóstico.
  /// Troca de conta ou descarte lança [SessionIdentityChangedException] sem
  /// publicar resultados na nova sessão. Efeitos já iniciados não são revertidos.
  ///
  /// Não revoga tokens de outros dispositivos nem remove evidências do backend.
  Future<void> signOut() async {
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
      await _authSession.signOut();
      _requireSignedOutOperation(operation);
      try {
        await _withStore(() async {
          _requireSignedOutOperation(operation);
          await _sessionStore.remove(operation.firebaseUid);
        });
      } on SessionIdentityChangedException {
        rethrow;
      } on Exception catch (error, stackTrace) {
        Error.throwWithStackTrace(
          SessionCacheRemovalException(
            firebaseUid: operation.firebaseUid,
            cause: error,
            causeStackTrace: stackTrace,
          ),
          stackTrace,
        );
      }
      _requireSignedOutOperation(operation);
      _emit(const SessionSignedOut());
    } on Exception catch (error, stackTrace) {
      _publishSignOutFailure(operation, error, stackTrace);
      rethrow;
    } finally {
      if (identical(_activeSignOut, operation)) _activeSignOut = null;
    }
  }

  /// Reavalia identidade e validade ao retomar a aplicação.
  ///
  /// Não consulta o backend, abre login ou remove cache. Antes da primeira
  /// identidade restaurada ou depois do descarte, não executa ações.
  /// Uma divergência de UID inicia a restauração da conta atual.
  void refresh() {
    if (_disposed || !_started || !_receivedUser) return;
    final actualUid = _authSession.currentUserId;
    if (actualUid != _currentUid) {
      _onUserChanged(actualUid);
    } else {
      _emit(_state);
    }
  }

  /// Mantém o contexto da verificação até concluir a tentativa de persistência.
  ///
  /// Publica sucesso em memória antes de gravar. Só o próprio contexto é
  /// liberado ao terminar; uma operação antiga não encerra a de outra conta.
  Future<InstitutionalVerificationResult> _runVerification({
    required bool renewal,
  }) async {
    _requireReadyForOperation(requiresAuthentication: renewal);
    final previous = _state;
    final operation = _VerificationOperation(
      renewal: renewal,
      firebaseUid: renewal ? _currentUid : null,
      revision: ++_revision,
    );
    _activeVerification = operation;

    if (previous is SessionSignedIn) {
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
      final InstitutionalVerificationResult result;
      try {
        result = renewal
            ? await _authService.reauthenticateAndVerify()
            : await _authService.signInAndVerify();
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
        _publishVerificationFailure(operation, error, stackTrace);
        rethrow;
      }

      _requireActiveOperation(operation, result.firebaseUid);
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
      if (identical(_activeVerification, operation)) {
        _activeVerification = null;
      }
    }
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
    if (_activeVerification != null ||
        _activeSignOut != null ||
        _state.isBusy) {
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
    final revision = ++_revision;
    final operation = _activeVerification;

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

    _activeVerification = null;
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
    final result = await _withStore(() => _sessionStore.load(firebaseUid));
    if (!_isCurrent(firebaseUid, revision)) return;

    _emit(
      SessionSignedIn(
        firebaseUid: firebaseUid,
        verification: switch (result) {
          SessionCacheFound(:final verification) => verification,
          _ => null,
        },
        cacheLoadResult: result,
        failure: switch (result) {
          SessionCacheRejected() => SessionFailure.cacheRejected,
          SessionCacheReadFailed() => SessionFailure.cacheReadFailed,
          _ => null,
        },
      ),
    );
  }

  /// Preserva a falha e seu diagnóstico somente na sessão correspondente.
  ///
  /// Uma exceção de contexto antigo continua no Future original, mas não
  /// substitui os dados da conta atual.
  void _publishVerificationFailure(
    _VerificationOperation operation,
    Exception error,
    StackTrace stackTrace,
  ) {
    if (_disposed || !identical(_activeVerification, operation)) return;
    final actualUid = _authSession.currentUserId;
    if (actualUid != _currentUid) _onUserChanged(actualUid);
    if (!identical(_activeVerification, operation)) return;

    final current = _state;
    final details = SessionFailureDetails(cause: error, stackTrace: stackTrace);
    final failure = _classifyFailure(error, current);
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
  void _requireActiveOperation(
    _VerificationOperation operation,
    String firebaseUid,
  ) {
    if (_disposed || !identical(_activeVerification, operation)) {
      throw const SessionIdentityChangedException();
    }
    final actualUid = _authSession.currentUserId;
    if (actualUid != _currentUid) _onUserChanged(actualUid);
    if (_canPublish(operation, firebaseUid)) return;

    if (identical(_activeVerification, operation)) {
      _activeVerification = null;
      _receivedUser = false;
      _onUserChanged(actualUid);
    }
    throw const SessionIdentityChangedException();
  }

  /// Confere o contexto e a identidade vinculados a uma publicação.
  bool _canPublish(_VerificationOperation operation, String firebaseUid) {
    return !_disposed &&
        identical(_activeVerification, operation) &&
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
    _activeVerification = null;
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
  /// O timer é rearmado para a evidência atual. Dados e diagnósticos permanecem
  /// preservados quando a política muda por expiração.
  void _emit(SessionState next) {
    if (_disposed) return;
    _expirationTimer?.cancel();
    _expirationTimer = null;

    if (next is SessionSignedIn) {
      next = SessionSignedIn(
        firebaseUid: next.firebaseUid,
        verification: next.verification,
        cacheLoadResult: next.cacheLoadResult,
        operation: next.operation,
        failure: next.failure,
        failureDetails: next.failureDetails,
        access: next.operation == SessionOperation.restoringCache
            ? SessionAccess.restoring
            : next.hasUnexpiredVerificationAt(_now())
            ? SessionAccess.ready
            : SessionAccess.entryRequired,
      );
    }

    _state = next;
    _changes.add(next);
    if (next is SessionSignedIn && next.access == SessionAccess.ready) {
      final remaining = next.verification!.expiresAt.difference(_now());
      if (remaining <= Duration.zero) {
        _expirationTimer = Timer(Duration.zero, refresh);
      } else {
        // Mantém a espera dentro de um intervalo suportado também na web.
        final delay = remaining > const Duration(days: 1)
            ? const Duration(days: 1)
            : remaining;
        _expirationTimer = Timer(delay, refresh);
      }
    }
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
    _activeVerification = null;
    _activeSignOut = null;
    _expirationTimer?.cancel();
    try {
      await _authSubscription?.cancel();
    } finally {
      await _changes.close();
    }
  }
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
final class _VerificationOperation {
  /// Captura a intenção, a identidade esperada e sua revisão inicial.
  _VerificationOperation({
    required this.renewal,
    required this.firebaseUid,
    required this.revision,
  });

  /// Distingue reautenticação da primeira identidade esperada durante login.
  final bool renewal;

  /// UID esperado ou associado ao primeiro evento da autenticação.
  String? firebaseUid;

  /// Revisão atualizada quando o login estabelece a identidade esperada.
  int revision;
}
