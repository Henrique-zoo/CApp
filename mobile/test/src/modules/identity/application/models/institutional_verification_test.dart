/// Igualdade da verificação inclui o resumo da instituição por valor.
library;

import 'package:capp/src/modules/identity/identity.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../../support/identity_fakes.dart';

/// Verifica os campos institucionais usados pela igualdade e pelo hash.
void main() {
  test('resumos equivalentes produzem verificações iguais', () {
    final first = testVerification();
    final second = testVerification();
    expect(identical(first.institution, second.institution), isFalse);
    expect(first, second);
    expect(first.hashCode, second.hashCode);
  });

  for (final institution in [
    const InstitutionSummary(
      id: UuidValue.raw('00000000-0000-0000-0000-000000000003'),
      name: 'Instituição Exemplo',
      acronym: 'IEX',
    ),
    const InstitutionSummary(
      id: UuidValue.raw('00000000-0000-0000-0000-000000000002'),
      name: 'Outra Instituição',
      acronym: 'IEX',
    ),
    const InstitutionSummary(
      id: UuidValue.raw('00000000-0000-0000-0000-000000000002'),
      name: 'Instituição Exemplo',
      acronym: null,
    ),
  ]) {
    test(
      'alterar instituição ${institution.id.uuid}/${institution.name}/${institution.acronym} altera igualdade',
      () {
        final original = testVerification();
        final changed = InstitutionalVerification(
          profile: original.profile,
          institution: institution,
          expiresAt: original.expiresAt,
        );
        expect(changed, isNot(original));
      },
    );
  }
}
