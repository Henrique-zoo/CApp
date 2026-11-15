/// Resposta de uma verificação institucional concluída com conta provisionada.
///
/// Compartilha os DTOs de usuário e evidência com a restauração de sessão, mas
/// exige uma evidência presente. Assim, o adaptador recebe os dois resultados da
/// mesma operação HTTP, sem precisar consultar a sessão depois de verificar.
/// A sugestão de e-mail fica separada da evidência: ela preenche o cadastro sem
/// virar outro endereço persistido. Contas ativas e a restauração não a recebem.
library;

import 'account_session_response.dart';
import 'institutional_verification_response.dart';

final class InstitutionalAuthResponse {
  const InstitutionalAuthResponse({
    required this.user,
    required this.verification,
    this.suggestedEmail,
  });

  final InternalUserResponse user;
  final InstitutionalVerificationResponse verification;

  /// Dado transitório e opcional, exclusivo da criação da conta.
  final String? suggestedEmail;

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
