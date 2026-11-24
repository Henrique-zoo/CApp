/// Adaptador de infraestrutura que conecta [FaqGateway] ao cliente HTTP [FaqApi].
///
/// Converte DTOs de transporte em entidades de domínio e traduz falhas Dio,
/// de formato e de serialização checked para [FaqException].
library;

import 'package:capp/src/infrastructure/backend/auth/firebase_app_check_interceptor.dart';
import 'package:capp/src/infrastructure/backend/auth/firebase_bearer_interceptor.dart';
import 'package:capp/src/infrastructure/backend/error/api_error.dart';
import 'package:capp/src/infrastructure/backend/faq/faq.dart';
import 'package:dio/dio.dart';
import 'package:json_annotation/json_annotation.dart';
import 'package:uuid/uuid.dart';

import '../application/ports/faq_gateway.dart';
import '../domain/entities/faq_entry.dart';
import '../domain/errors/faq_exception.dart';

/// Implementa o acesso ao módulo de FAQ utilizando a API HTTP do backend CApp.
final class BackendFaqGateway implements FaqGateway {
  /// Cria o gateway com o cliente HTTP já configurado na composição.
  const BackendFaqGateway(this._api);

  final FaqApi _api;

  @override
  Future<List<FaqEntry>> listEntries({
    required UuidValue academicCenterId,
    String? category,
    String? query,
  }) => _request(() async {
    final responses = await _api.listEntries(
      academicCenterId: academicCenterId,
      category: category,
      query: query,
    );
    return responses.map(_mapEntry).toList();
  });

  @override
  Future<List<String>> listCategories({required UuidValue academicCenterId}) =>
      _request(() => _api.listCategories(academicCenterId: academicCenterId));

  @override
  Future<FaqEntry> getEntry({
    required UuidValue academicCenterId,
    required UuidValue entryId,
  }) => _request(
    () async => _mapEntry(
      await _api.getEntry(
        academicCenterId: academicCenterId,
        faqEntryId: entryId,
      ),
    ),
  );

  @override
  Future<FaqEntry> createEntry({
    required UuidValue academicCenterId,
    required FaqEntryDraft draft,
  }) => _request(
    () async => _mapEntry(
      await _api.createEntry(
        academicCenterId: academicCenterId,
        request: CreateFaqEntryRequest(
          question: draft.question,
          answer: draft.answer,
          category: draft.category,
        ),
      ),
    ),
  );

  @override
  Future<FaqEntry> updateEntry({
    required UuidValue academicCenterId,
    required UuidValue entryId,
    required FaqEntryDraft draft,
  }) => _request(
    () async => _mapEntry(
      await _api.updateEntry(
        academicCenterId: academicCenterId,
        faqEntryId: entryId,
        request: UpdateFaqEntryRequest(
          question: draft.question,
          answer: draft.answer,
          category: draft.category,
        ),
      ),
    ),
  );

  @override
  Future<void> deleteEntry({
    required UuidValue academicCenterId,
    required UuidValue entryId,
  }) => _request(
    () => _api.deleteEntry(
      academicCenterId: academicCenterId,
      faqEntryId: entryId,
    ),
  );

  FaqEntry _mapEntry(FaqEntryResponse dto) => FaqEntry(
    id: dto.id,
    academicCenterId: dto.academicCenterId,
    question: dto.question,
    answer: dto.answer,
    category: dto.category,
    createdBy: dto.createdBy,
    updatedBy: dto.updatedBy,
    createdAt: dto.createdAt,
    updatedAt: dto.updatedAt,
  );

  Future<T> _request<T>(Future<T> Function() operation) async {
    try {
      return await operation();
    } on DioException catch (error) {
      final failure = _translateTransportFailure(error);
      Error.throwWithStackTrace(failure, failure.stackTrace);
    } on FormatException catch (error, stackTrace) {
      Error.throwWithStackTrace(
        _invalidResponse(error, stackTrace),
        stackTrace,
      );
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
    }
  }

