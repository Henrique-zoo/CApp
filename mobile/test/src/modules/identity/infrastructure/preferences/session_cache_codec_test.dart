/// Contrato persistente de versão 2, UUIDs e campos obrigatórios nullable.
///
/// As fixtures JSON são independentes do encoder. Não acessam preferências
/// ou transporte HTTP; registros antigos são rejeitados sem fabricar instituição.
library;

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/infrastructure/preferences/session_cache_codec.dart';

import '../../../../../support/identity_fakes.dart';

/// Registro atual construído sem depender do encoder que está sendo testado.
Map<String, dynamic> _record() => {
  'schemaVersion': 2,
  'firebaseUid': 'user-a',
  'verification': {
    'profile': {
      'microsoftObjectId': '00000000-0000-0000-0000-000000000001',
      'name': null,
      'email': null,
      'affiliation': {'type': 'nonStudent'},
    },
    'institution': <String, dynamic>{
      'id': '00000000-0000-0000-0000-000000000002',
      'name': 'Instituição Exemplo',
      'acronym': null,
    },
    'expiresAt': '2026-01-01T13:00:00.000Z',
  },
};

/// Registra compatibilidade, campos obrigatórios e rejeições explícitas.
void main() {
  const codec = SessionCacheCodec();

  test('recupera perfil e instituição com UUIDs e campos nulos', () {
    final result = codec.decode(
      jsonEncode(_record()),
      expectedFirebaseUid: 'user-a',
    ) as SessionCacheFound;
    expect(
      result.verification.profile.affiliation,
      isA<NonStudentAffiliation>(),
    );
    expect(
      result.verification.profile.microsoftObjectId,
      const UuidValue.raw('00000000-0000-0000-0000-000000000001'),
    );
    expect(
      result.verification.institution,
      const InstitutionSummary(
        id: UuidValue.raw('00000000-0000-0000-0000-000000000002'),
        name: 'Instituição Exemplo',
        acronym: null,
      ),
    );
    expect(result.verification.profile.name, isNull);
    expect(result.verification.expiresAt, DateTime.utc(2026, 1, 1, 13));
    expect(result.verification.expiresAt.isUtc, isTrue);
  });

  test('preserva estudante com curso nulo e registro expirado', () {
    final record = _record();
    record['verification']['profile']['affiliation'] = {
      'type': 'student',
      'courseName': null,
    };
    final result = codec.decode(
      jsonEncode(record),
      expectedFirebaseUid: 'user-a',
    ) as SessionCacheFound;
    expect(
      result.verification.profile.affiliation,
      const StudentAffiliation(courseName: null),
    );
    expect(result.verification.isValidAt(DateTime.utc(2026, 1, 2)), isFalse);
  });

  test('grava versão 2 com instituição e UUIDs textuais sem credenciais', () {
    final json = jsonDecode(
      codec.encode(firebaseUid: 'user-a', verification: testVerification()),
    ) as Map<String, dynamic>;
    expect(
      json.keys,
      unorderedEquals(['schemaVersion', 'firebaseUid', 'verification']),
    );
    expect(json['schemaVersion'], 2);
    expect(json['firebaseUid'], 'user-a');
    expect(
      json['verification'].keys,
      unorderedEquals(['profile', 'institution', 'expiresAt']),
    );
    expect(
      json['verification']['profile']['microsoftObjectId'],
      '00000000-0000-0000-0000-000000000001',
    );
    expect(json['verification']['institution'], {
      'id': '00000000-0000-0000-0000-000000000002',
      'name': 'Instituição Exemplo',
      'acronym': 'IEX',
    });
    expect(json['verification']['profile']['affiliation'], {
      'type': 'nonStudent',
    });
  });

  test('JSON malformado preserva a exceção e a pilha', () {
    final result = codec.decode(
      '{',
      expectedFirebaseUid: 'user-a',
    ) as SessionCacheRejected;
    expect(result.reason, SessionCacheRejectionReason.malformedJson);
    expect(result.cause, isA<FormatException>());
    expect(result.stackTrace, isNotNull);
  });

  test('versão e proprietário têm rejeições distintas', () {
    final version = _record()..['schemaVersion'] = 99;
    expect(
      (codec.decode(
        jsonEncode(version),
        expectedFirebaseUid: 'user-a',
      ) as SessionCacheRejected).reason,
      SessionCacheRejectionReason.unsupportedVersion,
    );
    expect(
      (codec.decode(
        jsonEncode(_record()),
        expectedFirebaseUid: 'user-b',
      ) as SessionCacheRejected).reason,
      SessionCacheRejectionReason.ownerMismatch,
    );
  });

  test('campo nullable ausente não equivale a campo explicitamente nulo', () {
    final record = _record();
    (record['verification']['profile'] as Map).remove('email');
    final result = codec.decode(
      jsonEncode(record),
      expectedFirebaseUid: 'user-a',
    ) as SessionCacheRejected;
    expect(result.reason, SessionCacheRejectionReason.invalidVerification);
    expect(result.cause, isA<FormatException>());
  });

  test('vínculo desconhecido não fabrica resultado não estudantil', () {
    final record = _record();
    record['verification']['profile']['affiliation'] = {'type': 'unknown'};
    expect(
      codec.decode(jsonEncode(record), expectedFirebaseUid: 'user-a'),
      isA<SessionCacheRejected>(),
    );
  });

  test('vínculo não estudantil não admite courseName, inclusive nulo', () {
    final record = _record();
    record['verification']['profile']['affiliation'] = <String, Object?>{
      'type': 'nonStudent',
      'courseName': null,
    };
    expect(
      codec.decode(jsonEncode(record), expectedFirebaseUid: 'user-a'),
      isA<SessionCacheRejected>(),
    );
  });

  for (final student in [false, true]) {
    for (final acronym in <String?>[null, 'IEX']) {
      test('ida e volta preserva estudante=$student e sigla=$acronym', () {
        final verification = testVerification(
          student: student,
          institutionAcronym: acronym,
        );
        final found = codec.decode(
          codec.encode(firebaseUid: 'user-a', verification: verification),
          expectedFirebaseUid: 'user-a',
        ) as SessionCacheFound;
        expect(found.verification, verification);
      });
    }
  }

  test('versão 1 sem instituição é rejeitada sem fabricar dados', () {
    final legacy = _record()..['schemaVersion'] = 1;
    (legacy['verification'] as Map).remove('institution');
    final result = codec.decode(
      jsonEncode(legacy),
      expectedFirebaseUid: 'user-a',
    ) as SessionCacheRejected;
    expect(result.reason, SessionCacheRejectionReason.unsupportedVersion);
  });

  for (final field in ['profile', 'institution']) {
    test('$field obrigatório ausente rejeita a verificação', () {
      final record = _record();
      (record['verification'] as Map).remove(field);
      final result = codec.decode(
        jsonEncode(record),
        expectedFirebaseUid: 'user-a',
      ) as SessionCacheRejected;
      expect(result.reason, SessionCacheRejectionReason.invalidVerification);
      expect(result.cause, isA<FormatException>());
    });
  }

  for (final field in [
    ('profile', 'name'),
    ('profile', 'email'),
    ('institution', 'acronym'),
  ]) {
    test('${field.$1}.${field.$2} ausente não equivale a nulo', () {
      final record = _record();
      (record['verification'][field.$1] as Map).remove(field.$2);
      final result = codec.decode(
        jsonEncode(record),
        expectedFirebaseUid: 'user-a',
      ) as SessionCacheRejected;
      expect(result.reason, SessionCacheRejectionReason.invalidVerification);
      expect(result.cause, isA<FormatException>());
    });
  }

  for (final field in [
    ('profile', 'microsoftObjectId'),
    ('institution', 'id'),
  ]) {
    for (final value in <Object?>['not-a-uuid', '', null, 42]) {
      test('${field.$1}.${field.$2} rejeita UUID incompatível: $value', () {
        final record = _record();
        record['verification'][field.$1][field.$2] = value;
        final result = codec.decode(
          jsonEncode(record),
          expectedFirebaseUid: 'user-a',
        ) as SessionCacheRejected;
        expect(result.reason, SessionCacheRejectionReason.invalidVerification);
        expect(result.cause, isA<FormatException>());
        expect(result.stackTrace, isNotNull);
        expect(result.cause.toString(), contains(field.$2));
        if (value == 'not-a-uuid') {
          expect(result.cause.toString(), isNot(contains('not-a-uuid')));
        }
      });
    }

    test('${field.$1}.${field.$2} precisa estar presente', () {
      final record = _record();
      (record['verification'][field.$1] as Map).remove(field.$2);
      final result = codec.decode(
        jsonEncode(record),
        expectedFirebaseUid: 'user-a',
      ) as SessionCacheRejected;
      expect(result.reason, SessionCacheRejectionReason.invalidVerification);
      expect(result.cause.toString(), contains('ausente'));
    });
  }

  test('normaliza UUIDs maiúsculos e aceita campos adicionais', () {
    final record = _record();
    const identifier = 'ABCDEF01-2345-6789-ABCD-EF0123456789';
    record['verification']['profile']['microsoftObjectId'] = identifier;
    record['verification']['institution']['id'] = identifier;
    record['verification']['institution']['extra'] = 'ignorado';
    final found = codec.decode(
      jsonEncode(record),
      expectedFirebaseUid: 'user-a',
    ) as SessionCacheFound;
    expect(
      found.verification.profile.microsoftObjectId.uuid,
      identifier.toLowerCase(),
    );
    expect(found.verification.institution.id.uuid, identifier.toLowerCase());
  });

  test('nome da instituição não admite nulo', () {
    final record = _record();
    record['verification']['institution']['name'] = null;
    final result = codec.decode(
      jsonEncode(record),
      expectedFirebaseUid: 'user-a',
    ) as SessionCacheRejected;
    expect(result.reason, SessionCacheRejectionReason.invalidVerification);
  });
}
