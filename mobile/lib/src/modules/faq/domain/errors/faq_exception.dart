/// Classificação de falhas e exceção pública do domínio de FAQ.
library;

/// Categorias de falha reconhecidas nas operações de consulta e gestão de FAQ.
enum FaqFailureKind {
  communicationFailed,
  operationCancelled,
  unauthorized,
  forbidden,
  academicCenterNotFound,
  academicCenterInactive,
  entryNotFound,
  validationError,
  appVerificationFailed,
  serviceUnavailable,
  invalidResponse,
  unexpectedFailure,
}

/// Exceção de domínio produzida pela fronteira de acesso aos dados do FAQ.
final class FaqException implements Exception {
  /// Cria a falha preservando causa, pilha e eventual mensagem pública do backend.
  const FaqException({
    required this.kind,
    required this.cause,
    required this.stackTrace,
    this.backendCode,
    this.publicMessage,
  });

  /// Categoria semântica da falha.
  final FaqFailureKind kind;

  /// Causa original preservada para diagnóstico.
  final Object cause;

  /// Pilha original da ocorrência.
  final StackTrace stackTrace;

  /// Código programático retornado pelo backend quando disponível.
  final String? backendCode;

  /// Mensagem pública fornecida pelo backend quando disponível.
  final String? publicMessage;
}

/// Traduz [error] para uma mensagem amigável exibível ao usuário.
String faqFailureMessage(FaqException error) => switch (error.kind) {
  FaqFailureKind.communicationFailed =>
    'Não foi possível conectar ao servidor para consultar o FAQ. Verifique sua conexão e tente novamente.',
  FaqFailureKind.operationCancelled => 'A consulta ao FAQ foi cancelada.',
  FaqFailureKind.unauthorized =>
    'Sua sessão expirou ou não pôde ser confirmada. Entre novamente para consultar o FAQ.',
  FaqFailureKind.forbidden =>
    'Você não possui permissão de gestão neste Centro Acadêmico.',
  FaqFailureKind.academicCenterNotFound =>
    'O Centro Acadêmico selecionado não foi encontrado.',
  FaqFailureKind.academicCenterInactive =>
    'O Centro Acadêmico selecionado está inativo no momento.',
  FaqFailureKind.entryNotFound =>
    'A pergunta frequente selecionada não foi encontrada.',
  FaqFailureKind.validationError =>
    error.publicMessage ??
        'Os dados informados para a pergunta frequente são inválidos.',
  FaqFailureKind.appVerificationFailed =>
    'Não foi possível comprovar a integridade do aplicativo para consultar o FAQ.',
  FaqFailureKind.serviceUnavailable =>
    'O serviço de FAQ está temporariamente indisponível. Tente novamente em instantes.',
  FaqFailureKind.invalidResponse =>
    'Recebemos uma resposta inesperada do servidor ao consultar o FAQ.',
  FaqFailureKind.unexpectedFailure =>
    'Ocorreu um erro inesperado ao consultar o FAQ. Tente novamente.',
};