  FaqException _translateTransportFailure(DioException error) {
    return switch (error.type) {
      DioExceptionType.connectionTimeout ||
      DioExceptionType.sendTimeout ||
      DioExceptionType.receiveTimeout ||
      DioExceptionType.transformTimeout ||
      DioExceptionType.connectionError ||
      DioExceptionType.badCertificate => FaqException(
        kind: FaqFailureKind.communicationFailed,
        cause: error,
        stackTrace: error.stackTrace,
      ),
      DioExceptionType.cancel => FaqException(
        kind: FaqFailureKind.operationCancelled,
        cause: error,
        stackTrace: error.stackTrace,
      ),
      DioExceptionType.badResponse => _translateBackendFailure(error),
      DioExceptionType.unknown => _translateUnknownFailure(error),
    };
  }

  FaqException _translateBackendFailure(DioException error) {
    final backendError = _readBackendError(error).error;

    final kind = switch (backendError.code) {
      'app_check_required' ||
      'app_check_invalid' => FaqFailureKind.appVerificationFailed,
      'app_check_unavailable' ||
      'service_unavailable' => FaqFailureKind.serviceUnavailable,
      'unauthorized' ||
      'invalid_token' ||
      'expired_token' ||
      'account_not_provisioned' => FaqFailureKind.unauthorized,
      'user_not_permitted_in_ca' || 'forbidden' => FaqFailureKind.forbidden,
      'academic_center_not_found' => FaqFailureKind.academicCenterNotFound,
      'academic_center_inactive' => FaqFailureKind.academicCenterInactive,
      'faq_entry_not_found' => FaqFailureKind.entryNotFound,
      'empty_question' ||
      'question_too_long' ||
      'empty_answer' ||
      'answer_too_long' ||
      'empty_category' ||
      'category_too_long' ||
      'inconsistent_timestamps' ||
      'bad_request' => FaqFailureKind.validationError,
      _ => switch (error.response?.statusCode) {
        400 || 422 => FaqFailureKind.validationError,
        401 => FaqFailureKind.unauthorized,
        403 => FaqFailureKind.forbidden,
        404 => FaqFailureKind.entryNotFound,
        500 || 502 || 503 || 504 => FaqFailureKind.serviceUnavailable,
        _ => FaqFailureKind.unexpectedFailure,
      },
    };

    return FaqException(
      kind: kind,
      cause: error,
      stackTrace: error.stackTrace,
      backendCode: backendError.code,
      publicMessage: backendError.message,
    );
  }

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

  FaqException _translateUnknownFailure(DioException error) {
    final inner = error.error;
    if (inner is FaqException) return inner;

    if (inner is FirebaseAppCheckTokenException) {
      return FaqException(
        kind: FaqFailureKind.appVerificationFailed,
        cause: error,
        stackTrace: error.stackTrace,
      );
    }

    if (inner is MissingFirebaseIdTokenException) {
      return FaqException(
        kind: FaqFailureKind.unauthorized,
        cause: error,
        stackTrace: error.stackTrace,
      );
    }

    if (inner is FormatException) {
      return _invalidResponse(error, error.stackTrace);
    }

    if (inner != null && inner is! Exception) {
      Error.throwWithStackTrace(inner, error.stackTrace);
    }

    return FaqException(
      kind: FaqFailureKind.unexpectedFailure,
      cause: error,
      stackTrace: error.stackTrace,
    );
  }

  bool _isPayloadFailure(Object error) {
    if (error is CheckedFromJsonException) {
      final inner = error.innerError;
      return inner != null && _isPayloadFailure(inner);
    }
    return error is BadKeyException ||
        error is TypeError ||
        error is FormatException;
  }

  FaqException _invalidResponse(Object cause, StackTrace stackTrace) =>
      FaqException(
        kind: FaqFailureKind.invalidResponse,
        cause: cause,
        stackTrace: stackTrace,
      );
}
