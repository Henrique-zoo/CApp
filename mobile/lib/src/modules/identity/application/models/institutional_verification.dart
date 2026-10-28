/// Evidência institucional composta por perfil e prazo de validade.
///
/// A autenticação Firebase e a confirmação institucional expiram segundo regras
/// diferentes. Esta biblioteca representa a confirmação usada pela aplicação,
/// sem tratá-la como token de autenticação ou como autorização emitida ao cliente.
///
/// ## Construção e uso
///
/// O mapper do gateway ou o codec local produz InstitutionalVerification.
/// O construtor normaliza expiresAt para UTC; isValidAt compara o prazo com um
/// instante fornecido, sem consultar implicitamente o relógio.
///
/// SessionController decide quando avaliar essa comparação, publica a prontidão
/// e agenda a expiração. O armazenamento aceita também evidência expirada:
/// ler um registro compatível e decidir reutilizá-lo são responsabilidades
/// distintas.
///
/// ## Fronteiras
///
/// O modelo não conhece a origem dos dados e não possui JSON, timers ou
/// renovação automática. O prazo local auxilia a experiência de entrada,
/// mas não comprova que o backend aceitará uma requisição futura.
/// A conta correspondente é associada fora deste objeto.
library;

import 'institutional_profile.dart';

/// Resultado institucional com prazo de validade.
///
/// A aplicação pode conservar uma instância expirada para diagnóstico
/// ou apresentação. Sua existência não significa que a sessão esteja pronta
/// para entrar no aplicativo.
///
/// O instante de expiração é normalizado para UTC, preservando o momento
/// informado pelo chamador.
final class InstitutionalVerification {
  /// Cria a verificação com [profile] e [expiresAt].
  ///
  /// Aceita instantes passados porque o armazenamento pode conter
  /// verificações expiradas.
  ///
  /// Não confere a política de duração do backend nem autentica a pessoa.
  InstitutionalVerification({
    required this.profile,
    required DateTime expiresAt,
  }) : expiresAt = expiresAt.toUtc();

  /// Perfil obtido na verificação institucional.
  final InstitutionalProfile profile;

  /// Instante de expiração, normalizado para UTC.
  final DateTime expiresAt;

  /// Indica se a verificação está dentro do prazo no instante [instant].
  ///
  /// No instante exato da expiração, retorna falso.
  /// A comparação considera o momento representado, independentemente do fuso.
  ///
  /// O relógio é fornecido pelo chamador para permitir avaliação determinística.
  /// Este método não agenda atualizações e não consulta o servidor.
  ///
  /// O resultado auxilia a política local de entrada. Não garante que uma
  /// operação protegida será autorizada posteriormente pelo backend.
  bool isValidAt(DateTime instant) {
    return expiresAt.isAfter(instant);
  }

  /// Compara perfil e instante de expiração por conteúdo.
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        other is InstitutionalVerification &&
            other.profile == profile &&
            other.expiresAt == expiresAt;
  }

  /// Hash dos atributos utilizados pela igualdade.
  @override
  int get hashCode => Object.hash(profile, expiresAt);
}
