/// Invariantes do perfil compartilhadas entre transporte e cache.
///
/// A reconstituição deve aceitar contas incompletas e rejeitar fotografias
/// inconsistentes antes de disponibilizá-las à coordenação da sessão.
library;

import 'package:capp/src/modules/identity/application/models/internal_user.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

void main() {
  InternalUser profile({
    InternalUserStatus status = InternalUserStatus.active,
    String? nickname = 'Pessoa',
    String? email = 'pessoa@example.test',
    int? semester = 4,
    String? entry = '2025.1',
  }) => InternalUser(
    id: const UuidValue.raw('00000000-0000-0000-0000-000000000010'),
    status: status,
    nickname: nickname,
    email: email,
    currentSemester: semester,
    institutionEntrySemester: entry,
  );

  test('perfil ativo exige dados completos e valores consistentes', () {
    for (final invalid in <InternalUser Function()>[
      () => profile(nickname: null),
      () => profile(nickname: ' '),
      () => profile(email: null),
      () => profile(email: ''),
      () => profile(semester: null),
      () => profile(semester: 0),
      () => profile(entry: null),
      () => profile(entry: '2025.3'),
    ]) {
      expect(invalid, throwsFormatException);
    }

    expect(profile().status, InternalUserStatus.active);
  });

  test(
    'perfil incompleto permite dados pendentes sem aceitar valores inválidos',
    () {
      expect(
        profile(
          status: InternalUserStatus.incomplete,
          nickname: null,
          email: null,
          semester: null,
          entry: null,
        ).status,
        InternalUserStatus.incomplete,
      );

      expect(
        () => profile(status: InternalUserStatus.incomplete, semester: 0),
        throwsFormatException,
      );
      expect(
        () => profile(status: InternalUserStatus.incomplete, entry: '2025.3'),
        throwsFormatException,
      );
    },
  );
}
