/// Classificação de falhas operacionais das integrações de identidade.
///
/// Esta biblioteca fornece um vocabulário da aplicação para falhas originadas em
/// Firebase, Microsoft e backend. Assim, controlador e apresentação não precisam
/// importar SDKs, interpretar Dio ou conhecer o formato JSON de cada resposta.
///
/// ## Caminho da falha
///
/// Os adaptadores reconhecem falhas externas e produzem IdentityException com
/// uma IdentityFailureKind. O caso de uso propaga essa exceção; o controlador
/// associa o diagnóstico à operação e publica seu resumo no estado.
/// A apresentação transforma a classificação em mensagem ou usa publicMessage
/// quando uma mensagem foi explicitamente marcada como pública.
///
/// ## Classificação e diagnóstico
///
/// Cancelamento do login e cancelamento do transporte permanecem distintos.
/// Recusa de credenciais, instituição não cadastrada, resposta inválida e
/// indisponibilidade também mantêm categorias próprias. Vínculo não estudantil
/// é resultado de sucesso e não pertence a esta classificação.
///
/// A causa e a pilha originais são conservadas separadamente da mensagem pública.
/// O código do backend pode acompanhar a falha sem se tornar uma dependência da
/// apresentação. Erros internos de programação não devem ser indiscriminadamente
/// convertidos em indisponibilidade de um serviço.
library;

/// Classificação estável das falhas externas de identidade.
///
/// Os adaptadores traduzem os códigos de seus provedores para estas categorias.
/// Uma categoria não substitui o diagnóstico original.
enum IdentityFailureKind {
  /// O usuário cancelou a interação de autenticação.
  authenticationCancelled,

  /// Uma credencial foi rejeitada ou as identidades não corresponderam.
  credentialsRejected,

  /// A instituição não está habilitada para o fluxo de entrada.
  institutionNotSupported,

  /// Uma verificação concorrente impediu concluir esta tentativa.
  verificationConflict,

  /// A comunicação com uma dependência não pôde ser concluída.
  ///
  /// Não afirma, isoladamente, que o dispositivo está sem internet.
  communicationFailed,

  /// Uma dependência informou indisponibilidade temporária.
  serviceUnavailable,

  /// Uma resposta externa não correspondeu ao contrato esperado.
  invalidResponse,

  /// O provedor informou uma falha operacional sem classificação específica.
  ///
  /// Não deve ser usada para capturar indiscriminadamente erros de programação.
  unexpectedExternalFailure,

  /// Uma operação externa em andamento foi cancelada.
  ///
  /// Não implica cancelamento do login Microsoft, logout ou reversão
  /// de efeitos já concluídos no servidor.
  operationCancelled,
}

/// Falha operacional de identidade com diagnóstico preservado.
///
/// As operações das portas de identidade lançam este tipo para falhas externas
/// reconhecidas. Objetos da hierarquia [Error] não devem ser convertidos
/// indiscriminadamente nesta exceção.
///
/// A apresentação usa [kind] e, quando apropriado, [publicMessage].
/// Não deve apresentar ou registrar integralmente [cause], que pode conter
/// dados de requisições, credenciais ou informações pessoais.
final class IdentityException implements Exception {
  /// Cria uma falha classificada com sua causa e pilha originais.
  ///
  /// [kind] descreve a falha para tratamento pela aplicação.
  /// [cause] e [stackTrace] preservam o diagnóstico da integração.
  ///
  /// [backendCode] mantém um código público recebido do backend, quando houver.
  /// Não contém status HTTP e pode admitir códigos ainda desconhecidos.
  ///
  /// [publicMessage] deve conter somente uma mensagem explicitamente destinada
  /// à apresentação. Não deve receber automaticamente `cause.toString()`
  /// ou uma mensagem interna arbitrária de SDK.
  ///
  /// O construtor não sanitiza nem valida os textos fornecidos.
  const IdentityException({
    required this.kind,
    required this.cause,
    required this.stackTrace,
    this.backendCode,
    this.publicMessage,
  });

  /// Categoria utilizada pela aplicação e pela apresentação.
  final IdentityFailureKind kind;

  /// Falha original produzida pela integração.
  final Object cause;

  /// Pilha original capturada ao tratar a falha.
  final StackTrace stackTrace;

  /// Código público retornado pelo backend, quando disponível.
  final String? backendCode;

  /// Mensagem destinada à apresentação, quando disponível.
  final String? publicMessage;

  /// Descreve somente a classificação, sem incluir causa ou dados externos.
  @override
  String toString() {
    return 'IdentityException(${kind.name})';
  }
}
