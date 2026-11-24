/// Adaptador HTTP que implementa [DemandsGateway] sobre [DemandsApi].
library;

import 'package:dio/dio.dart';
import 'package:json_annotation/json_annotation.dart';
import 'package:uuid/uuid.dart';

import 'package:capp/src/infrastructure/backend/auth/firebase_app_check_interceptor.dart';
import 'package:capp/src/infrastructure/backend/auth/firebase_bearer_interceptor.dart';
import 'package:capp/src/infrastructure/backend/demands/demands.dart';
import 'package:capp/src/infrastructure/backend/error/api_error.dart';

import '../../application/error/demand_exception.dart';
import '../../application/models/create_demand_input.dart';
import '../../application/models/demand.dart';
import '../../application/models/demand_comment.dart';
import '../../application/models/demand_forum.dart';
import '../../application/models/demand_status.dart';
import '../../application/models/demand_status_history_entry.dart';
import '../../application/ports/demands_gateway.dart';

/// Implementação concreta de [DemandsGateway] integrada à API do CApp.
final class BackendDemandsGateway implements DemandsGateway {
  const BackendDemandsGateway(this._api);

  final DemandsApi _api;

  @override
  Future<List<Demand>> listDemands({
    required UuidValue academicCenterId,
    DemandForum? forum,
    DemandStatus? status,
    bool mine = false,
    int? limit,
    int? offset,
  }) => _request(() async {
    final dtos = await _api.listDemands(
      academicCenterId: academicCenterId,
      forum: forum?.wireValue,
      status: status?.wireValue,
      mine: mine ? true : null,
      limit: limit,
      offset: offset,
    );
    return dtos.map(_mapDemand).toList();
  });

  @override
  Future<Demand> createDemand({
    required UuidValue academicCenterId,
    required CreateDemandInput input,
  }) => _request(() async {
    final dto = await _api.createDemand(
      academicCenterId: academicCenterId,
      request: CreateDemandRequestDto(
        forum: input.forum.wireValue,
        description: input.description,
        title: input.title,
        category: input.category,
        courseName: input.courseName,
      ),
    );
    return _mapDemand(dto);
  });

  @override
  Future<Demand> getDemand({
    required UuidValue academicCenterId,
    required UuidValue demandId,
  }) => _request(() async {
    final dto = await _api.getDemand(
      academicCenterId: academicCenterId,
      demandId: demandId,
    );
    return _mapDemand(dto);
  });

  @override
  Future<Demand> updateDemandStatus({
    required UuidValue academicCenterId,
    required UuidValue demandId,
    required DemandStatus status,
    required String opinion,
  }) => _request(() async {
    final dto = await _api.updateDemandStatus(
      academicCenterId: academicCenterId,
      demandId: demandId,
      request: UpdateDemandStatusRequestDto(
        status: status.wireValue,
        opinion: opinion,
      ),
    );
    return _mapDemand(dto);
  });

  @override
  Future<List<DemandComment>> listComments({
    required UuidValue academicCenterId,
    required UuidValue demandId,
  }) => _request(() async {
    final dtos = await _api.listComments(
      academicCenterId: academicCenterId,
      demandId: demandId,
    );
    return dtos.map(_mapComment).toList();
  });

  @override
  Future<DemandComment> createComment({
    required UuidValue academicCenterId,
    required UuidValue demandId,
    required String content,
  }) => _request(() async {
    final dto = await _api.createComment(
      academicCenterId: academicCenterId,
      demandId: demandId,
      request: CreateDemandCommentRequestDto(content: content),
    );
    return _mapComment(dto);
  });

  Demand _mapDemand(DemandResponseDto dto) {
    final history =
        dto.statusHistory?.map(_mapHistoryEntry).toList() ?? const [];
    return Demand(
      id: dto.id,
      academicCenterId: dto.academicCenterId,
      authorId: dto.authorId,
      forum: DemandForum.fromWire(dto.forum),
      title: dto.title,
      description: dto.description,
      category: dto.category,
      courseName: dto.courseName,
      status: DemandStatus.fromWire(dto.status),
      createdAt: dto.createdAt,
      updatedAt: dto.updatedAt,
      statusHistory: history,
      commentCount: dto.commentCount,
    );
  }

  DemandStatusHistoryEntry _mapHistoryEntry(DemandStatusHistoryResponseDto dto) {
    return DemandStatusHistoryEntry(
      id: dto.id,
      demandId: dto.demandId,
      fromStatus: dto.fromStatus == null
          ? null
          : DemandStatus.fromWire(dto.fromStatus!),
      toStatus: DemandStatus.fromWire(dto.toStatus),
      changedById: dto.changedById,
      opinion: dto.opinion ?? dto.comment,
      createdAt: dto.createdAt,
    );
  }

