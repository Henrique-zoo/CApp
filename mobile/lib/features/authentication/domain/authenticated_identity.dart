/// Identidade retornada pelo backend após validar o Firebase ID token.
class AuthenticatedIdentity {
  const AuthenticatedIdentity({
    required this.uid,
    required this.email,
    required this.emailVerified,
  });

  /// Identificador único do usuário no Firebase Authentication.
  final String uid;

  /// E-mail associado à identidade, quando disponível.
  final String? email;

  /// Indica se o provedor confirma o e-mail como verificado.
  final bool emailVerified;

  /// Constrói a identidade a partir do DTO da API.
  factory AuthenticatedIdentity.fromJson(Map<String, dynamic> json) {
    final uid = json['uid'];
    final email = json['email'];
    final emailVerified = json['email_verified'];

    if (uid is! String || uid.trim().isEmpty) {
      throw const FormatException(
        'A resposta não contém uma identidade válida.',
      );
    }
    if (email != null && email is! String) {
      throw const FormatException('O e-mail retornado pela API é inválido.');
    }
    if (emailVerified is! bool) {
      throw const FormatException(
        'A verificação de e-mail retornada pela API é inválida.',
      );
    }

    return AuthenticatedIdentity(
      uid: uid,
      email: email as String?,
      emailVerified: emailVerified,
    );
  }
}
