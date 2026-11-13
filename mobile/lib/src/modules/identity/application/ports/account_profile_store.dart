/// Persistência da fotografia interna para retomar a interface sem rede.
///
/// A fotografia é associada ao UID autenticado e nunca substitui autorização
/// backend. A validade institucional continua no SessionStore.
library;

import '../models/internal_user.dart';

abstract interface class AccountProfileStore {
  Future<InternalUser?> loadUser(String firebaseUid);

  Future<void> saveUser(String firebaseUid, InternalUser user);

  Future<void> removeUser(String firebaseUid);
}
