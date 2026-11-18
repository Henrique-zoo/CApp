/// Contrato entre o SDK MSAL e as falhas consumidas pela aplicação.
///
/// A coordenação precisa distinguir necessidade de interação, indisponibilidade
/// e cancelamento sem conhecer tipos MSAL. Esta suíte exercita o adaptador real
/// com um cliente controlado, verificando tradução, credenciais vazias e limpeza.
/// Não simula a renovação interna do SDK nem comprova OAuth ou configuração Android.
library;

import 'package:capp/src/modules/identity/application/errors/identity_exception.dart';
import 'package:capp/src/modules/identity/application/errors/microsoft_interaction_required_exception.dart';
import 'package:capp/src/modules/identity/infrastructure/microsoft/msal_microsoft_token_provider.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:msal_auth/msal_auth.dart';

/// Registra os resultados observáveis na fronteira da aplicação.
void main() {
  late _MsalClient client;
  late MsalMicrosoftTokenProvider provider;

  setUp(() {
    client = _MsalClient();
    provider = MsalMicrosoftTokenProvider(client);
  });

  test(
    'obtenção silenciosa pede acesso ao Graph sem abrir autorização',
    () async {
      expect(await provider.getAccessTokenSilently(), 'graph-test-token');
      expect(client.requestedScopes, ['https://graph.microsoft.com/User.Read']);
      expect(client.interactiveCalls, 0);
    },
  );

  test('obtenção interativa encaminha a sugestão de conta', () async {
    expect(
      await provider.getAccessTokenInteractively(
        loginHint: 'pessoa@example.test',
      ),
      'graph-test-token',
    );
    expect(client.requestedScopes, ['https://graph.microsoft.com/User.Read']);
    expect(client.loginHint, 'pessoa@example.test');
    expect(client.prompt, Prompt.whenRequired);
  });

  for (final failure in <MsalException>[
    const MsalUiRequiredException(
      oauthSubErrorCode: 'consent_required',
      oauthError: null,
      oauthErrorDescription: null,
      message: 'Consent required',
      correlationId: null,
    ),
    const MsalNoCurrentAccountException(message: 'No account'),
  ]) {
    test('${failure.runtimeType} exige interação sem iniciá-la', () async {
      final stack = StackTrace.current;

      client.onToken = () => Future.error(failure, stack);

      await expectLater(
        provider.getAccessTokenSilently(),
        throwsA(
          isA<MicrosoftInteractionRequiredException>()
              .having((error) => error.cause, 'causa', same(failure))
              .having((error) => error.stackTrace, 'pilha', same(stack)),
        ),
      );

      expect(client.interactiveCalls, 0);
    });
  }

  for (final (failure, kind) in [
    (
      const MsalClientException(
        errorCode: 'device_network_not_available',
        message: 'Network unavailable',
        correlationId: null,
      ),
      IdentityFailureKind.communicationFailed,
    ),
    (
      const MsalServiceException(
        errorCode: 'service_not_available',
        httpStatusCode: 503,
        message: 'Unavailable',
        correlationId: null,
      ),
      IdentityFailureKind.serviceUnavailable,
    ),
    (
      const MsalServiceException(
        errorCode: 'invalid_grant',
        httpStatusCode: 400,
        message: 'Rejected',
        correlationId: null,
      ),
      IdentityFailureKind.credentialsRejected,
    ),
  ]) {
    test('falha silenciosa mantém a categoria ${kind.name}', () async {
      final stack = StackTrace.current;

      client.onToken = () => Future.error(failure, stack);

      await expectLater(
        provider.getAccessTokenSilently(),
        throwsA(
          isA<IdentityException>()
              .having((error) => error.kind, 'categoria', kind)
              .having((error) => error.cause, 'causa', same(failure))
              .having((error) => error.stackTrace, 'pilha', same(stack)),
        ),
      );

      expect(client.interactiveCalls, 0);
    });
  }

  test('cancelamento interativo mantém categoria e causa', () async {
    const failure = MsalUserCancelException(
      message: 'Cancelled',
      correlationId: null,
    );

    client.onToken = () async => throw failure;

    await expectLater(
      provider.getAccessTokenInteractively(),
      throwsA(
        isA<IdentityException>()
            .having(
              (error) => error.kind,
              'categoria',
              IdentityFailureKind.authenticationCancelled,
            )
            .having((error) => error.cause, 'causa', same(failure)),
      ),
    );
  });

  test('token vazio é resposta inválida, não credencial disponível', () async {
    client.onToken = () async => _result('   ');

    await expectLater(
      provider.getAccessTokenSilently(),
      throwsA(
        isA<IdentityException>().having(
          (error) => error.kind,
          'categoria',
          IdentityFailureKind.invalidResponse,
        ),
      ),
    );
  });

  test('limpeza de conta já ausente conclui sem solicitar logout', () async {
    client.accountFailure = const MsalNoCurrentAccountException(
      message: 'No account',
    );

    await provider.clearSession();

    expect(client.signOutCalls, 0);
  });

  test('limpeza exige confirmação do SDK e admite nova tentativa', () async {
    client.signOutResult = false;

    await expectLater(
      provider.clearSession(),
      throwsA(
        isA<IdentityException>().having(
          (error) => error.kind,
          'categoria',
          IdentityFailureKind.unexpectedExternalFailure,
        ),
      ),
    );

    client.signOutResult = true;

    await provider.clearSession();

    expect(client.signOutCalls, 2);
  });

  test('erro de programação não é convertido em falha operacional', () async {
    final error = StateError('Falha de programação');

    client.onToken = () async => throw error;

    await expectLater(provider.getAccessTokenSilently(), throwsA(same(error)));
  });

  test('modo múltiplo seleciona pelo ID e não remove contas alheias', () async {
    final multi = _MultiClient();
    final provider = MsalMicrosoftTokenProvider.forAccount(
      multi,
      accountId: 'a',
    );

    expect(await provider.getAccessTokenSilently(), 'graph-test-token');
    expect(multi.identifier, 'a');

    await provider.clearSession();

    expect(multi.removed, ['a']);
    expect(multi.accounts.map((a) => a.id), ['b']);
  });

  test('modo múltiplo sem conta exige interação sem escolher outra', () async {
    final multi = _MultiClient();
    final provider = MsalMicrosoftTokenProvider.forAccount(
      multi,
      accountId: 'ausente',
    );

    await expectLater(
      provider.getAccessTokenSilently(),
      throwsA(isA<MicrosoftInteractionRequiredException>()),
    );

    expect(multi.identifier, isNull);
    expect(multi.removed, isEmpty);
  });

  test(
    'identidade ausente no primeiro acesso não limpa o cache das outras contas',
    () async {
      final multi = _MultiClient();
      final provider = MsalMicrosoftTokenProvider.forAccount(
        multi,
        accountId: null,
      );

      await provider.clearSession();
      await expectLater(
        provider.getAccessTokenSilently(),
        throwsA(isA<MicrosoftInteractionRequiredException>()),
      );

      expect(multi.accounts, hasLength(2));
      expect(multi.removed, isEmpty);
    },
  );
}

