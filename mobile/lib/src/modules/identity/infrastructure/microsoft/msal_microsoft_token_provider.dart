/// Adaptação do MSAL às operações de autorização Microsoft.
///
/// O token Microsoft retornado por um login não cobre todo o tempo de vida da
/// sessão Firebase. Esta biblioteca usa o cache e a renovação do MSAL para obter
/// credenciais Graph quando necessárias, isolando a aplicação dos tipos do SDK.
///
/// ## Papel no fluxo
///
/// AppDependencies injeta um MultipleAccountPca compartilhado e seleciona por objeto Microsoft. FirebaseMicrosoftAuth
/// usa este adaptador por MicrosoftTokenProvider, e encaminha a credencial para
/// a verificação institucional. O backend consulta o Graph e valida o vínculo;
/// este adaptador não faz essas consultas nem decide quando renovar a evidência.
///
/// A obtenção silenciosa pode reutilizar o cache ou acessar a rede. Ela nunca
/// abre autorização como efeito colateral. A obtenção interativa é uma operação
/// separada, chamada somente quando a coordenação permite participação do usuário.
/// O SDK possui o cache de tokens; não há cópia em preferências do aplicativo.
///
/// ## Falhas e limpeza
///
/// Necessidade de interação ou ausência de conta produz
/// MicrosoftInteractionRequiredException. Outras falhas MSAL são traduzidas
/// para IdentityException preservando causa e pilha. Token vazio é resposta
/// inválida; erros de programação continuam propagando.
///
/// clearSession remove a conta pelo SDK. Ausência prévia é sucesso; uma resposta
/// negativa de signOut é falha explícita. A operação não encerra o Firebase,
/// apaga o cache institucional ou revoga evidências no backend.
library;

import 'package:msal_auth/msal_auth.dart';

import '../../application/errors/identity_exception.dart';
import '../../application/errors/microsoft_interaction_required_exception.dart';
import '../../application/ports/microsoft_token_provider.dart';

/// Obtém credenciais Graph usando uma instância MSAL compartilhada.
final class MsalMicrosoftTokenProvider implements MicrosoftTokenProvider {
  /// Recebe o cliente inicializado, sem iniciar obtenção de tokens ou ouvintes.
  const MsalMicrosoftTokenProvider(SingleAccountPca client)
    : _client = client,
      _accountId = null;

  /// Seleciona explicitamente o objeto Microsoft associado ao UID Firebase.
  /// No modo múltiplo, ausência de conta nunca seleciona outra silenciosamente.
  const MsalMicrosoftTokenProvider.forAccount(
    MultipleAccountPca client, {
    required this._accountId,
  }) : _client = client;

  final PublicClientApplication _client;
  final String? _accountId;

  static const _scopes = ['https://graph.microsoft.com/User.Read'];

  /// Obtém um token Graph sem interação, preservando falhas de comunicação.
  @override
  Future<String> getAccessTokenSilently() => _execute(() async {
    if (_client is MultipleAccountPca && _accountId == null) {
      throw MicrosoftInteractionRequiredException(
        cause: StateError('Conta Microsoft ausente'),
        stackTrace: StackTrace.current,
      );
    }

    final client = _client;

    if (client is MultipleAccountPca &&
        !(await client.getAccounts()).any(
          (a) => a.id.toLowerCase() == _accountId!.toLowerCase(),
        )) {
      throw MicrosoftInteractionRequiredException(
        cause: StateError('Conta Microsoft não está no cache'),
        stackTrace: StackTrace.current,
      );
    }

    final result = await _client.acquireTokenSilent(
      scopes: _scopes,
      identifier: _accountId,
    );

    return _readAccessToken(result);
  });

