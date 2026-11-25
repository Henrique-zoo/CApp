/// API de aplicação pública do módulo de demandas.
library;

export 'application/demands_controller.dart' show DemandsController;
export 'application/error/demand_exception.dart'
    show DemandException, DemandFailureKind;
export 'application/models/create_demand_input.dart'
    show
        CreateDemandInput,
        maxDemandCategoryLength,
        maxDemandCommentLength,
        maxDemandCourseNameLength,
        maxDemandDescriptionLength,
        maxDemandOpinionLength,
        maxDemandTitleLength;
export 'application/models/demand.dart' show Demand;
export 'application/models/demand_comment.dart'
    show DemandComment, DemandCommentAuthorRole;
export 'application/models/demand_forum.dart' show DemandForum;
export 'application/models/demand_status.dart' show DemandStatus;
export 'application/models/demand_status_history_entry.dart'
    show DemandStatusHistoryEntry;
export 'application/ports/demands_gateway.dart' show DemandsGateway;
