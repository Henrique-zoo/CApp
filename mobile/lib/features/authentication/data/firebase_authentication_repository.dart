import 'package:dio/dio.dart';
import 'package:firebase_auth/firebase_auth.dart';
import 'package:flutter/foundation.dart';

import '../domain/authenticated_identity.dart';
import '../domain/authentication_repository.dart';

/// Configuração pública necessária para integrar o aplicativo ao ambiente.
///
/// Os valores são fornecidos no build com `--dart-define`; nenhum segredo do
/// cliente OAuth deve ser incluído no aplicativo.
class AuthenticationConfig {
  const AuthenticationConfig({
    required this.apiBaseUrl,
    required this.microsoftTenantId,
  });

  /// Configuração carregada do ambiente de compilação.
  const AuthenticationConfig.fromEnvironment()
    : apiBaseUrl = const String.fromEnvironment('CAPP_API_BASE_URL'),
      microsoftTenantId = const String.fromEnvironment(
        'CAPP_MICROSOFT_TENANT_ID',
      );

  /// URL base da API CApp, sem caminho de recurso.
  final String apiBaseUrl;

  /// ID GUID ou domínio do tenant institucional Microsoft.
  final String microsoftTenantId;
}

/// Repositório que integra Firebase Authentication e a API do CApp.
class FirebaseAuthenticationRepository implements AuthenticationRepository {
  FirebaseAuthenticationRepository({
    FirebaseAuth? firebaseAuth,
    Dio? client,
    AuthenticationConfig? config,
  }) : _firebaseAuthOverride = firebaseAuth,
       _client =
           client ??
           Dio(
             BaseOptions(
               connectTimeout: const Duration(seconds: 8),
               receiveTimeout: const Duration(seconds: 8),
             ),
           ),
       _config = config ?? const AuthenticationConfig.fromEnvironment();

  final FirebaseAuth? _firebaseAuthOverride;
  final Dio _client;
  final AuthenticationConfig _config;

  // O Firebase só é acessado quando uma operação de sessão é executada.
  FirebaseAuth get _firebaseAuth =>
      _firebaseAuthOverride ?? FirebaseAuth.instance;

  @override
  Future<AuthenticatedIdentity?> restoreSession() async {
    final user = _firebaseAuth.currentUser;
    if (user == null) return null;

    try {
      _validateConfiguration();
      return await _fetchBackendIdentity(user);
    } on AuthenticationFailure {
      await _signOutAfterFailure();
      rethrow;
    } on FirebaseAuthException {
      await _signOutAfterFailure();
      throw const AuthenticationFailure(
        AuthenticationFailureReason.serviceUnavailable,
        'Não foi possível restaurar a sessão. Entre novamente.',
      );
    } catch (_) {
      await _signOutAfterFailure();
      throw const AuthenticationFailure(
        AuthenticationFailureReason.serviceUnavailable,
        'Não foi possível validar a sessão. Tente entrar novamente.',
      );
    }
  }

  @override
  Future<AuthenticatedIdentity> signIn() async {
    _validateConfiguration();

    try {
      final provider = MicrosoftAuthProvider();
      provider.setCustomParameters({
        'tenant': _config.microsoftTenantId.trim(),
      });

      final UserCredential credential;
      if (kIsWeb) {
        credential = await _firebaseAuth.signInWithPopup(provider);
      } else {
        credential = await _firebaseAuth.signInWithProvider(provider);
      }

      final user = credential.user;
      if (user == null) {
        throw const AuthenticationFailure(
          AuthenticationFailureReason.unknown,
          'A Microsoft não retornou uma identidade autenticada. Tente novamente.',
        );
      }

      return await _fetchBackendIdentity(user);
    } on AuthenticationFailure {
      await _signOutAfterFailure();
      rethrow;
    } on FirebaseAuthException catch (error) {
      await _signOutAfterFailure();

      if (_isCancellation(error.code)) {
        throw const AuthenticationFailure(
          AuthenticationFailureReason.cancelled,
          'Autenticação cancelada.',
        );
      }
      if (error.code == 'user-disabled') {
        throw const AuthenticationFailure(
          AuthenticationFailureReason.institutionalAccountRejected,
          'A conta não está autorizada a acessar o CApp. Use sua conta institucional da UnB.',
        );
      }
      throw const AuthenticationFailure(
        AuthenticationFailureReason.unknown,
        'Não foi possível concluir o login institucional. Confira a conta e tente novamente.',
      );
    } catch (_) {
      await _signOutAfterFailure();
      throw const AuthenticationFailure(
        AuthenticationFailureReason.unknown,
        'Não foi possível concluir o login institucional. Tente novamente.',
      );
    }
  }

