/// Adaptação do protocolo HTTP ao caso de uso de identidade.
///
/// Esta biblioteca implementa a porta InstitutionalIdentityGateway na fronteira
/// do módulo. Conhece tanto IdentityApi e seus DTOs quanto os modelos e falhas
/// da aplicação, impedindo que detalhes do transporte atravessem essa fronteira.
///
/// ## Caminho de uma verificação
///
/// 1. Recebe o access token Microsoft do caso de uso e constrói a requisição.
/// 2. Chama IdentityApi; o transporte acrescenta o ID token Firebase.
/// 3. Converte o DTO de sucesso pelo mapper institucional.
/// 4. Devolve um modelo próprio, sem publicar sessão ou gravar cache.
///
/// O vínculo não estudantil continua sendo resposta bem-sucedida. Um HTTP 403
/// isolado não é reinterpretado como ausência de participação estudantil.
///
/// ## Tradução de falhas
///
/// Falhas de transporte, códigos públicos do backend e respostas incompatíveis
/// são classificados em IdentityException. A causa e a pilha são conservadas;
/// quando o envelope de erro também não puder ser lido, o diagnóstico preserva
/// a falha HTTP e a conversão.
///
/// A captura de desserialização é delimitada a problemas de conteúdo reconhecidos.
/// Erros internos não são indiscriminadamente ocultados como indisponibilidade.
/// Cancelamento HTTP permanece diferente de cancelamento da interação Microsoft.
///
/// ## Limites da fronteira
///
/// A classificação de negócio pertence a este adaptador, não ao cliente HTTP
/// compartilhado. O controlador continua responsável por conta vigente, prazo,
/// concorrência e efeito da falha sobre o destino apresentado.
library;

import 'package:dio/dio.dart';
import 'package:json_annotation/json_annotation.dart';

import 'package:capp/src/infrastructure/backend/auth/firebase_bearer_interceptor.dart';
import 'package:capp/src/infrastructure/backend/error/api_error.dart';
import 'package:capp/src/infrastructure/backend/identity/dto/institutional_auth_request.dart';
import 'package:capp/src/infrastructure/backend/identity/dto/institutional_verification_response.dart';
import 'package:capp/src/infrastructure/backend/identity/identity_api.dart';

import 'institutional_verification_mapper.dart';

import 'package:capp/src/modules/identity/application/errors/identity_exception.dart';
import 'package:capp/src/modules/identity/application/models/institutional_verification.dart';
import 'package:capp/src/modules/identity/application/ports/institutional_identity_gateway.dart';

