/// Adaptação da autenticação Firebase às portas do módulo de identidade.
///
/// O aplicativo usa Firebase para manter a autenticação e Microsoft para fornecer
/// a credencial delegada consumida pelo backend. Esta biblioteca concentra os
/// detalhes desses SDKs, mantendo o caso de uso independente de User,
/// UserCredential e dos mecanismos de interação de cada plataforma.
///
/// ## Uma identidade compartilhada
///
/// A mesma instância de FirebaseAuth implementa AuthSession, MicrosoftSignIn e
/// a obtenção do ID token usado no transporte. AppDependencies compartilha esse
/// adaptador para que a conta observada corresponda às requisições protegidas.
///
/// watchUserId adapta authStateChanges para UID e remove valores consecutivos
/// iguais. A primeira emissão informa a restauração do SDK. Erros continuam como
/// erros do stream, sem produzir um evento falso de logout.
///
/// ## Login e reautenticação
///
/// O provedor solicita User.Read, sem fixar um tenant. Na web, utiliza popup;
/// nas plataformas nativas com suporte, utiliza o provedor do SDK.
/// O resultado precisa conter um access token Microsoft não vazio.
/// Reautenticação exige uma conta existente, que será novamente conferida
/// pelo caso de uso e pelo controlador.
///
/// Estabelecer um UID Firebase não conclui a entrada no CApp: a verificação
/// institucional no backend ainda precisa ser realizada.
///
/// ## Dois tipos de token
///
/// O access token Microsoft é obtido da credencial do login e segue para o corpo
/// da verificação. O ID token Firebase é consultado no usuário atual por requisição
/// protegida e segue no cabeçalho Authorization. Nenhum deles é gravado no cache
/// institucional.
///
/// getFirebaseIdToken retorna nulo apenas na ausência de usuário. Uma conta cuja
/// resposta não fornece token utilizável produz falha explícita.
///
/// ## Falhas e ciclo de vida
///
/// FirebaseAuthException conhecidas são traduzidas para IdentityException,
/// preservando causa e pilha sem tornar mensagens arbitrárias do SDK públicas.
/// Falhas inesperadas continuam propagando. A configuração do provedor e o
/// suporte da plataforma são pré-condições externas.
///
/// O SDK possui a persistência da autenticação. signOut encerra a conta local,
/// mas não limpa o perfil em preferências ou revoga evidências do backend.
/// O adaptador não assume o encerramento do aplicativo Firebase global.
library;

import 'dart:async';

import 'package:firebase_auth/firebase_auth.dart';
import 'package:flutter/foundation.dart';

import '../../application/errors/identity_exception.dart';
import '../../application/errors/session_identity_changed_exception.dart';
import '../../application/ports/auth_session.dart';
import '../../application/ports/microsoft_sign_in.dart';

/// Implementa os contratos de autenticação sobre uma única instância Firebase.
///
/// Pressupõe configuração do provedor e suporte da plataforma. O SDK possui
/// a persistência da autenticação; esta classe não possui o perfil institucional.
/// Seu descarte não é responsável por encerrar o aplicativo Firebase.
final class FirebaseMicrosoftAuth implements MicrosoftSignIn, AuthSession {
  /// Recebe a instância compartilhada pela observação e pelas requisições.
  ///
  /// Construir não apresenta login nem registra inscrições.
  const FirebaseMicrosoftAuth(this._firebaseAuth);

  final FirebaseAuth _firebaseAuth;

  /// Observa o UID restaurado e as alterações posteriores.
  ///
  /// A primeira emissão vem da restauração do SDK. Valores consecutivos iguais
  /// são removidos. Falhas conhecidas são traduzidas, sem fabricar UID nulo.
  /// Falhas inesperadas continuam como erros do stream com sua pilha.
  @override
  Stream<String?> watchUserId() {
    return _firebaseAuth
        .authStateChanges()
        .transform(
          StreamTransformer<User?, String?>.fromHandlers(
            handleData: (user, sink) => sink.add(user?.uid),
            handleError:
                (Object error, StackTrace stack, EventSink<String?> sink) {
                  sink.addError(
                    error is FirebaseAuthException
                        ? _translate(error, stack)
                        : error,
                    stack,
                  );
                },
          ),
        )
        .distinct();
  }

  /// Consulta o UID conhecido sem validar perfil ou autorização no backend.
  ///
  /// A ausência inicial só pode ser interpretada depois da primeira emissão
  /// de [watchUserId]. A consulta não substitui esse acompanhamento.
  @override
  String? get currentUserId => _firebaseAuth.currentUser?.uid;

