/// Casos de fronteira dos contratos de sessão e perfil interno.
library;

import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/infrastructure/backend/identity/dto/account_session_response.dart';

void main() {
  Map<String, Object?> complete() => {
    'id': '00000000-0000-0000-0000-000000000010',
    'status': 'active',
    'nickname': 'Pessoa',
    'email': 'pessoal@example.test',
    'currentSemester': 4,
    'institutionEntrySemester': '2023.2',
  };

  test('sessão sem evidência mantém o perfil sem inventar confirmação', () {
    final parsed = AccountSessionResponse.fromJson({
      'user': complete(),
      'verification': null,
    });

    expect(parsed.verification, isNull);
    expect(parsed.user.currentSemester, 4);
  });

  test('complementos opcionais são preservados sem exigir mudança de contrato antigo', () {
    final json = complete()
      ..addAll({
        'name': 'Nome declarado',
        'institutionalEmail': 'declarado@example.test',
        'courseName': 'Curso declarado',
      });
    final parsed = InternalUserResponse.fromJson(json);

    expect(parsed.name, 'Nome declarado');
    expect(parsed.institutionalEmail, 'declarado@example.test');
    expect(parsed.courseName, 'Curso declarado');

    json['courseName'] = 42;

    expect(() => InternalUserResponse.fromJson(json), throwsFormatException);
  });

  test(
    'perfil incompleto aceita ausência dos dados, mas não campos omitidos',
    () {
      final json = complete()
        ..['status'] = 'incomplete'
        ..['nickname'] = null
        ..['email'] = null
        ..['currentSemester'] = null
        ..['institutionEntrySemester'] = null;

      expect(InternalUserResponse.fromJson(json).status, 'incomplete');

      json.remove('nickname');

      expect(() => InternalUserResponse.fromJson(json), throwsFormatException);
    },
  );

  for (final (key, value) in <(String, Object?)>[
    ('id', 'not-uuid'),
    ('status', 'unknown'),
    ('currentSemester', 1.5),
  ]) {
    test('recusa formato HTTP incompatível: $key=$value', () {
      final json = complete()..[key] = value;

      expect(() => InternalUserResponse.fromJson(json), throwsFormatException);
    });
  }
}
