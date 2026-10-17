import 'package:flutter/foundation.dart';

import '../domain/authenticated_identity.dart';
import '../domain/authentication_repository.dart';

/// Etapa corrente da sessão de autenticação na interface.
enum AuthenticationStatus {
  /// A sessão persistida está sendo confirmada com o backend.
  restoring,

  /// Não existe uma sessão autenticada disponível.
  signedOut,

  /// O fluxo de login está em andamento.
  signingIn,

  /// Há uma identidade confirmada pelo backend.
  authenticated,

  /// O encerramento da sessão está em andamento.
  signingOut,
}

/// Estado imutável consumido pela interface de autenticação.
class AuthenticationViewState {
  const AuthenticationViewState({
    required this.status,
    this.identity,
    this.message,
  });

  final AuthenticationStatus status;
  final AuthenticatedIdentity? identity;
  final String? message;
}

/// Coordena login, restauração e encerramento da sessão.
class AuthenticationController extends ChangeNotifier {
  AuthenticationController(this._repository);

  final AuthenticationRepository _repository;
  bool _disposed = false;

  AuthenticationViewState _state = const AuthenticationViewState(
    status: AuthenticationStatus.restoring,
  );

  /// Estado atual apresentado pela interface.
  AuthenticationViewState get state => _state;

  /// Valida a sessão persistida, caso exista.
  Future<void> restoreSession() async {
    try {
      final identity = await _repository.restoreSession();
      _emit(
        AuthenticationViewState(
          status: identity == null
              ? AuthenticationStatus.signedOut
              : AuthenticationStatus.authenticated,
          identity: identity,
        ),
      );
    } on AuthenticationFailure catch (error) {
      _emit(
        AuthenticationViewState(
          status: AuthenticationStatus.signedOut,
          message: error.message,
        ),
      );
    } catch (_) {
      _emit(
        const AuthenticationViewState(
          status: AuthenticationStatus.signedOut,
          message:
              'Não foi possível restaurar a sessão. Tente entrar novamente.',
        ),
      );
    }
  }

  /// Inicia a autenticação e só libera o acesso após a confirmação da API.
  Future<void> signIn() async {
    if (_state.status == AuthenticationStatus.signingIn ||
        _state.status == AuthenticationStatus.restoring ||
        _state.status == AuthenticationStatus.signingOut) {
      return;
    }

    _emit(
      const AuthenticationViewState(status: AuthenticationStatus.signingIn),
    );

    try {
      final identity = await _repository.signIn();
      _emit(
        AuthenticationViewState(
          status: AuthenticationStatus.authenticated,
          identity: identity,
        ),
      );
    } on AuthenticationFailure catch (error) {
      _emit(
        AuthenticationViewState(
          status: AuthenticationStatus.signedOut,
          message: error.reason == AuthenticationFailureReason.cancelled
              ? null
              : error.message,
        ),
      );
    } catch (_) {
      _emit(
        const AuthenticationViewState(
          status: AuthenticationStatus.signedOut,
          message: 'Não foi possível concluir o login institucional. Tente novamente.',
        ),
      );
    }
  }

  /// Encerra a sessão local e retorna à tela de entrada.
  Future<void> signOut() async {
    if (_state.status != AuthenticationStatus.authenticated) return;

    final currentIdentity = _state.identity;
    _emit(
      AuthenticationViewState(
        status: AuthenticationStatus.signingOut,
        identity: currentIdentity,
      ),
    );

    try {
      await _repository.signOut();
      _emit(
        const AuthenticationViewState(status: AuthenticationStatus.signedOut),
      );
    } catch (_) {
      _emit(
        AuthenticationViewState(
          status: AuthenticationStatus.authenticated,
          identity: currentIdentity,
          message: 'Não foi possível encerrar a sessão. Tente novamente.',
        ),
      );
    }
  }

  void _emit(AuthenticationViewState nextState) {
    if (_disposed) return;
    _state = nextState;
    notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    super.dispose();
  }
}
