/// Resultado observável da consulta de prontidão feita na abertura do CApp.
///
/// Pertence à camada app porque descreve um diagnóstico global da execução,
/// independente da identidade do usuário. A apresentação pode informar uma falha
/// sem alterar o destino calculado pelo controlador de sessão.
///
/// Os estados distinguem ausência de consulta, operação em andamento, resposta
/// positiva e falha com diagnóstico. Uma falha não é convertida em ausência de
/// resultado: conserva a exceção e sua pilha para investigação.
///
/// Uma resposta positiva descreve apenas o instante consultado. Não garante
/// disponibilidade posterior, autorização ou funcionamento dos provedores de
/// autenticação. Estas classes não consultam rede nem modificam a sessão.
library;

/// Fotografia da última verificação de prontidão desta execução.
sealed class BackendReadinessState {
  /// Permite construir somente as variantes declaradas nesta biblioteca.
  const BackendReadinessState();
}

/// A composição ainda não solicitou uma consulta.
final class BackendReadinessNotChecked extends BackendReadinessState {
  /// Representa ausência de resultado, sem presumir disponibilidade.
  const BackendReadinessNotChecked();
}

/// Existe uma consulta em andamento, inicial ou solicitada pelo usuário.
final class BackendReadinessChecking extends BackendReadinessState {
  /// Representa a operação atual, sem reutilizar o resultado anterior.
  const BackendReadinessChecking();
}

/// O endpoint confirmou prontidão na última consulta concluída.
final class BackendReadinessReady extends BackendReadinessState {
  /// Registra sucesso pontual, sem prometer disponibilidade futura.
  const BackendReadinessReady();
}

/// Não foi possível confirmar prontidão na última consulta.
///
/// Pode representar HTTP 503, falha de rede ou outra rejeição do endpoint.
/// Não equivale a uma conclusão sobre a conexão local do dispositivo.
final class BackendReadinessFailed extends BackendReadinessState {
  /// Preserva a exceção recebida e a pilha associada à tentativa.
  const BackendReadinessFailed({required this.cause, required this.stackTrace});

  /// Diagnóstico original; não deve ser apresentado diretamente ao usuário.
  final Exception cause;

  /// Pilha original da falha, conservada para diagnóstico.
  final StackTrace stackTrace;
}