  DemandComment _mapComment(DemandCommentResponseDto dto) {
    return DemandComment(
      id: dto.id,
      demandId: dto.demandId,
      authorId: dto.authorId,
      authorRole: DemandCommentAuthorRole.fromWire(dto.authorRole),
      content: dto.content,
      createdAt: dto.createdAt,
    );
  }

  Future<T> _request<T>(Future<T> Function() operation) async {
    try {
      return await operation();
    } on DioException catch (error) {
      final failure = _translateTransportFailure(error);
      Error.throwWithStackTrace(failure, failure.stackTrace);
    } on FormatException catch (error, stack) {
      Error.throwWithStackTrace(_invalidResponse(error, stack), stack);
    } on CheckedFromJsonException catch (error, stack) {
      if (!_isPayloadFailure(error)) {
        Error.throwWithStackTrace(
          error.innerError ?? error,
          error.innerStack ?? stack,
        );
      }
      Error.throwWithStackTrace(_invalidResponse(error, stack), stack);
    }
  }

  DemandException _translateTransportFailure(DioException error) {
    return switch (error.type) {
      DioExceptionType.connectionTimeout ||
      DioExceptionType.sendTimeout ||
      DioExceptionType.receiveTimeout ||
      DioExceptionType.transformTimeout ||
      DioExceptionType.connectionError ||
      DioExceptionType.badCertificate => DemandException(
        kind: DemandFailureKind.communicationFailed,
        cause: error,
        stackTrace: error.stackTrace,
      ),
      DioExceptionType.cancel => DemandException(
        kind: DemandFailureKind.operationCancelled,
        cause: error,
        stackTrace: error.stackTrace,
      ),
      DioExceptionType.badResponse => _translateBackendFailure(error),
      DioExceptionType.unknown => _translateUnknownFailure(error),
    };
  }

  DemandException _translateBackendFailure(DioException error) {
    final backendError = _readBackendError(error).error;
    final kind = switch (backendError.code) {
      'app_check_required' ||
      'app_check_invalid' => DemandFailureKind.appVerificationFailed,
      'app_check_unavailable' ||
      'service_unavailable' => DemandFailureKind.serviceUnavailable,
      'unauthorized' ||
      'invalid_token' ||
      'expired_token' ||
      'account_not_provisioned' => DemandFailureKind.unauthorized,
      'student_membership_required' =>
        DemandFailureKind.studentMembershipRequired,
      'user_not_permitted_in_ca' ||
      'user_not_found' ||
      'forbidden' => DemandFailureKind.forbiddenCaForum,
      'academic_center_not_found' => DemandFailureKind.academicCenterNotFound,
      'academic_center_inactive' => DemandFailureKind.academicCenterInactive,
      'demand_not_found' ||
      'demand_belongs_to_another_ca' => DemandFailureKind.demandNotFound,
      'bad_request' ||
      'invalid_forum' ||
      'invalid_status' ||
      'invalid_demand' ||
      'course_name_required' ||
      'course_not_represented_by_ca' ||
      'course_not_allowed_for_physical_space' ||
      'empty_description' ||
      'description_too_long' ||
      'empty_title' ||
      'title_too_long' ||
      'empty_category' ||
      'category_too_long' ||
      'empty_course_name' ||
      'course_name_too_long' ||
      'opinion_required' ||
      'empty_comment' ||
      'comment_too_long' ||
      'coordination_forwarding_not_allowed' ||
      'invalid_status_transition' => DemandFailureKind.validationRejected,
      _ => switch (error.response?.statusCode) {
        401 => DemandFailureKind.unauthorized,
        403 => DemandFailureKind.forbiddenCaForum,
        404 => DemandFailureKind.demandNotFound,
        400 || 422 => DemandFailureKind.validationRejected,
        500 || 502 || 503 || 504 => DemandFailureKind.serviceUnavailable,
        _ => DemandFailureKind.unexpected,
      },
    };

    return DemandException(
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

  DemandException _translateUnknownFailure(DioException error) {
    final inner = error.error;
    if (inner is DemandException) return inner;
    if (inner is FirebaseAppCheckTokenException) {
      return DemandException(
        kind: DemandFailureKind.appVerificationFailed,
        cause: error,
        stackTrace: error.stackTrace,
      );
    }
    if (inner is MissingFirebaseIdTokenException) {
      return DemandException(
        kind: DemandFailureKind.unauthorized,
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
    return DemandException(
      kind: DemandFailureKind.unexpected,
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

  DemandException _invalidResponse(Object cause, StackTrace stackTrace) {
    return DemandException(
      kind: DemandFailureKind.invalidResponse,
      cause: cause,
      stackTrace: stackTrace,
    );
  }
}
