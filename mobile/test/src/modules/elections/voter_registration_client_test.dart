import 'dart:convert';
import 'dart:typed_data';

import 'package:capp/src/infrastructure/backend/backend_api.dart';
import 'package:capp/src/modules/elections/elections.dart';
import 'package:capp/src/modules/elections/infrastructure/backend/voter_registration_gateway.dart';
import 'package:capp/src/modules/elections/infrastructure/device_voter_keys.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../support/http_fakes.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  final vector = loadApiContractExample('election-voter-registration.json');
  final payload = vector['payload']! as Map<String, dynamic>;
  final accountId = UuidValue.withFormatValidation(
    payload['accountId'] as String,
  );
  final electionId = UuidValue.withFormatValidation(
    payload['electionId'] as String,
  );
  final publicKey = payload['publicKey'] as String;
  const empty = {
    'requiresRegistration': true,
    'voterId': null,
    'publicKey': null,
    'participated': false,
  };
  final registered = {
    'requiresRegistration': true,
    'voterId': accountId.uuid,
    'publicKey': publicKey,
    'participated': false,
  };

  late UuidValue? account;
  setUp(() => account = accountId);
  tearDown(
    () => messenger.setMockMethodCallHandler(DeviceVoterKeys.channel, null),
  );

  VoterRegistrationService client(
    FakeHttpAdapter transport, {
    String baseUrl = 'https://backend.example.invalid',
    Future<String?> Function()? token,
  }) {
    final backend = BackendApi(
      baseUrl: baseUrl,
      getFirebaseIdToken: token ?? () async => 'firebase-test',
      getFirebaseAppCheckToken: () async => 'app-test',
      dio: createTestDio(transport),
    );
    return VoterRegistrationService(
      BackendVoterRegistrationGateway(backend.voterRegistration),
      DeviceVoterKeys(),
      () => account,
    );
  }

  test('envia o envelope do vetor externo e recupera o cadastro após resposta perdida', () async {
    final keys = <String>[];
    var confirmed = false;
    messenger.setMockMethodCallHandler(DeviceVoterKeys.channel, (call) async {
      final arguments = call.arguments as Map;
      expect(arguments['alias'], 'capp.elections.v1.$accountId.$electionId');
      keys.add(call.method);
      if (call.method == 'publicKey') {
        expect(arguments['create'], !confirmed);
        return base64Url.decode(base64Url.normalize(publicKey));
      }
      // Construção independente dos bytes canônicos do vetor OpenAPI.
      final text = utf8.decode(arguments['message'] as Uint8List);
      expect(
        text,
        'CAPP-ELECTIONS/1\n{"accountId":"$accountId",'
        '"electionId":"$electionId","kind":"voter-registration",'
        '"publicKey":"$publicKey","version":1}',
      );
      final signature = vector['signature'] as String;
      return base64Url.decode(base64Url.normalize(signature));
    });
    final transport = FakeHttpAdapter((options) {
      expect(options.path, '/elections/$electionId/registration/me');
      expect(options.followRedirects, isFalse);
      expect(options.headers['Authorization'], 'Bearer firebase-test');
      expect(options.headers['X-Firebase-AppCheck'], 'app-test');
      if (options.method == 'GET') {
        return jsonTestResponse(confirmed ? registered : empty);
      }
      expect(options.method, 'PUT');
      expect(options.data, vector);
      confirmed = true;
      return jsonTestResponse({}, status: 503);
    });
    await expectLater(
      client(transport).register(electionId),
      throwsA(VoterRegistrationFailure.unavailable),
    );
    final state = await client(transport).register(electionId);
    expect(state.voterId, accountId);
    expect(state.publicKey, publicKey);
    expect(keys, ['publicKey', 'sign', 'publicKey']);
    expect(transport.requests.map((r) => r.method), ['GET', 'PUT', 'GET']);
  });

  test(
    'consulta em outro aparelho, sem substituir o par nem gerar segredo',
    () async {
      var reads = 0;
      messenger.setMockMethodCallHandler(DeviceVoterKeys.channel, (call) async {
        expect(call.method, 'publicKey');
        expect(call.arguments['create'], false);
        reads++;
        return null;
      });
      final transport = FakeHttpAdapter(
        (_) => jsonTestResponse({...registered, 'participated': true}),
      );
      final service = client(transport);
      expect((await service.status(electionId)).participated, isTrue);
      expect(reads, 0);
      await expectLater(
        service.register(electionId),
        throwsA(VoterRegistrationFailure.keyOnAnotherDevice),
      );
      expect(reads, 1);
      expect(transport.requests.map((r) => r.method), ['GET', 'GET']);
    },
  );

  test('troca da conta durante a obtenção do token impede o envio', () async {
    final transport = FakeHttpAdapter((_) => fail('enviou com outra conta'));
    final service = client(
      transport,
      token: () async {
        account = null;
        return 'outra-conta';
      },
    );
    await expectLater(
      service.status(electionId),
      throwsA(VoterRegistrationFailure.accountChanged),
    );
    expect(transport.requests, isEmpty);
  });

  test('dispensa cadastro sem gerar chave ou enviar PUT', () async {
    messenger.setMockMethodCallHandler(
      DeviceVoterKeys.channel,
      (_) => fail('gerou chave para cadastro dispensado'),
    );
    final transport = FakeHttpAdapter(
      (_) => jsonTestResponse({...empty, 'requiresRegistration': false}),
    );
    final service = client(transport);
    await expectLater(
      service.register(electionId),
      throwsA(VoterRegistrationFailure.registrationNotRequired),
    );
    expect(transport.requests.map((request) => request.method), ['GET']);
  });

  test('HTTP simples é recusado antes de solicitar credenciais', () async {
    final transport = FakeHttpAdapter((_) => fail('envio em HTTP'));
    final service = client(
      transport,
      baseUrl: 'http://backend.example.invalid',
      token: () => fail('solicitou credenciais'),
    );
    await expectLater(service.status(electionId), throwsStateError);
  });
}