  /// Obtém o ID token utilizado pelo transporte das requisições protegidas.
  ///
  /// Retorna nulo somente quando não existe usuário no SDK. Uma conta sem token
  /// utilizável produz falha explícita de resposta, em vez de token ausente.
  /// Exceções Firebase são traduzidas com causa e pilha preservadas.
  ///
  /// O ID token é distinto do access token Microsoft e não é armazenado aqui.
  Future<String?> getFirebaseIdToken() async {
    final user = _firebaseAuth.currentUser;
    if (user == null) return null;
    try {
      final token = await user.getIdToken();
      if (token == null || token.isEmpty) {
        throw _missingToken('O Firebase não retornou um ID token');
      }
      return token;
    } on FirebaseAuthException catch (error, stack) {
      Error.throwWithStackTrace(_translate(error, stack), stack);
    }
  }

  /// Encerra a autenticação local e permite que o SDK publique UID nulo.
  ///
  /// Não remove cache, revoga tokens de outros dispositivos ou cancela ouvintes.
  /// Falhas Firebase são traduzidas; erros inesperados propagam.
  @override
  Future<void> signOut() async {
    try {
      await _firebaseAuth.signOut();
    } on FirebaseAuthException catch (error, stack) {
      Error.throwWithStackTrace(_translate(error, stack), stack);
    }
  }

  /// Autentica com Microsoft e retorna a credencial delegada não vazia.
  ///
  /// Usa popup na web e provedor nativo nas plataformas suportadas. Pode emitir
  /// o UID antes de concluir. Cancelamentos e recusas mantêm sua classificação.
  /// Uma resposta sem access token produz IdentityException.invalidResponse.
  @override
  Future<String> signInAndGetAccessToken() async {
    try {
      final provider = _createProvider();
      final result = kIsWeb
          ? await _firebaseAuth.signInWithPopup(provider)
          : await _firebaseAuth.signInWithProvider(provider);
      return _extractAccessToken(result);
    } on FirebaseAuthException catch (error, stack) {
      Error.throwWithStackTrace(_translate(error, stack), stack);
    }
  }

  /// Obtém uma nova credencial para a conta atual sem trocar sua identidade.
  ///
  /// Ausência da conta lança SessionIdentityChangedException. O SDK rejeita
  /// credenciais de outra conta. Falhas e cancelamentos são traduzidos;
  /// concluir não renova a evidência institucional sem consultar o backend.
  @override
  Future<String> reauthenticateAndGetAccessToken() async {
    final user = _firebaseAuth.currentUser;
    if (user == null) throw const SessionIdentityChangedException();
    try {
      final provider = _createProvider();
      final result = kIsWeb
          ? await user.reauthenticateWithPopup(provider)
          : await user.reauthenticateWithProvider(provider);
      return _extractAccessToken(result);
    } on FirebaseAuthException catch (error, stack) {
      Error.throwWithStackTrace(_translate(error, stack), stack);
    }
  }

  /// Solicita o escopo delegado necessário à verificação, sem fixar tenant.
  MicrosoftAuthProvider _createProvider() =>
      MicrosoftAuthProvider()..addScope('User.Read');

  /// Exige o access token da resposta sem registrar seu conteúdo.
  String _extractAccessToken(UserCredential result) {
    final token = result.credential?.accessToken;
    if (token == null || token.isEmpty) {
      throw _missingToken('A Microsoft não retornou um access token');
    }
    return token;
  }

  /// Representa resposta de credencial incompleta com diagnóstico explícito.
  IdentityException _missingToken(String message) => IdentityException(
    kind: IdentityFailureKind.invalidResponse,
    cause: FormatException(message),
    stackTrace: StackTrace.current,
  );

  /// Traduz códigos conhecidos e conserva a exceção original.
  ///
  /// Mensagens arbitrárias do SDK não são marcadas como públicas. Códigos novos
  /// permanecem em uma falha operacional não classificada, com causa acessível.
  IdentityException _translate(FirebaseAuthException error, StackTrace stack) {
    final kind = switch (error.code) {
      'popup-closed-by-user' ||
      'cancelled-popup-request' ||
      'web-context-canceled' ||
      'user-cancelled' => IdentityFailureKind.authenticationCancelled,
      'network-request-failed' => IdentityFailureKind.communicationFailed,
      'too-many-requests' ||
      'quota-exceeded' => IdentityFailureKind.serviceUnavailable,
      'invalid-credential' ||
      'invalid-login-credentials' ||
      'user-disabled' ||
      'user-not-found' ||
      'wrong-password' ||
      'user-mismatch' ||
      'invalid-user-token' ||
      'user-token-expired' ||
      'requires-recent-login' ||
      'account-exists-with-different-credential' =>
        IdentityFailureKind.credentialsRejected,
      _ => IdentityFailureKind.unexpectedExternalFailure,
    };
    return IdentityException(kind: kind, cause: error, stackTrace: stack);
  }
}
