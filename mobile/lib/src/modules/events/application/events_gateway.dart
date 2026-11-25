/// Porta de saída para consulta de eventos publicados e seus detalhes.
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
}
