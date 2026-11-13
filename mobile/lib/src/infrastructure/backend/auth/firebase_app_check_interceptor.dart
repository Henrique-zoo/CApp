/// Comprovação do aplicativo nas requisições ao backend personalizado.
///
/// O endpoint marca extra['firebaseAppCheck'] com true. O transporte consulta o
/// SDK por uma função injetada e envia X-Firebase-AppCheck, separadamente do
/// Authorization que identifica a pessoa. Health e ready não usam esse marcador.
/// O SDK possui o cache e a renovação; esta biblioteca não conserva credenciais,
/// registra tokens, repete requisições ou abre interfaces de autenticação.
///
/// Token ausente ou falha operacional do provedor impede o envio. O adaptador
/// do módulo traduz esse diagnóstico sem confundi-lo com uma senha incorreta.
/// Erros de programação permanecem identificáveis na causa do DioException.
library;

import 'package:dio/dio.dart';

/// Obtém o atestado atual do SDK; nulo significa que não há token disponível.
typedef FirebaseAppCheckTokenProvider = Future<String?> Function();

/// Acrescenta comprovação somente nas operações explicitamente marcadas.
final class FirebaseAppCheckInterceptor extends Interceptor {
  FirebaseAppCheckInterceptor(this._provideToken);

  final FirebaseAppCheckTokenProvider _provideToken;

  @override
  void onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    if (options.extra['firebaseAppCheck'] != true) {
      handler.next(options);

      return;
    }

    try {
      final token = await _provideToken();

      if (token == null || token.isEmpty) {
        throw const FirebaseAppCheckTokenException();
      }

      options.headers['X-Firebase-AppCheck'] = token;

      handler.next(options);
    } catch (error, stack) {
      handler.reject(
        DioException(
          requestOptions: options,
          type: DioExceptionType.unknown,
          error: error is FirebaseAppCheckTokenException || error is! Exception
              ? error
              : FirebaseAppCheckTokenException(cause: error),
          stackTrace: stack,
        ),
      );
    }
  }
}

/// Falha de atestado, independente da identidade e da senha do usuário.
final class FirebaseAppCheckTokenException implements Exception {
  const FirebaseAppCheckTokenException({this.cause});

  /// Diagnóstico original; não deve ser apresentado ou registrado integralmente.
  final Object? cause;

  @override
  String toString() => 'Não foi possível obter a comprovação do aplicativo';
}
