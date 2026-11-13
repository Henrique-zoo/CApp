/// Referência local para escolher uma conta sem armazenar suas credenciais.
///
/// Selecionar esta referência ainda exige uma sessão Firebase utilizável ou a
/// senha interna. Sua presença no dispositivo não autentica nem autoriza ninguém.
library;

/// Decisão explícita de saída, aplicada somente à conta selecionada.
enum AccountSignOutMode {
  /// Oculta a conta ativa e conserva a sessão SDK para uma próxima seleção.
  keepSession,

  /// Conserva a referência, mas encerra as credenciais locais dessa conta.
  requirePassword,

  /// Encerra as credenciais e remove os dados locais, sem excluir a conta remota.
  forgetAccount,
}

/// Metadados apresentados no seletor. Não incluem senhas ou tokens.
final class SavedAccount {
  const SavedAccount({
    required this.firebaseUid,
    required this.requiresPassword,
    this.email,
    this.name,
  });

  final String firebaseUid;

  final bool requiresPassword;
  final String? email;
  final String? name;
}
