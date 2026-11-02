/// Resultados da leitura local usados pela coordenação da sessão.
///
/// Uma ausência de registro, um conteúdo incompatível e uma falha de acesso
/// podem levar à mesma tela de entrada, mas possuem causas e possibilidades de
/// recuperação distintas. Esta união preserva essa diferença no contrato da
/// aplicação, sem expor preferências ou exceções específicas do plugin à interface.
///
/// ## Produção e consumo
///
/// SessionStore retorna SessionCacheFound com o modelo reconstruído,
/// SessionCacheMissing quando a chave não existe, SessionCacheRejected para
/// conteúdo inutilizável e SessionCacheReadFailed quando a leitura falha.
/// O codec determina rejeições de conteúdo; o adaptador determina ausência
/// e falhas ao acessar o meio de armazenamento.
///
/// SessionController conserva o resultado no estado e decide se a verificação
/// encontrada permite entrada. Encontrado não significa válido no instante atual:
/// registros expirados continuam sendo resultados encontrados.
///
/// ## Diagnóstico e efeitos
///
/// Rejeições podem conservar a exceção de formato e sua pilha. Rejeições por
/// comparação possuem o motivo, e falhas de acesso conservam seu diagnóstico.
/// Nenhum dos tipos remove registros, executa migração ou autentica a conta.
/// Erros de programação não devem ser ocultados como ausência de cache.
///
/// ## Exemplo
///
/// ```dart
/// String describe(SessionCacheLoadResult result) => switch (result) {
///   SessionCacheFound() => 'encontrado',
///   SessionCacheMissing() => 'ausente',
///   SessionCacheRejected(:final reason) => 'rejeitado: ${reason.name}',
///   SessionCacheReadFailed() => 'falha de acesso',
/// };
/// assert(describe(const SessionCacheMissing()) == 'ausente');
/// ```
library;

import 'models/institutional_verification.dart';

/// Resultado da tentativa de recuperar uma verificação local.
///
/// Não carrega uma decisão de autorização. A validade temporal de um resultado
/// encontrado também é avaliada fora do armazenamento.
sealed class SessionCacheLoadResult {
  /// Permite construir as variantes pertencentes a esta biblioteca.
  const SessionCacheLoadResult();
}

/// Verificação recuperada do armazenamento local.
///
/// O registro pode estar expirado. Sua recuperação não comprova autenticação
/// nem autorização atual no servidor.
final class SessionCacheFound extends SessionCacheLoadResult {
  /// Preserva a verificação recuperada e seus campos opcionais.
  const SessionCacheFound(this.verification);

  /// Última verificação armazenada para o UID solicitado.
  final InstitutionalVerification verification;
}

/// Não existe registro para o UID solicitado.
final class SessionCacheMissing extends SessionCacheLoadResult {
  /// Representa inexistência do registro, sem indicar falha de leitura.
  const SessionCacheMissing();
}

/// Motivo da rejeição de um registro existente.
enum SessionCacheRejectionReason {
  /// A preferência existe, mas não contém uma string JSON.
  unexpectedValueType,

  /// A string não pôde ser interpretada como JSON.
  malformedJson,

  /// A raiz, o UID ou a versão não possuem a estrutura obrigatória.
  invalidEnvelope,

  /// A versão é um inteiro válido, mas não é suportada pelo leitor atual.
  unsupportedVersion,

  /// O UID dentro do envelope difere do UID solicitado.
  ownerMismatch,

  /// O objeto de verificação está ausente ou não corresponde ao contrato JSON.
  invalidVerification,
}

/// Registro existente que não pode ser reutilizado.
///
/// Não é uma falha de acesso ao armazenamento. Rejeições estruturais podem ser
/// detectadas por comparação, sem exceção; nesse caso causa e pilha são ausentes.
/// A criação deste resultado não remove nem corrige o registro.
final class SessionCacheRejected extends SessionCacheLoadResult {
  /// Preserva [reason] e, quando existente, o diagnóstico da conversão.
  const SessionCacheRejected(this.reason, {this.cause, this.stackTrace});

  /// Motivo estável para recuperação e diagnóstico pela aplicação.
  final SessionCacheRejectionReason reason;

  /// Exceção original, quando a rejeição veio de uma conversão.
  /// Pode conter dados do JSON e não deve ser exibida ou registrada integralmente
  /// sem considerar o conteúdo pessoal presente no cache.
  final Object? cause;

  /// Pilha original, quando houver uma exceção.
  final StackTrace? stackTrace;
}

/// Falha de acesso ao armazenamento.
///
/// O leitor não conseguiu determinar se havia um registro utilizável. Difere
/// tanto da inexistência quanto da confirmação de conteúdo inválido.
final class SessionCacheReadFailed extends SessionCacheLoadResult {
  /// Preserva a exceção operacional e sua pilha original.
  const SessionCacheReadFailed({required this.cause, required this.stackTrace});

  /// Exceção produzida pelo acesso às preferências.
  final Exception cause;

  /// Pilha recebida durante a captura da falha.
  final StackTrace stackTrace;
}
