/// Adaptação da autenticação Firebase e sua integração com credenciais Microsoft.
///
/// O aplicativo mantém sua identidade autenticada no Firebase, mas a consulta
/// institucional exige uma credencial Microsoft com ciclo de vida independente.
/// Este adaptador combina essas operações por meio de MicrosoftTokenProvider,
/// mantendo o caso de uso independente de User, UserCredential e dos mecanismos
/// concretos de obtenção de tokens. O SDK MSAL fica em outro adaptador.
///
/// ## Uma identidade compartilhada
///
/// Este adaptador usa uma instância de FirebaseAuth para AuthSession, MicrosoftSignIn e
/// a obtenção do ID token usado no transporte. AppDependencies compartilha esse
/// adaptador para que a conta observada corresponda às requisições protegidas.
///
/// watchUserId adapta authStateChanges para UID e remove valores consecutivos
/// iguais. A primeira emissão informa a restauração do SDK. Erros continuam como
/// erros do stream, sem produzir um evento falso de logout.
///
/// ## Login inicial e credenciais da conta atual
///
/// O login estabelece somente a conta Firebase e preserva o cache MSAL. A
/// aplicação consulta o backend antes de solicitar uma credencial Graph: contas
/// com confirmação vigente não precisam dessa segunda etapa. O token Microsoft
/// que o Firebase eventualmente retorna não é usado. Na composição Android atual,
/// Firebase e MSAL têm sessões próprias e podem apresentar duas autorizações
/// quando é necessário confirmar o vínculo institucional.
///
/// Para uma conta já autenticada, getAccessTokenForCurrentUser tenta a obtenção
/// silenciosa. Somente MicrosoftInteractionRequiredException permite passar à
/// obtenção interativa, e apenas quando allowInteraction é verdadeiro. Uma falha
/// de comunicação propaga sem abrir autorização. Não se executa reautenticação
/// Firebase para renovar a evidência institucional.
///
/// A permanência do UID é conferida antes e depois das etapas assíncronas.
/// O e-mail enviado como loginHint apenas sugere uma conta: não prova que o
/// token retornado corresponde à identidade Firebase. O backend valida essa
/// correspondência. As revisões do SessionController também são necessárias
/// para rejeitar resultados após trocas de sessão A/B/A.
///
/// A seleção de popup na web ou provedor nativo se aplica à etapa Firebase.
/// Ela não torna o adaptador Microsoft compatível com todas as plataformas;
/// a composição MSAL atual foi preparada para Android.
///
/// Estabelecer um UID Firebase não conclui a entrada no CApp: é preciso recuperar
/// a conta e sua confirmação institucional, renovando-a apenas quando necessário.
///
/// ## Dois tipos de token
///
/// O access token Microsoft vem de MicrosoftTokenProvider e segue para o corpo
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
/// Os SDKs possuem suas persistências de autenticação e credenciais. signOut
/// limpa a sessão Microsoft antes de encerrar a conta Firebase. Se a primeira
/// etapa falhar, a segunda não é executada; se a segunda falhar, a primeira
/// não é desfeita. As falhas propagam, sem simular uma saída bem-sucedida.
/// O cache institucional em preferências é removido pelo controlador após a
/// saída; este adaptador não revoga evidências do backend.
/// O adaptador não assume o encerramento do aplicativo Firebase global.
library;

import 'dart:async';

import 'package:firebase_auth/firebase_auth.dart';
import 'package:flutter/foundation.dart';

import '../../application/errors/identity_exception.dart';

import 'firebase_auth_failure.dart';
import '../../application/errors/microsoft_interaction_required_exception.dart';
import '../../application/errors/session_identity_changed_exception.dart';
import '../../application/ports/auth_session.dart';
import '../../application/ports/microsoft_sign_in.dart';
import '../../application/ports/microsoft_token_provider.dart';

/// Implementa os contratos de autenticação sobre uma única instância Firebase.
///
/// Pressupõe configuração do provedor e suporte da plataforma. O SDK possui
/// a persistência da autenticação; esta classe não possui o perfil institucional.
/// Seu descarte não é responsável por encerrar o aplicativo Firebase.
final class FirebaseMicrosoftAuth implements MicrosoftSignIn, AuthSession {
  /// Recebe a instância compartilhada pela observação e pelas requisições.
  ///
  /// Construir não apresenta login nem registra inscrições.
  const FirebaseMicrosoftAuth(
    this._firebaseAuth, {
    required this._microsoftTokenProvider,
  });

  final FirebaseAuth _firebaseAuth;

