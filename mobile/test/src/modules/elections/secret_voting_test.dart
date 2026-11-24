import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:capp/src/infrastructure/backend/backend_api.dart';
import 'package:capp/src/infrastructure/backend/elections/ballot_dto.dart';
import 'package:capp/src/modules/elections/infrastructure/cryptography/election_signing_input.dart';
import 'package:capp/src/modules/elections/infrastructure/secret_voting.dart';
import 'package:capp/src/modules/elections/infrastructure/backend/voter_registration_gateway.dart';
import 'package:capp/src/modules/elections/infrastructure/device_voter_keys.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../support/http_fakes.dart';
import '../../../support/preferences_fake.dart';

String submissionFor(String signed) {
  final envelope = jsonDecode(signed) as Map;
  final payload = envelope['payload'] as Map;
  return jsonEncode({
    'payload': {
      for (final field in ['manifestHash', 'publicKey', 'ciphertexts'])
        field: payload[field],
    },
    'signature': envelope['signature'],
  });
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  final vectors = jsonDecode(
    File('../packages/api-contracts/elections/vectors.json').readAsStringSync(),
  );
  final domains =
      (jsonDecode(
            File('../packages/api-contracts/elections/profile.json')
                .readAsStringSync(),
          ) as Map)['domainsHex']
          as Map<String, dynamic>;
  List<int> prefix(String kind) {
    final hex = domains[kind] as String;
    return [
      for (var i = 0; i < hex.length; i += 2)
        int.parse(hex.substring(i, i + 2), radix: 16),
    ];
  }

  Map<String, dynamic> vector(String name) =>
      (vectors['cases'] as List).firstWhere((v) => v['name'] == 'valid/$name');
  Map<String, dynamic> envelope(String name) => {
    'payload': vector(name)['payload'],
    'signature': vector(name)['signature'],
  };

  final manifest = vector('manifest');
  final ballot = vector('ballot-0');
  final trustedHash = manifest['sha256'] as String;
  final account = UuidValue.withFormatValidation(
    ballot['payload']['accountId'] as String,
  );
  final election = UuidValue.withFormatValidation(
    manifest['payload']['electionId'] as String,
  );
  final voterKey = ballot['payload']['publicKey'] as String;
  final serverKey = manifest['payload']['serverSigningKey'] as String;
  late Map<String, dynamic> state;
  final alias = DeviceVoterKeys.aliasFor(account, election);
  final draftKey = '$alias.signed-ballot-v4';
  String encoded(List<int> bytes) =>
      base64Url.encode(bytes).replaceAll('=', '');

  late FakePreferences prefs;
  late Map<String, dynamic> board;
  late Map<String, dynamic> record;
  late int postStatus;
  late FakeHttpAdapter transport;
  late List<String> posted;
  late UuidValue? active;
  late Set<String> signedByDevice;
  late String? localKey;
  late int keyCreations;
  late DateTime now;
  late Map<String, dynamic> ownReceipt;
  SecretVotingClient client() {
    final api = BackendApi(
      baseUrl: 'https://example.invalid',
      getFirebaseAppCheckToken: () async => 'app-test',
      getFirebaseIdToken: () async => 'id-test',
      dio: createTestDio(transport),
    );
    return SecretVotingClient(
      api.ballots,
      BackendVoterRegistrationGateway(api.voterRegistration),
      () => active,
      prefs,
      clock: () => now,
    );
  }

  Future<BallotSession> open({String? pin}) =>
      client().open(election, trustedManifestHash: pin ?? trustedHash);
  Future<ElectionInspection> inspect() =>
      client().inspect(election, trustedManifestHash: trustedHash);

  setUp(() {
    state = {
      'publicKey': voterKey,
      'voterId': ballot['payload']['accountId'],
      'participated': false,
    };
    active = account;
    prefs = FakePreferences();
    now = DateTime.utc(2026, 11, 19);
    localKey = voterKey;
    keyCreations = 0;
    ownReceipt = envelope('receipt-0');
    board = {
      'manifest': envelope('manifest'),
      'startsAt': '2026-11-20T12:00:00Z',
      'endsAt': '2026-11-22T12:00:00Z',
      'entries': [envelope('entry-0')],
    };
    record = {
      'entry': envelope('entry-0'),
      'receipt': envelope('receipt-0'),
      'recovered': false,
    };
    postStatus = 201;
    posted = [];
    signedByDevice = {};
    transport = FakeHttpAdapter((options) {
      expect(options.followRedirects, false);
      if (options.path == '/elections/$election/board') {
        expect(options.headers.containsKey('Authorization'), false);
        return jsonTestResponse(board);
      }
      expect(options.headers['Authorization'], 'Bearer id-test');
      expect(options.headers['X-Firebase-AppCheck'], 'app-test');
      if (options.path == '/elections/$election/registration/me') {
        return jsonTestResponse({'requiresRegistration': true, ...state});
      }
      if (options.path == '/elections/$election/receipts/me') {
        expect(options.method, 'GET');
        return jsonTestResponse(ownReceipt);
      }
      expect(options.path, '/elections/$election/ballots');
      expect(options.method, 'POST');
      posted.add(options.data as String);
      return jsonTestResponse(record, status: postStatus);
    });
    // O canal é substituído no host Flutter. A verificação deste teste exige
    // os bytes canônicos, a chave e a assinatura do vetor externo exato.
    messenger.setMockMethodCallHandler(DeviceVoterKeys.channel, (call) async {
      final args = call.arguments as Map;
      if (call.method == 'publicKey') {
        expect(args['alias'], DeviceVoterKeys.aliasFor(active!, election));
        if (args['create'] == true && localKey == null) {
          keyCreations++;
          localKey = voterKey;
        }
        return localKey == null
            ? null
            : base64Url.decode(base64Url.normalize(localKey!));
      }
      final text = utf8.decode(args['message'] as Uint8List);
      if (call.method == 'sign') {
        expect(args['alias'], alias);
        signedByDevice.add(text);
        return Uint8List.fromList([1, 2, 3]);
      }
      expect(call.method, 'verify');
      final key = encoded(args['publicKey']);
      final sig = encoded(args['signature']);
      if (sig == 'AQID') {
        return key == voterKey && signedByDevice.contains(text);
      }
      for (final item in [
        'manifest',
        'ballot-0',
        'ballot-1',
        'entry-0',
        'entry-1',
        'receipt-0',
        'receipt-1',
      ]) {
        final v = vector(item);
        if (key ==
                (item.startsWith('ballot-')
                    ? v['payload']['publicKey']
                    : serverKey) &&
            sig == v['signature'] &&
            text ==
                '${utf8.decode(prefix((v['payload']['kind'] ?? 'ballot') as String))}${v['canonical']}') {
          return true;
        }
      }
      return false;
    });
  });
  tearDown(
    () => messenger.setMockMethodCallHandler(DeviceVoterKeys.channel, null),
  );

  test('usa os domínios fixos e os hashes dos vetores compartilhados', () {
    for (final kind in domains.keys) {
      final payload = {'version': 1, 'kind': kind};
      expect(
        electionSigningBytes(payload).take(prefix(kind).length),
        prefix(kind),
      );
    }
    for (final vector in vectors['cases'] as List) {
      expect(
        vector['payload']['kind'] == null
            ? ballotDigest(vector['payload'] as Map<String, dynamic>)
            : electionDigest(vector['payload'] as Map<String, dynamic>),
        vector['sha256'],
      );
    }
    expect(
      () => electionSigningBytes({'version': 1, 'kind': 'unknown'}),
      throwsFormatException,
    );
  });

  test('converte a cédula assinada no corpo HTTP do exemplo compartilhado', () {
    final public = envelope('ballot-0');
    final ballot = public['payload'] as Map<String, dynamic>;
    final dto = BallotSubmissionDto(
      manifestHash: ballot['manifestHash'] as String,
      publicKey: ballot['publicKey'] as String,
      ciphertexts: (ballot['ciphertexts'] as List).cast<Map<String, dynamic>>(),
      signature: public['signature'] as String,
    );
    final example = jsonDecode(
      File('../packages/api-contracts/examples/election-ballot-submission.json')
          .readAsStringSync(),
    );
    expect(dto.toJson(), example);
  });

  test('cria a chave somente no primeiro voto sem cadastro prévio', () async {
    state = {
      'requiresRegistration': false,
      'publicKey': null,
      'voterId': null,
      'participated': false,
    };
    localKey = null;

    final election = await inspect();
    expect(election.canPrepareVote, true);
    expect(keyCreations, 0);
    final session = election.session!;
    final draft = await session.prepare(session.optionIds.first);
    expect(keyCreations, 1);
    expect(jsonDecode(draft.json)['payload']['accountId'], account.uuid);

    postStatus = 503;
    await expectLater(session.submit(draft), throwsA(isA<DioException>()));
    final reopened = await open();
    expect((await reopened.savedVote())!.json, draft.json);
    await expectLater(reopened.submit(draft), throwsA(isA<DioException>()));
    expect(posted, [submissionFor(draft.json), submissionFor(draft.json)]);
    expect(keyCreations, 1);
  });

  test(
    'mostra chapas e período autenticados mesmo antes do cadastro',
    () async {
      state = {'publicKey': null, 'voterId': null, 'participated': false};
      final result = await inspect();
      expect(result.registered, false);
      expect(result.hasSavedReceipt, false);
      expect(result.localKeyMatches, false);
      expect(result.participated, false);
      expect(result.session, isNull);
      expect(result.options.map((o) => o.name), [
        'Aurora',
        'Horizonte',
        'Nulo',
      ]);
      expect(result.options.map((o) => o.isNull), [false, false, true]);
      expect(result.startsAt, DateTime.utc(2026, 11, 20, 12));
      expect(result.endsAt, DateTime.utc(2026, 11, 22, 12));
      expect(result.phase, ElectionPhase.preparation);
      expect(result.timeUntilNextPhase, result.startsAt.difference(now));
      now = result.startsAt;
      expect(result.phase, ElectionPhase.voting);
      expect(result.timeUntilNextPhase, result.endsAt.difference(now));
      now = result.endsAt;
      expect(result.phase, ElectionPhase.closed);
      expect(result.timeUntilNextPhase, isNull);
      await expectLater(open(), throwsStateError);
      expect(prefs.values, isEmpty);
    },
  );

  test(
    'recusa período inválido e encerramento com assinatura inválida',
    () async {
      board['endsAt'] = board['startsAt'];
      await expectLater(inspect(), throwsFormatException);
      board['endsAt'] = '2026-11-22T12:00:00Z';
      board['closure'] = envelope('receipt-0');
      await expectLater(inspect(), throwsFormatException);
      expect(transport.requests.every((r) => r.path.endsWith('/board')), true);
    },
  );

  test(
    'recupera e vincula o recibo em outro aparelho após o encerramento',
    () async {
      state['participated'] = true;
      now = DateTime.utc(2026, 11, 23);
      expect((await inspect()).localKeyMatches, true);
      localKey = null;
      final result = await inspect();
      expect(result.phase, ElectionPhase.closed);
      expect(result.registered, true);
      expect(result.participated, true);
      expect(result.localKeyMatches, false);
      final receipt = await result.session!.recoverReceipt();
      expect(receipt.recovered, true);
      expect(jsonDecode(receipt.receiptJson), envelope('receipt-0'));
      expect(prefs.values['$alias.signed-receipt-v2'], receipt.receiptJson);
      expect(await result.session!.isIncluded(receipt), true);
      expect(posted, isEmpty);
      expect(signedByDevice, isEmpty);
    },
  );

  test(
    'recusa recibo de outro eleitor mesmo com assinaturas válidas',
    () async {
      ownReceipt = envelope('receipt-1');
      board['entries'] = [envelope('entry-0'), envelope('entry-1')];
      state['participated'] = true;
      final session = await open();
      await expectLater(session.recoverReceipt(), throwsFormatException);
      expect(prefs.values, isEmpty);
      ownReceipt = {...envelope('receipt-0'), 'signature': 'AQID'};
      await expectLater(session.recoverReceipt(), throwsFormatException);
      active = null;
      await expectLater(session.recoverReceipt(), throwsStateError);
    },
  );

  test(
    'preserva o recibo se a participação e depois o cadastro desaparecerem',
    () async {
      prefs.values[draftKey] = jsonEncode(envelope('ballot-0'));
      final original = await open();
      final draft = (await original.savedVote())!;
      final receipt = await original.submit(draft);
      board['entries'] = [];
      state['participated'] = false;
      final stale = await inspect();
      expect(stale.hasSavedReceipt, true);
      expect(stale.participated, false);
      expect(
        (await stale.session!.recoverReceipt()).receiptJson,
        receipt.receiptJson,
      );

      state = {'publicKey': null, 'voterId': null, 'participated': false};
      final lost = await inspect();
      expect(lost.hasSavedReceipt, true);
      expect(lost.registered, false);
      final session = lost.session!;
      final recovered = await session.recoverReceipt();
      expect(recovered.receiptJson, receipt.receiptJson);
      expect(await session.isIncluded(recovered), false);
      await expectLater(
        session.prepare(session.optionIds.first),
        throwsStateError,
      );
      await expectLater(session.submit(draft), throwsStateError);
      expect(posted, [submissionFor(draft.json)]);
      localKey = null;
      await expectLater(session.recoverReceipt(), throwsStateError);
      expect(prefs.values['$alias.signed-receipt-v2'], receipt.receiptJson);
    },
  );

  test(
    'cifra e assina uma vez, restaura o pacote e reenvia os mesmos bytes',
    () async {
      final session = await open();
      final signed = await session.prepare(session.optionIds.first);
      final payload = jsonDecode(signed.json)['payload'] as Map;
      expect(payload.keys.toSet(), {
        'ciphertexts',
        'electionId',
        'manifestHash',
        'publicKey',
        'accountId',
      });
      final encrypted = payload['ciphertexts'] as List;
      expect(
        encrypted.map((v) => v['optionId']),
        session.optionIds.map((id) => id.uuid),
      );
      for (final part in encrypted) {
        expect(part.keys.toSet(), {'optionId', 'a', 'b'});
        expect(BigInt.parse(part['a']), greaterThan(BigInt.one));
        expect(BigInt.parse(part['b']), greaterThan(BigInt.two));
      }
      expect(prefs.values[draftKey], signed.json);
      postStatus = 503;
      await expectLater(session.submit(signed), throwsA(isA<DioException>()));
      final reopened = await open();
      final saved = await reopened.savedVote();
      expect(saved!.json, signed.json);
      await expectLater(reopened.submit(saved), throwsA(isA<DioException>()));
      expect(posted, [submissionFor(signed.json), submissionFor(signed.json)]);
      await expectLater(
        reopened.prepare(reopened.optionIds.last),
        throwsStateError,
      );
      expect(signedByDevice.length, 1);
    },
  );

  test(
    'confere recibo e inclusão separadamente, inclusive para um reenvio',
    () async {
      prefs.values[draftKey] = jsonEncode(envelope('ballot-0'));
      final session = await open();
      final draft = (await session.savedVote())!;
      final accepted = await session.submit(draft);
      expect(jsonDecode(accepted.receiptJson), envelope('receipt-0'));
      expect(accepted.recovered, false);
      expect(prefs.values['$alias.signed-receipt-v2'], accepted.receiptJson);
      expect(await session.isIncluded(accepted), true);
      board['entries'] = [];
      final recovered = await (await open()).recoverReceipt();
      expect(recovered.receiptJson, accepted.receiptJson);
      expect(await session.isIncluded(recovered), false);
      expect(
        transport.requests.any((r) => r.path.endsWith('/receipts/me')),
        false,
      );
      record['recovered'] = true;
      postStatus = 200;
      final retry = await session.submit(draft);
      expect(retry.recovered, true);
      expect(retry.receiptJson, accepted.receiptJson);
      record['receipt'] = envelope('receipt-1');
      await expectLater(session.submit(draft), throwsFormatException);
      record['receipt'] = {...envelope('receipt-0'), 'signature': 'AQID'};
      await expectLater(session.submit(draft), throwsFormatException);
    },
  );

  test(
    'a consulta autentica o envelope e os campos exibidos do recibo',
    () async {
      final session = await open();
      final genuine = await session.recoverReceipt();
      AcceptedVote displayed({String? envelope, int? index}) => AcceptedVote(
        receiptJson: envelope ?? genuine.receiptJson,
        index: index ?? genuine.index,
        entryHash: genuine.entryHash,
        ballotHash: genuine.ballotHash,
        recovered: true,
      );
      await expectLater(
        session.isIncluded(displayed(index: 2)),
        throwsFormatException,
      );
      final changed = {...envelope('receipt-0'), 'signature': 'AQID'};
      await expectLater(
        session.isIncluded(displayed(envelope: jsonEncode(changed))),
        throwsFormatException,
      );
      board['entries'] = [envelope('entry-0'), envelope('entry-1')];
      final other = vector('receipt-1')['payload'];
      final otherVote = AcceptedVote(
        receiptJson: jsonEncode(envelope('receipt-1')),
        index: other['index'],
        entryHash: other['entryHash'],
        ballotHash: other['ballotHash'],
        recovered: true,
      );
      await expectLater(session.isIncluded(otherVote), throwsFormatException);
      await expectLater(
        session.submit(SignedVote(jsonEncode(envelope('ballot-0')))),
        throwsStateError,
      );
      expect(posted, isEmpty);
    },
  );

  test('um registro assinado no índice do recibo não basta se o predecessor foi substituído', () async {
    final second = vector('ballot-1')['payload'];
    final secondAccount = UuidValue.withFormatValidation(
      second['accountId'] as String,
    );
    state = {
      'voterId': second['accountId'],
      'publicKey': second['publicKey'],
      'participated': true,
    };
    await expectLater(open(), throwsFormatException);

    active = secondAccount;
    localKey = second['publicKey'] as String;
    final secondAlias = DeviceVoterKeys.aliasFor(secondAccount, election);
    prefs.values['$secondAlias.signed-ballot-v4'] = jsonEncode(
      envelope('ballot-1'),
    );
    record = {
      'recovered': true,
      'entry': envelope('entry-1'),
      'receipt': envelope('receipt-1'),
    };
    postStatus = 200;
    board['entries'] = [envelope('entry-0'), envelope('entry-1')];
    final session = await open();
    final accepted = await session.submit((await session.savedVote())!);
    expect(accepted.index, 2);
    expect(await session.isIncluded(accepted), true);
    // O segundo registro legítimo ocupa também o lugar do primeiro. A posição
    // observada no recibo continua idêntica e ambas as assinaturas são válidas.
    board['entries'] = [envelope('entry-1'), envelope('entry-1')];
    expect(await session.isIncluded(accepted), false);
  });

  test('recusa manifesto divergente antes de enviar credenciais e impede troca de conta', () async {
    await expectLater(open(pin: '0' * 64), throwsFormatException);
    expect(transport.requests.map((o) => o.path), [
      '/elections/$election/board',
    ]);
    final session = await open();
    active = null;
    await expectLater(
      session.prepare(session.optionIds.last),
      throwsStateError,
    );
    expect(posted, isEmpty);
  });
}
