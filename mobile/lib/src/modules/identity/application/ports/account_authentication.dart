/// Autenticação interna e seleção de contas com sessões independentes.
///
/// O controlador solicita operações sem conhecer FirebaseApp ou o armazenamento
/// do SDK. Senhas são argumentos transitórios, nunca dados de cache. A mesma
/// implementação deve fornecer AuthSession e MicrosoftSignIn à composição.
library;

import '../models/saved_account.dart';

abstract interface class AccountAuthentication {
  Future<List<SavedAccount>> savedAccounts();

  /// Exige senha quando a referência estiver protegida ou o SDK perdeu a sessão.
  Future<void> selectAccount(String firebaseUid, {String? password});

  /// Autentica uma conta interna existente. Não cria outro UID Firebase.
  Future<void> signInWithPassword({
    required String email,
    required String password,
  });

  /// Vincula password ao UID atual e confirma a credencial antes de retornar.
  Future<void> setUpPassword({required String email, required String password});

  Future<void> sendPasswordReset(String email);

  Future<void> leaveAccount(AccountSignOutMode mode);

  /// Esquece uma referência do seletor sem exigir autenticação nessa conta.
  Future<void> forgetSavedAccount(String firebaseUid);

  /// Disponibilidade local do provedor password; não concede autorização backend.
  bool get hasPassword;

  String? get currentEmail;
}
