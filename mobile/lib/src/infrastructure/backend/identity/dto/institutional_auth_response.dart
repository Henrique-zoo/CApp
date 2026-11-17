/// Resposta de uma verificação institucional concluída com conta provisionada.
///
/// Compartilha os DTOs de usuário e evidência com a restauração de sessão, mas
/// exige uma evidência presente. Assim, o adaptador recebe os dois resultados da
/// mesma operação HTTP, sem precisar consultar a sessão depois de verificar.
/// A sugestão de e-mail fica separada da evidência: ela preenche o cadastro sem
/// virar outro endereço persistido. Contas ativas e a restauração não a recebem.
///
/// O cliente HTTP lê este envelope e o gateway produz um `AccountSession` próprio
/// da aplicação. A coordenação conserva a sugestão somente durante o cadastro e
/// persiste usuário/evidência por contratos separados. Esta biblioteca valida a
/// representação do protocolo; não autentica a resposta, calcula validade ou
/// escolhe se o aplicativo deve abrir o formulário.
library;

import 'account_session_response.dart';
import 'institutional_verification_response.dart';

/// Resultado de sucesso da verificação, diferente da restauração sem evidência.
final class InstitutionalAuthResponse {
  /// Recebe DTOs já tipados sem executar [InstitutionalAuthResponse.fromJson].
  ///
  /// [verification] é obrigatória para este resultado. [suggestedEmail] só deve
  /// ser fornecido se [user] estiver incompleto e o Graph informou texto não vazio.
  /// Construção manual não verifica essas condições nem conserva dados em cache.
  const InstitutionalAuthResponse({
    required this.user,
    required this.verification,
    this.suggestedEmail,
  });

  /// Conta provisionada ou recuperada pela mesma operação institucional.
  final InternalUserResponse user;

  /// Evidência concluída e persistida pelo backend, ainda sem avaliação local do prazo.
  final InstitutionalVerificationResponse verification;

  /// Dado transitório e opcional, exclusivo do preenchimento de conta incompleta.
  /// Não integra o perfil, a evidência ou o e-mail escolhido para entrar no CApp.
  final String? suggestedEmail;

  /// Interpreta o resultado e distingue omissão da sugestão de um valor inválido.
  ///
  /// [json] reutiliza a estrutura de [AccountSessionResponse], mas exige evidência
  /// não nula. `suggestedEmail` pode estar ausente; quando presente, deve ser string
  /// não vazia após `trim` e acompanhar `user.status == incomplete`. O texto válido
  /// é preservado sem aparar. A sintaxe completa de endereço não é validada aqui.
  ///
  /// Lança [FormatException] para evidência ausente, sugestão nula/em branco, tipo
  /// incompatível ou sugestão enviada para outro estado de conta. Falhas da leitura
  /// dos DTOs aninhados propagam, para classificação pelo gateway. Não consulta o
  /// Graph, verifica assinatura, avalia expiração ou modifica o e-mail da conta.
  factory InstitutionalAuthResponse.fromJson(Map<String, dynamic> json) {
    final session = AccountSessionResponse.fromJson(json);
    final verification = session.verification;

    if (verification == null) {
      throw const FormatException('Verificação institucional ausente');
    }

    final suggestedEmail = json['suggestedEmail'];

    if (json.containsKey('suggestedEmail') &&
        (suggestedEmail is! String ||
            suggestedEmail.trim().isEmpty ||
            session.user.status != 'incomplete')) {
      throw const FormatException('Sugestão de e-mail inválida');
    }

    return InstitutionalAuthResponse(
      user: session.user,
      verification: verification,
      suggestedEmail: suggestedEmail as String?,
    );
  }
}
