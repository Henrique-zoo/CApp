/// Porta de saída para consulta de eventos publicados, detalhes e gestão de inscrição.
library;

import 'package:uuid/uuid.dart';

import 'event_models.dart';

/// Contrato de comunicação da camada de aplicação com a fonte de dados de eventos.
abstract interface class EventsGateway {
  /// Lista os eventos publicados do Centro Acadêmico identificado por [academicCenterId].
  Future<List<PublishedEvent>> listPublishedByAcademicCenter(
    UuidValue academicCenterId,
  );

  /// Busca as informações detalhadas de um evento publicado específico em [academicCenterId].
  Future<PublishedEvent> getPublishedDetail({
    required UuidValue academicCenterId,
    required UuidValue eventId,
  });

  /// Realiza a inscrição do participante autenticado no evento [eventId] do CA [academicCenterId].
  Future<EventRegistrationRecord> registerInEvent({
    required UuidValue academicCenterId,
    required UuidValue eventId,
  });

  /// Cancela a inscrição ativa do participante autenticado no evento [eventId] do CA [academicCenterId].
  Future<EventRegistrationRecord> cancelRegistration({
    required UuidValue academicCenterId,
    required UuidValue eventId,
  });
}
