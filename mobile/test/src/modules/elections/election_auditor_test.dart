// O processo OpenSSL confere as assinaturas dos vetores sem depender do
// verificador do servidor ou substituir a checagem por fixtures verdadeiras.
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:capp/src/modules/elections/application/election_audit.dart';
import 'package:capp/src/modules/elections/infrastructure/cryptography/election_auditor.dart';
import 'package:capp/src/modules/elections/infrastructure/device_voter_keys.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  final corpus = jsonDecode(
    File('../packages/api-contracts/elections/vectors.json').readAsStringSync(),
  ) as Map<String, dynamic>;
  final cases = {
    for (final row in corpus['cases'] as List) row['name'] as String: row,
  };
  Map<String, dynamic> envelope(String name) {
    final row = cases[name];
    return jsonDecode(
      jsonEncode({'payload': row['payload'], 'signature': row['signature']}),
    ) as Map<String, dynamic>;
  }

  Future<ElectionAudit> audit(
    String scenario, {
    bool missing = false,
    bool tampered = false,
  }) async {
    final manifest = envelope('$scenario/manifest');
    final election = UuidValue.withFormatValidation(
      manifest['payload']['electionId'] as String,
    );
    final closure = envelope('$scenario/closure');
    final count = closure['payload']['voterCount'] as int;
    final entries = [
      for (var i = 0; i < count; i++) envelope('$scenario/entry-$i'),
    ];
    if (tampered) {
      entries.first['payload']['ballot']['payload']['accountId'] =
          '99999999-1111-4111-8111-111111111111';
    }
    final authorityCount = (manifest['payload']['authorities'] as List).length;
    return auditElection(
      board: {
        ...corpus['periods'][scenario] as Map,
        'manifest': manifest,
        'entries': entries,
        'checkpoint': count == 0
            ? null
            : envelope('$scenario/receipt-${count - 1}'),
        'closure': closure,
        'aggregate': envelope('$scenario/aggregate'),
      },
      tally: {
        'shares': [
          for (var i = 0; i < authorityCount - (missing ? 1 : 0); i++)
            envelope('$scenario/share-$i'),
        ],
        'result': missing ? null : envelope('$scenario/result'),
      },
      election: election,
      trustedManifestHash: cases['$scenario/manifest']['sha256'] as String,
      externalReceipt: count == 0 ? null : envelope('$scenario/receipt-0'),
    );
  }

  late Directory scratch;
  setUpAll(() async {
    scratch = await Directory.systemTemp.createTemp('capp-offline-verifier-');
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(DeviceVoterKeys.channel, (call) async {
          expect(call.method, 'verify');
          final args = call.arguments as Map;
          final key = await File('${scratch.path}/key.der')
              .writeAsBytes(args['publicKey'] as Uint8List);
          final signature = await File('${scratch.path}/sig')
              .writeAsBytes(args['signature'] as Uint8List);
          final message = await File('${scratch.path}/message')
              .writeAsBytes(args['message'] as Uint8List);
          final result = await Process.run('openssl', [
            'dgst',
            '-sha256',
            '-verify',
            key.path,
            '-keyform',
            'DER',
            '-sigopt',
            'rsa_padding_mode:pss',
            '-sigopt',
            'rsa_pss_saltlen:32',
            '-sigopt',
            'rsa_mgf1_md:sha256',
            '-signature',
            signature.path,
            message.path,
          ]);
          return result.exitCode == 0;
        });
  });
  tearDownAll(() async {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(DeviceVoterKeys.channel, null);
    await scratch.delete(recursive: true);
  });

  test(
    'confere os totais com prova e assinatura sem acesso ao servidor',
    () async {
      final result = await audit('valid');
      expect(result.tally, AuditedTally.valid);
      expect(result.totals, [1, 0, 1]);
      expect(result.anchoredThrough, 1);
      expect(result.finalClosureAnchored, false);
    },
  );

  test(
    'distingue autoridade ausente, voto adulterado e agregado inconsistente',
    () async {
      final incomplete = await audit('valid', missing: true);
      expect(incomplete.tally, AuditedTally.incomplete);
      expect(incomplete.totals, isNull);
      await expectLater(audit('valid', tampered: true), throwsFormatException);
      expect((await audit('excess')).tally, AuditedTally.annulled);
    },
  );
}