  @override
  Future<void> signOut() async {
    await _firebaseAuth.signOut();
  }

  void _validateConfiguration() {
    if (_config.microsoftTenantId.trim().isEmpty) {
      throw const AuthenticationFailure(
        AuthenticationFailureReason.configuration,
        'O tenant institucional Microsoft não está configurado neste ambiente.',
      );
    }

    final uri = Uri.tryParse(_config.apiBaseUrl.trim());
    if (uri == null ||
        !uri.hasAuthority ||
        uri.host.isEmpty ||
        !{'http', 'https'}.contains(uri.scheme) ||
        uri.userInfo.isNotEmpty ||
        uri.hasQuery ||
        uri.hasFragment) {
      throw const AuthenticationFailure(
        AuthenticationFailureReason.configuration,
        'A URL da API não está configurada corretamente neste ambiente.',
      );
    }

    if (kReleaseMode && uri.scheme != 'https') {
      throw const AuthenticationFailure(
        AuthenticationFailureReason.configuration,
        'O aplicativo exige uma URL HTTPS da API em builds de produção.',
      );
    }
  }

  Future<AuthenticatedIdentity> _fetchBackendIdentity(User user) async {
    final token = await user.getIdToken();
    if (token == null || token.isEmpty) {
      throw const AuthenticationFailure(
        AuthenticationFailureReason.institutionalAccountRejected,
        'Não foi possível obter uma sessão válida. Entre novamente.',
      );
    }

    final identityUri = Uri.parse(_config.apiBaseUrl.trim())
        .resolve('/identity/me');

    try {
      final response = await _client.getUri<dynamic>(
        identityUri,
        options: Options(
          headers: {'Authorization': 'Bearer $token'},
          responseType: ResponseType.json,
        ),
      );

      final data = response.data;
      if (data is! Map) {
        throw const FormatException('Resposta de identidade inválida.');
      }

      final json = data.map<String, dynamic>(
        (key, value) => MapEntry(key.toString(), value),
      );
      return AuthenticatedIdentity.fromJson(json);
    } on DioException catch (error) {
      // Nunca registra o token nem detalhes sensíveis da resposta.
      final statusCode = error.response?.statusCode;
      if (statusCode == 401 || statusCode == 403) {
        throw const AuthenticationFailure(
          AuthenticationFailureReason.institutionalAccountRejected,
          'Não foi possível confirmar uma identidade institucional autorizada. Use sua conta institucional da UnB.',
        );
      }

      throw const AuthenticationFailure(
        AuthenticationFailureReason.serviceUnavailable,
        'O serviço de autenticação está temporariamente indisponível. Tente novamente.',
      );
    } on FormatException {
      throw const AuthenticationFailure(
        AuthenticationFailureReason.serviceUnavailable,
        'O serviço de autenticação retornou uma resposta inválida. Tente novamente.',
      );
    }
  }

  Future<void> _signOutAfterFailure() async {
    try {
      await _firebaseAuth.signOut();
    } catch (_) {
      throw const AuthenticationFailure(
        AuthenticationFailureReason.sessionOperationFailed,
        'Não foi possível limpar a sessão local. Feche o aplicativo e tente novamente.',
      );
    }
  }

  bool _isCancellation(String code) => const {
    'web-context-canceled',
    'popup-closed-by-user',
    'cancelled-popup-request',
    'sign_in_canceled',
    'user-cancelled',
  }.contains(code);
}
