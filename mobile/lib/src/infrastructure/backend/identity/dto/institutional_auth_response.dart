/// Resposta de uma verificação institucional concluída com conta provisionada.
///
/// Compartilha os DTOs de usuário e evidência com a restauração de sessão, mas
/// exige uma evidência presente. Assim, o adaptador recebe os dois resultados da
/// mesma operação HTTP, sem precisar consultar a sessão depois de verificar.
library;

import 'account_session_response.dart';
import 'institutional_verification_response.dart';

final class InstitutionalAuthResponse {
  const InstitutionalAuthResponse({
    required this.user,
    required this.verification,
  });

  final InternalUserResponse user;
  final InstitutionalVerificationResponse verification;

  factory InstitutionalAuthResponse.fromJson(Map<String, dynamic> json) {
    final session = AccountSessionResponse.fromJson(json);
    final verification = session.verification;

    if (verification == null) {
      throw const FormatException('Verificação institucional ausente');
    }

    return InstitutionalAuthResponse(
      user: session.user,
      verification: verification,
    );
  }
}