/// Resposta sintética do SDK; o token não é interpretado como JWT pelo adaptador.
AuthenticationResult _result(String token) => AuthenticationResult(
  accessToken: token,
  authenticationScheme: 'Bearer',
  expiresOn: DateTime.utc(2026, 1, 1, 13),
  idToken: null,
  authority: 'https://login.microsoftonline.com/organizations',
  tenantId: null,
  scopes: const ['https://graph.microsoft.com/User.Read'],
  correlationId: null,
  account: _account(),
);

/// Conta do cache nativo, sem vínculo presumido com uma identidade Firebase.
Account _account() => Account(id: 'account-test', username: null, name: null);

/// Substitui apenas as operações usadas pelo adaptador, sem canal de plataforma.
class _MsalClient extends Fake implements SingleAccountPca {
  Future<AuthenticationResult> Function()? onToken;
  MsalException? accountFailure;
  bool signOutResult = true;
  int signOutCalls = 0;
  int interactiveCalls = 0;
  List<String>? requestedScopes;
  String? loginHint;
  Prompt? prompt;

  @override
  Future<AuthenticationResult> acquireTokenSilent({
    required List<String> scopes,
    String? identifier,
    String? authority,
  }) async {
    requestedScopes = scopes;

    return onToken == null ? _result('graph-test-token') : await onToken!();
  }

  @override
  Future<AuthenticationResult> acquireToken({
    required List<String> scopes,
    Prompt prompt = Prompt.whenRequired,
    String? loginHint,
    String? authority,
    CustomWebViewConfig? customWebViewConfig,
  }) async {
    interactiveCalls++;

    requestedScopes = scopes;
    this.loginHint = loginHint;
    this.prompt = prompt;

    return onToken == null ? _result('graph-test-token') : await onToken!();
  }

  @override
  Future<Account> get currentAccount async {
    if (accountFailure != null) throw accountFailure!;

    return _account();
  }

  @override
  Future<bool> signOut() async {
    signOutCalls++;

    return signOutResult;
  }
}

class _MultiClient extends Fake implements MultipleAccountPca {
  final accounts = [
    Account(id: 'a', username: null, name: null),
    Account(id: 'b', username: null, name: null),
  ];
  final removed = <String>[];
  String? identifier;

  @override
  Future<List<Account>> getAccounts() async => List.of(accounts);

  @override
  Future<bool> removeAccount({required String identifier}) async {
    removed.add(identifier);
    accounts.removeWhere((a) => a.id == identifier);

    return true;
  }

  @override
  Future<AuthenticationResult> acquireTokenSilent({
    required List<String> scopes,
    String? identifier,
    String? authority,
  }) async {
    this.identifier = identifier;

    return _result('graph-test-token');
  }
}
