/// Referência técnica persistida para reencontrar uma sessão Firebase e MSAL.
///
/// O seletor usa somente SavedAccount. O nome do FirebaseApp e o objeto Microsoft
/// necessário à limpeza pertencem à infraestrutura e permanecem neste envelope,
/// lido pelo AccountRegistry. O formato persistente existente é preservado.
library;

import 'package:uuid/uuid.dart';

import '../../application/models/saved_account.dart';

/// Metadados do SDK associados à referência pública de uma conta lembrada.
final class FirebaseAccountReference {
  const FirebaseAccountReference({
    required this.account,
    required this.authAppName,
    this.microsoftObjectId,
  });

  final SavedAccount account;

  final String authAppName;
  final UuidValue? microsoftObjectId;

  String get firebaseUid => account.firebaseUid;

  bool get requiresPassword => account.requiresPassword;

  String? get email => account.email;

  String? get name => account.name;

  /// Protege a seleção sem perder a referência às credenciais a limpar.
  FirebaseAccountReference requiringPassword({String? email, String? name}) =>
      FirebaseAccountReference(
        account: SavedAccount(
          firebaseUid: firebaseUid,
          requiresPassword: true,
          email: email ?? this.email,
          name: name ?? this.name,
        ),
        authAppName: authAppName,
        microsoftObjectId: microsoftObjectId,
      );
}