  final MicrosoftTokenProvider _microsoftTokenProvider;

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
                        ? translateFirebaseAuthFailure(error, stack)
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
      Error.throwWithStackTrace(
        translateFirebaseAuthFailure(error, stack),
        stack,
      );
    }
  }

  /// Remove a sessão Microsoft antes de encerrar a autenticação Firebase.
  ///
  /// Uma falha na limpeza Microsoft impede prosseguir com a saída Firebase.
  /// O cache institucional continua pertencendo ao SessionController.
  @override
  Future<void> signOut() async {
    final expectedUid = _firebaseAuth.currentUser?.uid;

    await _microsoftTokenProvider.clearSession();

    if (_firebaseAuth.currentUser?.uid != expectedUid) {
      throw const SessionIdentityChangedException();
    }

    try {
      await _firebaseAuth.signOut();
    } on FirebaseAuthException catch (error, stack) {
      Error.throwWithStackTrace(
        translateFirebaseAuthFailure(error, stack),
        stack,
      );
    }
  }

  /// Estabelece a identidade Firebase antes da consulta à conta CApp.
  ///
  /// A obtenção de credenciais Graph é solicitada separadamente, somente quando
  /// a restauração indicar necessidade de provisionamento ou sincronização.
  @override
  Future<void> signIn() async {
    try {
      final provider = _createProvider();
      final result = kIsWeb
          ? await _firebaseAuth.signInWithPopup(provider)
          : await _firebaseAuth.signInWithProvider(provider);

      final user = result.user;

      if (user == null) {
        throw IdentityException(
          kind: IdentityFailureKind.invalidResponse,
          cause: const FormatException('O Firebase não retornou o usuário'),
          stackTrace: StackTrace.current,
        );
      }

      _requireSameFirebaseUser(user.uid);
    } on FirebaseAuthException catch (error, stack) {
      Error.throwWithStackTrace(
        translateFirebaseAuthFailure(error, stack),
        stack,
      );
    }
  }

  /// Obtém uma nova credencial para a conta atual sem trocar sua identidade.
  ///
  /// Ausência ou mudança da conta Firebase lança SessionIdentityChangedException.
  /// A tentativa silenciosa só admite interação quando [allowInteraction]
  /// permite. Uma autorização Microsoft bem-sucedida ainda precisa ser validada
  /// pelo backend contra a identidade vinculada ao Firebase e não renova,
  /// sozinha, a evidência institucional.
  @override
  Future<String> getAccessTokenForCurrentUser({bool allowInteraction = true}) {
    final user = _firebaseAuth.currentUser;

    if (user == null) throw const SessionIdentityChangedException();

    return _getMicrosoftAccessToken(user, allowInteraction: allowInteraction);
  }

  /// Obtém a credencial e impede aplicar o resultado a outro UID Firebase.
  ///
  /// Somente necessidade explícita de interação permite abrir o provedor.
  /// A identidade Microsoft efetiva permanece sujeita à validação no backend.
  Future<String> _getMicrosoftAccessToken(
    User user, {
    required bool allowInteraction,
  }) async {
    _requireSameFirebaseUser(user.uid);

    String token;

    try {
      token = await _microsoftTokenProvider.getAccessTokenSilently();
    } on MicrosoftInteractionRequiredException {
      _requireSameFirebaseUser(user.uid);

      if (!allowInteraction) rethrow;

      token = await _microsoftTokenProvider.getAccessTokenInteractively(
        loginHint:
            user.providerData
                .where((p) => p.providerId == 'microsoft.com')
                .firstOrNull
                ?.email ??
            user.email,
      );
    }

    _requireSameFirebaseUser(user.uid);

    return token;
  }

  /// Solicita uma autenticação Microsoft nova para uma entrada explícita.
  ///
  /// prompt=login impede que esquecer uma conta seja seguido apenas pela escolha
  /// de uma sessão do navegador já aberta. Esta política vale para novo login
  /// Firebase; renovação da evidência continua silenciosa pelo MSAL e não passa
  /// por este método. O CApp não apaga cookies de outros aplicativos.
  MicrosoftAuthProvider _createProvider() => MicrosoftAuthProvider()
    ..addScope('User.Read')
    ..setCustomParameters(const {'prompt': 'login'});

  void _requireSameFirebaseUser(String expectedUid) {
    if (_firebaseAuth.currentUser?.uid != expectedUid) {
      throw const SessionIdentityChangedException();
    }
  }

  /// Representa resposta de credencial incompleta com diagnóstico explícito.
  IdentityException _missingToken(String message) => IdentityException(
    kind: IdentityFailureKind.invalidResponse,
    cause: FormatException(message),
    stackTrace: StackTrace.current,
  );
}