  /// Solicita autorização quando necessária, usando [loginHint] como sugestão.
  ///
  /// Não comprova correspondência com a conta Firebase; essa validação pertence
  /// ao backend. Cancelamento e recusa mantêm suas categorias da aplicação.
  @override
  Future<String> getAccessTokenInteractively({String? loginHint}) =>
      _execute(() async {
        final result = await _client.acquireToken(
          scopes: _scopes,
          prompt: Prompt.whenRequired,
          loginHint: loginHint,
        );

        return _readAccessToken(result);
      });

  /// Remove a conta MSAL local; ausência de conta já satisfaz o contrato.
  ///
  /// A resposta deve confirmar a remoção. Falhas propagam para que a operação
  /// de saída ou de novo login não prossiga como se a limpeza tivesse concluído.
  @override
  Future<void> clearSession() => _execute(() async {
    try {
      final bool cleared;
      final client = _client;

      if (client is MultipleAccountPca) {
        if (_accountId == null) return;

        final accounts = await client.getAccounts();

        if (!accounts.any(
          (a) => a.id.toLowerCase() == _accountId.toLowerCase(),
        )) {
          return;
        }

        cleared = await client.removeAccount(identifier: _accountId);
      } else {
        final single = client as SingleAccountPca;

        await single.currentAccount;

        cleared = await single.signOut();
      }

      if (!cleared) {
        throw IdentityException(
          kind: IdentityFailureKind.unexpectedExternalFailure,
          cause: StateError('O MSAL não confirmou a remoção da conta.'),
          stackTrace: StackTrace.current,
        );
      }
    } on MsalNoCurrentAccountException {
      // A sessão Microsoft já está ausente.
    }
  });

  /// Exige uma credencial não vazia sem interpretar ou registrar seu conteúdo.
  String _readAccessToken(AuthenticationResult result) {
    if (result.accessToken.trim().isEmpty) {
      throw IdentityException(
        kind: IdentityFailureKind.invalidResponse,
        cause: const FormatException('O MSAL retornou um access token vazio.'),
        stackTrace: StackTrace.current,
      );
    }

    return result.accessToken;
  }

  /// Traduz falhas reconhecidas do SDK conservando o diagnóstico original.
  Future<T> _execute<T>(Future<T> Function() operation) async {
    try {
      return await operation();
    } on MsalUiRequiredException catch (error, stackTrace) {
      Error.throwWithStackTrace(
        MicrosoftInteractionRequiredException(
          cause: error,
          stackTrace: stackTrace,
        ),
        stackTrace,
      );
    } on MsalNoCurrentAccountException catch (error, stackTrace) {
      Error.throwWithStackTrace(
        MicrosoftInteractionRequiredException(
          cause: error,
          stackTrace: stackTrace,
        ),
        stackTrace,
      );
    } on MsalException catch (error, stackTrace) {
      Error.throwWithStackTrace(
        IdentityException(
          kind: _failureKind(error),
          cause: error,
          stackTrace: stackTrace,
        ),
        stackTrace,
      );
    }
  }

  /// Classifica causas operacionais sem expor mensagens arbitrárias do SDK.
  IdentityFailureKind _failureKind(MsalException error) => switch (error) {
    MsalUserCancelException() => IdentityFailureKind.authenticationCancelled,
    MsalDeclinedScopeException() => IdentityFailureKind.credentialsRejected,
    MsalClientException(
      errorCode: 'device_network_not_available' || 'io_error',
    ) =>
      IdentityFailureKind.communicationFailed,
    MsalServiceException(errorCode: 'request_timeout') =>
      IdentityFailureKind.communicationFailed,
    MsalServiceException(:final httpStatusCode)
        when httpStatusCode == 429 ||
            (httpStatusCode >= 500 && httpStatusCode < 600) =>
      IdentityFailureKind.serviceUnavailable,
    MsalServiceException(errorCode: 'service_not_available') =>
      IdentityFailureKind.serviceUnavailable,
    MsalServiceException(errorCode: 'access_denied' || 'invalid_grant') =>
      IdentityFailureKind.credentialsRejected,
    _ => IdentityFailureKind.unexpectedExternalFailure,
  };
}
