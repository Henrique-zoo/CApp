/// Operações, estado e modelos do módulo de FAQ consumidos pelo aplicativo.
library;

export 'application/faq_controller.dart' show FaqController;
export 'application/faq_state.dart' show FaqState;
export 'application/ports/faq_gateway.dart' show FaqGateway;
export 'domain/entities/faq_entry.dart'
    show
        FaqEntry,
        FaqEntryDraft,
        faqMaxAnswerLength,
        faqMaxCategoryLength,
        faqMaxQuestionLength,
        validateFaqAnswer,
        validateFaqCategory,
        validateFaqQuestion;
export 'domain/errors/faq_exception.dart'
    show FaqException, FaqFailureKind, faqFailureMessage;
