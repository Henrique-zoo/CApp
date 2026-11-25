/// Cliente HTTP dos endpoints de demandas nos Centros Acadêmicos.
///
/// Implementa as rotas descritas no OpenAPI (`/api/v1/academic-centers/...`)
/// utilizando o transporte compartilhado com Firebase Auth e App Check.
library;

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

import 'dto/demand_dto.dart';

/// Endpoints HTTP para listagem, criação, detalhes, status e comentários.
final class DemandsApi {
  const DemandsApi(this._dio);

  final Dio _dio;

  static const Map<String, Object?> _authExtra = {
    'firebaseAuth': true,
    'firebaseAppCheck': true,
  };

  Options _options({required int expectedStatus}) => Options(
    contentType: Headers.jsonContentType,
    responseType: ResponseType.json,
    followRedirects: false,
    receiveDataWhenStatusError: true,
    validateStatus: (status) => status == expectedStatus,
    extra: _authExtra,
  );

  /// Lista demandas do Centro Acadêmico com filtros opcionais.
  Future<List<DemandResponseDto>> listDemands({
    required UuidValue academicCenterId,
    String? forum,
    String? status,
    bool? mine,
    int? limit,
    int? offset,
    CancelToken? cancelToken,
  }) async {
    final caId = academicCenterId.uuid.toLowerCase();
    final response = await _dio.get<Object?>(
      '/api/v1/academic-centers/$caId/demands',
      queryParameters: {
        'forum': ?forum,
        'status': ?status,
        'mine': ?mine,
        'limit': ?limit,
        'offset': ?offset,
      },
      options: _options(expectedStatus: 200),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! List) {
      throw const FormatException('Listagem de demandas sem array JSON');
    }

    return data.map((item) {
      if (item is! Map<String, dynamic>) {
        throw const FormatException('Item de demanda inválido na listagem');
      }
      return DemandResponseDto.fromJson(item);
    }).toList();
  }

  /// Cria uma nova demanda no fórum institucional ou de espaço físico.
  Future<DemandResponseDto> createDemand({
    required UuidValue academicCenterId,
    required CreateDemandRequestDto request,
    CancelToken? cancelToken,
  }) async {
    final caId = academicCenterId.uuid.toLowerCase();
    final response = await _dio.post<Object?>(
      '/api/v1/academic-centers/$caId/demands',
      data: request.toJson(),
      options: _options(expectedStatus: 201),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! Map<String, dynamic>) {
      throw const FormatException('Resposta de criação sem objeto JSON');
    }
    return DemandResponseDto.fromJson(data);
  }

  /// Consulta os detalhes e histórico de status de uma demanda.
  Future<DemandResponseDto> getDemand({
    required UuidValue academicCenterId,
    required UuidValue demandId,
    CancelToken? cancelToken,
  }) async {
    final caId = academicCenterId.uuid.toLowerCase();
    final id = demandId.uuid.toLowerCase();
    final response = await _dio.get<Object?>(
      '/api/v1/academic-centers/$caId/demands/$id',
      options: _options(expectedStatus: 200),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! Map<String, dynamic>) {
      throw const FormatException('Detalhe de demanda sem objeto JSON');
    }
    return DemandResponseDto.fromJson(data);
  }

  /// Atualiza o status de uma demanda pela gestão do CA com parecer obrigatório.
  Future<DemandResponseDto> updateDemandStatus({
    required UuidValue academicCenterId,
    required UuidValue demandId,
    required UpdateDemandStatusRequestDto request,
    CancelToken? cancelToken,
  }) async {
    final caId = academicCenterId.uuid.toLowerCase();
    final id = demandId.uuid.toLowerCase();
    final response = await _dio.patch<Object?>(
      '/api/v1/academic-centers/$caId/demands/$id/status',
      data: request.toJson(),
      options: _options(expectedStatus: 200),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! Map<String, dynamic>) {
      throw const FormatException('Atualização de status sem objeto JSON');
    }
    return DemandResponseDto.fromJson(data);
  }

  /// Lista comentários e interações de uma demanda em ordem cronológica.
  Future<List<DemandCommentResponseDto>> listComments({
    required UuidValue academicCenterId,
    required UuidValue demandId,
    CancelToken? cancelToken,
  }) async {
    final caId = academicCenterId.uuid.toLowerCase();
    final id = demandId.uuid.toLowerCase();
    final response = await _dio.get<Object?>(
      '/api/v1/academic-centers/$caId/demands/$id/comments',
      options: _options(expectedStatus: 200),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! List) {
      throw const FormatException('Comentários da demanda sem array JSON');
    }

    return data.map((item) {
      if (item is! Map<String, dynamic>) {
        throw const FormatException('Item de comentário inválido');
      }
      return DemandCommentResponseDto.fromJson(item);
    }).toList();
  }

  /// Publica um comentário ou resposta na linha do tempo da demanda.
  Future<DemandCommentResponseDto> createComment({
    required UuidValue academicCenterId,
    required UuidValue demandId,
    required CreateDemandCommentRequestDto request,
    CancelToken? cancelToken,
  }) async {
    final caId = academicCenterId.uuid.toLowerCase();
    final id = demandId.uuid.toLowerCase();
    final response = await _dio.post<Object?>(
      '/api/v1/academic-centers/$caId/demands/$id/comments',
      data: request.toJson(),
      options: _options(expectedStatus: 201),
      cancelToken: cancelToken,
    );

    final data = response.data;
    if (data is! Map<String, dynamic>) {
      throw const FormatException('Resposta de comentário sem objeto JSON');
    }
    return DemandCommentResponseDto.fromJson(data);
  }
}
