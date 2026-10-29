/// Testes exclusivos do contrato persistente de versão 1.
///
/// As fixtures representam registros anteriores e não usam o encoder novo
/// para produzi-los. Não acessa preferências ou transporte HTTP.
library;

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:capp/src/modules/identity/identity.dart';
import 'package:capp/src/modules/identity/infrastructure/preferences/session_cache_codec.dart';

import '../../../../../support/identity_fakes.dart';

/// Registro compatível com a representação anterior do cache.
Map<String, dynamic> _legacyRecord() => {
  'schemaVersion': 1,
  'firebaseUid': 'user-a',
  'verification': {
    'profile': {
      'microsoftObjectId': '00000000-0000-0000-0000-000000000001',
      'name': null,
      'email': null,
      'affiliation': {'type': 'nonStudent'},
    },
    'expiresAt': '2026-01-01T13:00:00.000Z',
  },
};

/// Registra compatibilidade, campos obrigatórios e rejeições explícitas.
void main() {
  const codec = SessionCacheCodec();

  test('lê o registro antigo sem exigir migração de versão', () {
    final result = codec.decode(
      jsonEncode(_legacyRecord()),
      expectedFirebaseUid: 'user-a',
    ) as SessionCacheFound;
    expect(
      result.verification.profile.affiliation,
      isA<NonStudentAffiliation>(),
    );
    expect(result.verification.profile.name, isNull);
    expect(result.verification.expiresAt, DateTime.utc(2026, 1, 1, 13));
    expect(result.verification.expiresAt.isUtc, isTrue);
  });

  test('preserva estudante com curso nulo e registro expirado', () {
    final record = _legacyRecord();
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

  test('escreve envelope local sem credenciais e preserva a versão', () {
    final json = jsonDecode(
      codec.encode(firebaseUid: 'user-a', verification: testVerification()),
    ) as Map<String, dynamic>;
    expect(
      json.keys,
      unorderedEquals(['schemaVersion', 'firebaseUid', 'verification']),
    );
    expect(json['schemaVersion'], 1);
    expect(json['firebaseUid'], 'user-a');
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
    final version = _legacyRecord()..['schemaVersion'] = 2;
    expect(
      (codec.decode(
        jsonEncode(version),
        expectedFirebaseUid: 'user-a',
      ) as SessionCacheRejected).reason,
      SessionCacheRejectionReason.unsupportedVersion,
    );
    expect(
      (codec.decode(
        jsonEncode(_legacyRecord()),
        expectedFirebaseUid: 'user-b',
      ) as SessionCacheRejected).reason,
      SessionCacheRejectionReason.ownerMismatch,
    );
  });

  test('campo nullable ausente não equivale a campo explicitamente nulo', () {
    final record = _legacyRecord();
    (record['verification']['profile'] as Map).remove('email');
    final result = codec.decode(
      jsonEncode(record),
      expectedFirebaseUid: 'user-a',
    ) as SessionCacheRejected;
    expect(result.reason, SessionCacheRejectionReason.invalidVerification);
    expect(result.cause, isA<FormatException>());
  });

  test('vínculo desconhecido não fabrica resultado não estudantil', () {
    final record = _legacyRecord();
    record['verification']['profile']['affiliation'] = {'type': 'unknown'};
    expect(
      codec.decode(jsonEncode(record), expectedFirebaseUid: 'user-a'),
      isA<SessionCacheRejected>(),
    );
  });

  test('vínculo não estudantil não admite courseName, inclusive nulo', () {
    final record = _legacyRecord();
    record['verification']['profile']['affiliation'] = <String, Object?>{
      'type': 'nonStudent',
      'courseName': null,
    };
    expect(
      codec.decode(jsonEncode(record), expectedFirebaseUid: 'user-a'),
      isA<SessionCacheRejected>(),
    );
  });
}