/// Implementa a verificação institucional usando o backend CApp.
///
/// Recebe um cliente com transporte e autenticação já configurados.
/// Não assume seu encerramento, não mantém cache e não observa a sessão.
///
/// O resultado de vínculo não estudantil é convertido normalmente.
final class BackendInstitutionalIdentityGateway
    implements InstitutionalIdentityGateway {
  /// Usa o cliente HTTP fornecido pela composição.
  ///
  /// A construção não executa requisições ou consulta credenciais.
  const BackendInstitutionalIdentityGateway(this._api);

  final IdentityApi _api;

  /// Consulta o backend e devolve uma verificação própria da aplicação.
  ///
  /// [microsoftAccessToken] deve ser não vazio e pertencer à identidade
  /// utilizada na autenticação da requisição.
  ///
  /// Lança [ArgumentError] para credencial vazia, sem incluir seu valor
  /// no diagnóstico.
  ///
  /// Falhas reconhecidas de transporte ou resposta são traduzidas para
  /// [IdentityException]. Erros inesperados de programação continuam
  /// propagando, inclusive quando encapsulados pelo transporte.
  ///
  /// A conversão do modelo ocorre depois da leitura do DTO. Não avalia
  /// expiração, persiste dados ou decide navegação.
  @override
  Future<InstitutionalVerification> verify({
    required String microsoftAccessToken,
  }) async {
    if (microsoftAccessToken.isEmpty) {
      throw ArgumentError('O access token Microsoft não pode ser vazio');
    }

    final InstitutionalVerificationResponse response;

    try {
      response = await _api.verifyInstitutionalIdentity(
        request: InstitutionalAuthRequest(
          microsoftAccessToken: microsoftAccessToken,
        ),
      );
    } on DioException catch (error) {
      final failure = _translateTransportFailure(error);

      Error.throwWithStackTrace(failure, failure.stackTrace);
    } on CheckedFromJsonException catch (error, stackTrace) {
      if (!_isPayloadFailure(error)) {
        Error.throwWithStackTrace(
          error.innerError ?? error,
          error.innerStack ?? stackTrace,
        );
      }

      Error.throwWithStackTrace(
        _invalidResponse(error, stackTrace),
        stackTrace,
      );
    } on FormatException catch (error, stackTrace) {
      Error.throwWithStackTrace(
        _invalidResponse(error, stackTrace),
        stackTrace,
      );
    }

    return mapInstitutionalVerification(response);
  }

  /// Traduz a categoria informada pelo transporte.
  ///
  /// Timeouts e falhas de conexão não afirmam que o dispositivo está offline.
  /// Cancelamento HTTP possui categoria distinta do cancelamento do login.
  IdentityException _translateTransportFailure(DioException error) {
    return switch (error.type) {
      DioExceptionType.connectionTimeout ||
      DioExceptionType.sendTimeout ||
      DioExceptionType.receiveTimeout ||
      DioExceptionType.transformTimeout ||
      DioExceptionType.connectionError ||
      DioExceptionType.badCertificate => _transportFailure(
        IdentityFailureKind.communicationFailed,
        error,
      ),
      DioExceptionType.cancel => _transportFailure(
        IdentityFailureKind.operationCancelled,
        error,
      ),
      DioExceptionType.badResponse => _translateBackendFailure(error),
      DioExceptionType.unknown => _translateUnknownFailure(error),
    };
  }

  /// Interpreta uma resposta HTTP de erro conforme o envelope do backend.
  ///
  /// Códigos conhecidos possuem prioridade sobre a classificação genérica
  /// do status. Um HTTP 403 isolado não é interpretado como ausência
  /// de vínculo estudantil.
  ///
  /// Corpo incompatível com o contrato produz falha de resposta inválida,
  /// preservando os diagnósticos disponíveis.
  IdentityException _translateBackendFailure(DioException error) {
    final backendError = _readBackendError(error).error;

    final kind = switch (backendError.code) {
      'unauthorized' ||
      'invalid_token' ||
      'expired_token' ||
      'invalid_microsoft_id' ||
      'identity_mismatch' => IdentityFailureKind.credentialsRejected,
      'unknown_institution' => IdentityFailureKind.institutionNotSupported,
      'institutional_verification_required' =>
        IdentityFailureKind.verificationConflict,
      'authentication_provider_unavailable' ||
      'service_unavailable' => IdentityFailureKind.serviceUnavailable,
      _ => switch (error.response?.statusCode) {
        401 => IdentityFailureKind.credentialsRejected,
        502 || 503 || 504 => IdentityFailureKind.serviceUnavailable,
        _ => IdentityFailureKind.unexpectedExternalFailure,
      },
    };

    return IdentityException(
      kind: kind,
      cause: error,
      stackTrace: error.stackTrace,
      backendCode: backendError.code,
      publicMessage: backendError.message,
    );
  }

  /// Lê o envelope público de uma resposta rejeitada pelo transporte.
  ///
  /// Se a conversão falhar por dados incompatíveis, preserva tanto
  /// a falha HTTP quanto a falha de desserialização em um registro diagnóstico.
  /// Nenhum desses erros é convertido em ausência de resposta.
  ///
  /// Falhas internas não reconhecidas como dados inválidos são repropagadas.
  ApiError _readBackendError(DioException transportError) {
    final data = transportError.response?.data;

    if (data is! Map<String, dynamic>) {
      final failure = _invalidResponse(
        transportError,
        transportError.stackTrace,
      );

      Error.throwWithStackTrace(failure, failure.stackTrace);
    }

    try {
      return ApiError.fromJson(data);
    } on CheckedFromJsonException catch (error, stackTrace) {
      if (!_isPayloadFailure(error)) {
        Error.throwWithStackTrace(
          error.innerError ?? error,
          error.innerStack ?? stackTrace,
        );
      }

      final failure = _invalidResponse((
        transport: transportError,
        decoding: error,
        decodingStackTrace: stackTrace,
      ), stackTrace);

      Error.throwWithStackTrace(failure, stackTrace);
    }
  }

  /// Examina falhas que Dio não classificou em uma categoria específica.
  ///
  /// Preserva uma IdentityException já produzida por outra fronteira.
  /// Reconhece ausência de credencial e JSON malformado.
  ///
  /// Objetos encapsulados que não implementam Exception, incluindo Error,
  /// são repropagados com a pilha preservada.
  IdentityException _translateUnknownFailure(DioException error) {
    final inner = error.error;

    if (inner is IdentityException) return inner;

    if (inner is MissingFirebaseIdTokenException) {
      return _transportFailure(IdentityFailureKind.credentialsRejected, error);
    }

    if (inner is FormatException) {
      return _invalidResponse(error, error.stackTrace);
    }

    if (inner != null && inner is! Exception) {
      Error.throwWithStackTrace(inner, error.stackTrace);
    }

    return _transportFailure(
      IdentityFailureKind.unexpectedExternalFailure,
      error,
    );
  }

  /// Reconhece falhas de campos, tipos e formatos dentro da desserialização.
  ///
  /// TypeError é aceito somente neste contexto delimitado de leitura dos DTOs.
  /// Outros erros encapsulados pelo gerador não são classificados
  /// indiscriminadamente como dados inválidos.
  bool _isPayloadFailure(Object error) {
    if (error is CheckedFromJsonException) {
      final inner = error.innerError;

      return inner != null && _isPayloadFailure(inner);
    }

    return error is BadKeyException ||
        error is TypeError ||
        error is FormatException;
  }

  /// Preserva a falha de transporte sob uma classificação da aplicação.
  IdentityException _transportFailure(
    IdentityFailureKind kind,
    DioException error,
  ) {
    return IdentityException(
      kind: kind,
      cause: error,
      stackTrace: error.stackTrace,
    );
  }

  /// Representa uma resposta externa incompatível com o contrato.
  ///
  /// [cause] pode conter o diagnóstico de transporte, de conversão ou ambos.
  /// Nenhuma mensagem técnica é marcada automaticamente como pública.
  IdentityException _invalidResponse(Object cause, StackTrace stackTrace) {
    return IdentityException(
      kind: IdentityFailureKind.invalidResponse,
      cause: cause,
      stackTrace: stackTrace,
    );
  }
}
