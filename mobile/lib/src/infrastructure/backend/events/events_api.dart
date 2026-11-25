/// Cliente HTTP para os endpoints de listagem e detalhes de eventos do backend.
library;

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

import 'event_dto.dart';

/// Encapsula as chamadas HTTP de consulta da agenda pública de eventos.
final class EventsApi {
  const EventsApi(this._dio);

  final Dio _dio;

  /// Lista os eventos publicados do Centro Acadêmico ativo informado.
  ///
  /// Consome `GET /api/v1/academic-centers/{ca_id}/events` (compatível também
  /// com o cabeçalho `X-Academic-Center-Id`).
  Future<List<EventDto>> listPublishedEventsByCa(
    UuidValue academicCenterId,
  ) async {
    final response = await _dio.get<List<dynamic>>(
      '/api/v1/academic-centers/${academicCenterId.uuid}/events',
      queryParameters: const {'only_published': true},
      options: Options(
        headers: {'X-Academic-Center-Id': academicCenterId.uuid},
        followRedirects: false,
      ),
    );

    final data = response.data;
    if (data == null) {
      throw const FormatException('Resposta vazia na listagem de eventos.');
    }

    return data
        .map((item) {
          if (item is! Map<String, dynamic>) {
            throw const FormatException('Item de evento inválido na listagem.');
          }
          return EventDto.fromJson(item);
        })
        .toList(growable: false);
  }

  /// Obtém os detalhes completos de um evento publicado de um Centro Acadêmico específico.
  ///
  /// Consome `GET /api/v1/academic-centers/{ca_id}/events/{id}`.
  Future<EventDto> getPublishedEventDetail({
    required UuidValue academicCenterId,
    required UuidValue eventId,
  }) async {
    final response = await _dio.get<Map<String, dynamic>>(
      '/api/v1/academic-centers/${academicCenterId.uuid}/events/${eventId.uuid}',
      options: Options(
        headers: {'X-Academic-Center-Id': academicCenterId.uuid},
        followRedirects: false,
      ),
    );

    final data = response.data;
    if (data == null) {
      throw const FormatException('Resposta vazia nos detalhes do evento.');
    }

    return EventDto.fromJson(data);
  }
}
