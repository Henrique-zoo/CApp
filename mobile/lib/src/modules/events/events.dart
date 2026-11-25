/// API pública da camada de aplicação do módulo de eventos.
library;

export 'application/event_failure.dart'
    show EventQueryException, EventQueryFailureKind;
export 'application/event_models.dart'
    show
        AcademicCenterContext,
        EventPublicationStatus,
        EventRegistrationRecord,
        EventRegistrationStatus,
        PublishedEvent;
export 'application/events_gateway.dart' show EventsGateway;
export 'application/events_schedule_controller.dart'
    show EventsScheduleController;
