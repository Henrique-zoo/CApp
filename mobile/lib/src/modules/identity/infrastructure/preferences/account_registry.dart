/// Índice persistente dos contêineres Firebase e da escolha de conta ativa.
///
/// Um único envelope substitui índice e seleção juntos. A flag de senha é gravada
/// antes de encerrar o SDK, evitando que uma saída interrompida volte a permitir
/// entrada rápida. O chamador serializa mutações. Este índice não guarda tokens,
/// senhas, permissões ou evidência institucional e não decide acesso ao backend.
library;

import 'dart:convert';

import 'package:uuid/uuid.dart';

import 'package:shared_preferences/shared_preferences.dart';

import '../../application/models/saved_account.dart';
import '../firebase/firebase_account_reference.dart';

final class AccountRegistry {
  AccountRegistry(this.preferences);

  final SharedPreferencesAsync preferences;
  static const key = 'capp.identity.accounts.v1';

  Future<({List<FirebaseAccountReference> accounts, String? activeUid})?>
  read() async {
    final encoded = await preferences.getString(key);

    if (encoded == null) return null;

    final data = jsonDecode(encoded);

    if (data is! Map<String, dynamic> ||
        data['version'] != 1 ||
        data['accounts'] is! List ||
        (data['activeUid'] != null && data['activeUid'] is! String)) {
      throw const FormatException('Índice de contas incompatível');
    }

    final accounts = <FirebaseAccountReference>[];
    final uids = <String>{};
    final apps = <String>{};

    for (final entry in data['accounts'] as List) {
      if (entry is! Map<String, dynamic> ||
          entry['uid'] is! String ||
          (entry['uid'] as String).isEmpty ||
          entry['app'] is! String ||
          (entry['app'] as String).isEmpty ||
          entry['requiresPassword'] is! bool ||
          (entry['email'] != null && entry['email'] is! String) ||
          (entry['name'] != null && entry['name'] is! String) ||
          (entry['microsoftObjectId'] != null &&
              entry['microsoftObjectId'] is! String) ||
          !uids.add(entry['uid'] as String) ||
          !apps.add(entry['app'] as String)) {
        throw const FormatException('Referência de conta inválida');
      }

      accounts.add(
        FirebaseAccountReference(
          account: SavedAccount(
            firebaseUid: entry['uid'] as String,
            requiresPassword: entry['requiresPassword'] as bool,
            email: entry['email'] as String?,
            name: entry['name'] as String?,
          ),
          authAppName: entry['app'] as String,
          microsoftObjectId: entry['microsoftObjectId'] == null
              ? null
              : UuidValue.withFormatValidation(
                  entry['microsoftObjectId'] as String,
                ),
        ),
      );
    }

    final activeUid = data['activeUid'] as String?;

    if (activeUid != null && !uids.contains(activeUid)) {
      throw const FormatException('Conta ativa ausente no índice');
    }

    return (accounts: accounts, activeUid: activeUid);
  }

  Future<void> write(
    List<FirebaseAccountReference> accounts,
    String? activeUid,
  ) => preferences.setString(
    key,
    jsonEncode({
      'version': 1,
      'activeUid': activeUid,
      'accounts': [
        for (final account in accounts)
          {
            'uid': account.firebaseUid,
            'app': account.authAppName,
            'requiresPassword': account.requiresPassword,
            'email': account.email,
            'name': account.name,
            'microsoftObjectId': account.microsoftObjectId?.uuid,
          },
      ],
    }),
  );
}
