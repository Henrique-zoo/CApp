import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:capp/src/infrastructure/backend/backend_api.dart';
import 'package:capp/src/modules/elections/infrastructure/secret_voting.dart';
import 'package:capp/src/modules/elections/infrastructure/voter_registration_client.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../support/http_fakes.dart';
import '../../../support/preferences_fake.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  final vectors = jsonDecode(
    File('../packages/api-contracts/elections/vectors.json').readAsStringSync(),
  );
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
    'aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa',
  );
  final election = UuidValue.withFormatValidation(
    manifest['payload']['electionId'] as String,
  );
  final voterKey = ballot['payload']['publicKey'] as String;
  final serverKey = manifest['payload']['serverSigningKey'] as String;
  late Map<String, dynamic> state;
  final alias = DeviceVoterKeys.aliasFor(account, election);
  final draftKey = '$alias.signed-ballot';
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
  Future<BallotSession> open({String? pin}) {
    final api = BackendApi(
      baseUrl: 'https://example.invalid',
      getFirebaseAppCheckToken: () async => 'app-test',
      getFirebaseIdToken: () async => 'id-test',
      dio: createTestDio(transport),
    );
    return SecretVotingClient(
      api.ballots,
      api.voterRegistration,
      () => active,
      prefs,
    ).open(election, trustedManifestHash: pin ?? trustedHash);
  }

  setUp(() {
    state = {
      'publicKey': voterKey,
      'voterId': ballot['payload']['voterId'],
      'participated': false,
    };
    active = account;
    prefs = FakePreferences();
    board = {
      'manifest': envelope('manifest'),
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
        return jsonTestResponse(state);
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
        expect(args['alias'], alias);
        expect(args['create'], false);
        return base64Url.decode(base64Url.normalize(voterKey));
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
            text == 'CAPP-ELECTIONS/1\n${v['canonical']}') {
          return true;
        }
      }
      return false;
    });
  });
  tearDown(
    () => messenger.setMockMethodCallHandler(DeviceVoterKeys.channel, null),
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
        'kind',
        'manifestHash',
        'publicKey',
        'submissionId',
        'version',
        'voterId',
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
      expect(posted, [signed.json, signed.json]);
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
      expect(await session.isIncluded(accepted), true);
      board['entries'] = [];
      expect(await session.isIncluded(accepted), false);
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

  test('um registro assinado no índice do recibo não basta se o predecessor foi substituído', () async {
    final second = vector('ballot-1')['payload'];
    state = {
      'voterId': second['voterId'],
      'publicKey': second['publicKey'],
      'participated': true,
    };
    prefs.values[draftKey] = jsonEncode(envelope('ballot-1'));
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
