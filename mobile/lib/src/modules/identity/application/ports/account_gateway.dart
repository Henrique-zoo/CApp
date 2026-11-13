/// Operações do perfil interno autenticadas pelo Firebase, sem token Microsoft.
///
/// A restauração consulta usuário e prazo institucional no backend. Somente a
/// sincronização institucional passa pela porta InstitutionalIdentityGateway.
library;

import '../models/internal_user.dart';

abstract interface class AccountGateway {
  /// Recupera conta e evidência sem consultar o Graph ou estender seu prazo.
  ///
  /// Evidência nula solicita sincronização. IdentityFailureKind.accountNotProvisioned
  /// identifica primeiro acesso; recusas e falhas temporárias propagam com suas
  /// categorias, sem serem tratadas como conta ausente.
  Future<AccountSession> restoreSession();

  Future<InternalUser> completeProfile(ProfileCompletion profile);
}
