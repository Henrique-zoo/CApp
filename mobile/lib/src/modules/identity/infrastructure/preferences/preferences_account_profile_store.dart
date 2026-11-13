/// Cache versionado do perfil interno, isolado por UID Firebase.
///
/// A restauração online substitui esta fotografia pelo backend. Conteúdo inválido
/// é recusado; senhas e tokens nunca participam do envelope. O codec é local e
/// não depende dos DTOs HTTP, para permitir evoluções independentes.
library;

import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';

import '../../application/models/internal_user.dart';
import '../../application/ports/account_profile_store.dart';

final class PreferencesAccountProfileStore implements AccountProfileStore {
  const PreferencesAccountProfileStore(this.preferences);

  final SharedPreferencesAsync preferences;

  String _key(String uid) => 'capp.identity.user.$uid';

  @override
  Future<InternalUser?> loadUser(String firebaseUid) async {
    final value = await preferences.getString(_key(firebaseUid));

    if (value == null) {
      return null;
    }

    final data = jsonDecode(value);

    if (data is! Map<String, dynamic> ||
        data['version'] != 1 ||
        data['uid'] != firebaseUid ||
        data['id'] is! String ||
        !InternalUserStatus.values.any((s) => s.name == data['status'])) {
      throw const FormatException('Cache de perfil incompatível');
    }

    // Extensão aditiva do envelope v1: caches anteriores não têm complementos.
    for (final key in [
      'nickname',
      'email',
      'entrySemester',
      'name',
      'institutionalEmail',
      'courseName',
    ]) {
      if (data[key] != null && data[key] is! String) {
        throw const FormatException('Campo de perfil incompatível');
      }
    }

    final semester = data['currentSemester'];

    if (semester != null && semester is! int) {
      throw const FormatException('Período incompatível');
    }

    final term = data['entrySemester'] as String?;

    return InternalUser(
      id: UuidValue.withFormatValidation(data['id'] as String),
      status: InternalUserStatus.values.byName(data['status'] as String),
      nickname: data['nickname'] as String?,
      email: data['email'] as String?,
      currentSemester: semester as int?,
      institutionEntrySemester: term,
      name: data['name'] as String?,
      institutionalEmail: data['institutionalEmail'] as String?,
      courseName: data['courseName'] as String?,
    );
  }

  @override
  Future<void> saveUser(String firebaseUid, InternalUser user) =>
      preferences.setString(
        _key(firebaseUid),
        jsonEncode({
          'version': 1,
          'uid': firebaseUid,
          'id': user.id.toString(),
          'status': user.status.name,
          'nickname': user.nickname,
          'email': user.email,
          'currentSemester': user.currentSemester,
          'entrySemester': user.institutionEntrySemester,
          'name': user.name,
          'institutionalEmail': user.institutionalEmail,
          'courseName': user.courseName,
        }),
      );

  @override
  Future<void> removeUser(String firebaseUid) =>
      preferences.remove(_key(firebaseUid));
}
