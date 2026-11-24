/// Classificação das falhas de negócio e transporte do módulo de demandas.
library;

/// Categorias públicas de falha nas operações de demandas.
enum DemandFailureKind {
  /// Usuário não possui permissão no Centro Acadêmico ou curso para a ação.
  forbiddenCaForum,

  /// Usuário não possui vínculo estudantil ativo na instituição.
  studentMembershipRequired,

  /// Centro Acadêmico informado não foi encontrado.
  academicCenterNotFound,

  /// Centro Acadêmico informado está inativo ou arquivado.
  academicCenterInactive,

  /// Demanda solicitada não foi encontrada ou pertence a outro CA.
  demandNotFound,

  /// Dados informados violaram regra de negócio ou validação do backend.
  validationRejected,

  /// Credencial de autenticação ausente, inválida ou expirada.
  unauthorized,

  /// Verificação de autenticidade do aplicativo (App Check) falhou.
  appVerificationFailed,

  /// Falha de comunicação ou timeout com o servidor.
  communicationFailed,

  /// Serviço temporariamente indisponível.
  serviceUnavailable,

  /// Resposta do servidor incompatível com o contrato esperado.
  invalidResponse,

  /// Operação cancelada antes da conclusão.
  operationCancelled,

  /// Falha externa não classificada.
  unexpected,
}

/// Exceção da camada de aplicação para falhas em operações de demandas.
final class DemandException implements Exception {
  const DemandException({
    required this.kind,
    required this.cause,
    required this.stackTrace,
    this.backendCode,
    this.publicMessage,
  });

  final DemandFailureKind kind;
  final Object cause;
  final StackTrace stackTrace;
  final String? backendCode;
  final String? publicMessage;

  @override
  String toString() =>
      'DemandException(kind: $kind, backendCode: $backendCode, publicMessage: $publicMessage)';
}
