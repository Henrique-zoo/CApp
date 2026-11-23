import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:capp/src/infrastructure/backend/backend_api.dart';
import 'package:capp/src/modules/elections/infrastructure/election_failure_adapter.dart';
import 'package:capp/src/modules/elections/infrastructure/secret_voting.dart';
import 'package:capp/src/modules/elections/infrastructure/voter_registration_client.dart';
import 'package:capp/src/modules/elections/presentation/election_home.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../support/http_fakes.dart';
import '../../../support/preferences_fake.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final vectors = jsonDecode(
    File('../packages/api-contracts/elections/vectors.json').readAsStringSync(),
  );
  Map<String, dynamic> vector(String name) =>
      (vectors['cases'] as List).firstWhere((v) => v['name'] == 'valid/$name');
  Map<String, dynamic> envelope(String name) => {
    'payload': vector(name)['payload'],
    'signature': vector(name)['signature'],
  };
  final electionId = UuidValue.withFormatValidation(
    vector('manifest')['payload']['electionId'],
  );
  final account = UuidValue.withFormatValidation(
    'aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa',
  );
  final voter = vector('ballot-0')['payload'];
  final opening = DateTime.parse(
    vector('calendar-origin')['payload']['startsAt'],
  );
  final closing = DateTime.parse(
    vector('calendar-origin')['payload']['endsAt'],
  );
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  late DateTime now;
  late bool registered, localKey, participated, includeEntry, acceptReplay;
  late int signed;
  late SecretVotingClient client;
  late List<String> sent;
  late Completer<void> enrollment;
  late FakePreferences preferences;

  setUp(() {
    now = opening;
    registered = true;
    localKey = true;
    participated = false;
    includeEntry = true;
    acceptReplay = false;
    signed = 0;
    sent = [];
    preferences = FakePreferences();
    final transport = FakeHttpAdapter((options) {
      switch (options.path.split('/').last) {
        case 'board':
          return jsonTestResponse({
            'manifest': envelope('manifest'),
            'calendarOrigin': envelope('calendar-origin'),
            'entries': [if (includeEntry) envelope('entry-0')],
          });
        case 'me':
          if (options.path.endsWith('/receipts/me')) {
            return jsonTestResponse(envelope('receipt-0'));
          }
          return jsonTestResponse({
            'voterId': registered ? voter['voterId'] : null,
            'publicKey': registered ? voter['publicKey'] : null,
            'participated': participated,
          });
        case 'ballots':
          sent.add(options.data as String);
          if (acceptReplay) {
            participated = true;
            return jsonTestResponse({
              'entry': envelope('entry-0'),
              'receipt': envelope('receipt-0'),
              'recovered': true,
            });
          }
          return jsonTestResponse({}, status: 503);
        default:
          throw StateError(options.path);
      }
    });
    final api = BackendApi(
      baseUrl: 'https://eleicao.example.invalid',
      getFirebaseAppCheckToken: () async => 'app-test',
      getFirebaseIdToken: () async => 'id-test',
      dio: createTestDio(transport),
    );
    client = SecretVotingClient(
      api.ballots,
      api.voterRegistration,
      () => account,
      preferences,
      clock: () => now,
    );
    // A suíte do protocolo verifica as assinaturas. Aqui observamos os
    // comandos do eleitor e a sua apresentação com o cliente real.
    messenger.setMockMethodCallHandler(DeviceVoterKeys.channel, (call) async {
      switch (call.method) {
        case 'verify':
          return true;
        case 'publicKey':
          expect(call.arguments['create'], false);
          return localKey
              ? base64Url.decode(base64Url.normalize(voter['publicKey']))
              : null;
        case 'sign':
          signed++;
          return Uint8List.fromList([1, 2, 3]);
        default:
          throw UnsupportedError(call.method);
      }
    });
  });
  tearDown(
    () => messenger.setMockMethodCallHandler(DeviceVoterKeys.channel, null),
  );

  Future<void> example(WidgetTester tester) async {
    enrollment = Completer<void>();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ElectionHome(
            classifyFailure: classifyElectionFailure,
            inspect: () => client.inspect(
              electionId,
              trustedManifestHash: vector('manifest')['sha256'],
            ),
            register: () async {
              await enrollment.future;
              registered = true;
            },
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
  }

  Future<void> tap(WidgetTester tester, String text) async {
    await tester.tap(find.text(text));
    await tester.pumpAndSettle();
  }

  testWidgets('habilitação explícita e abertura na própria tela', (
    tester,
  ) async {
    now = opening.subtract(const Duration(minutes: 10));
    registered = false;
    await example(tester);
    await tap(tester, 'Abrir eleição');
    expect(registered, false);
    await tap(tester, 'Habilitar para votar');
    expect(find.text('Habilitando este aparelho…'), findsOneWidget);
    enrollment.complete();
    await tester.pump();
    await tester.pumpAndSettle();
    expect(
      find.text('Você está habilitado para votar neste aparelho.'),
      findsOneWidget,
    );
    now = opening;
    await tester.pump(const Duration(minutes: 10));
    await tester.pumpAndSettle();
    expect(find.text('Escolha uma opção'), findsOneWidget);
    expect(signed, 0);
  });

  testWidgets(
    'não abre cédula sem habilitação, no aparelho errado ou fora do prazo',
    (tester) async {
      registered = false;
      await example(tester);
      await tap(tester, 'Ver situação');
      expect(find.text('O prazo de habilitação terminou.'), findsOneWidget);
      registered = true;
      localKey = false;
      await tap(tester, 'Voltar ao Início');
      await tap(tester, 'Ver situação');
      expect(find.text('Use o aparelho habilitado'), findsOneWidget);
      now = closing;
      await tap(tester, 'Voltar ao Início');
      await tap(tester, 'Ver situação');
      expect(find.text('Votação encerrada'), findsOneWidget);
      expect(sent, isEmpty);
    },
  );

  testWidgets('alterar antes do envio e reabrir a mesma cédula após falha', (
    tester,
  ) async {
    await example(tester);
    await tap(tester, 'Votar');
    final voteButton = find.widgetWithText(FilledButton, 'Votar');
    expect(tester.widget<FilledButton>(voteButton).onPressed, isNull);
    await tap(tester, 'Aurora');
    await tap(tester, 'Votar');
    expect(find.textContaining('Você escolheu Aurora'), findsOneWidget);
    await tap(tester, 'Voltar e alterar');
    expect(sent, isEmpty);
    await tap(tester, 'Voto nulo');
    await tap(tester, 'Votar');
    expect(find.textContaining('Você escolheu Voto nulo'), findsOneWidget);
    await tap(tester, 'Confirmar e enviar');
    expect(find.text('Envio sem confirmação'), findsOneWidget);
    await tester.pageBack();
    await tester.pumpAndSettle();
    await tap(tester, 'Retomar envio');
    await tap(tester, 'Retomar envio');
    expect(sent.length, 2);
    expect(sent.first, sent.last);
    expect(signed, 1);
  });

  testWidgets(
    'reenvio aceito mostra o recibo e permite conferir a inclusão novamente',
    (tester) async {
      final alias = DeviceVoterKeys.aliasFor(account, electionId);
      preferences.values['$alias.signed-ballot'] = jsonEncode(
        envelope('ballot-0'),
      );
      acceptReplay = true;
      includeEntry = false;
      await example(tester);
      await tap(tester, 'Retomar envio');
      await tap(tester, 'Retomar envio');
      expect(find.text('Voto registrado'), findsOneWidget);
      expect(find.text('Registro nº 1'), findsOneWidget);
      expect(find.text('Inclusão verificada no quadro público.'), findsNothing);
      includeEntry = true;
      await tap(tester, 'Conferir novamente');
      expect(
        find.text('Inclusão verificada no quadro público.'),
        findsOneWidget,
      );
      expect(sent.length, 1);
      expect(signed, 0);
    },
  );

  testWidgets(
    'recupera e verifica o recibo em outro aparelho após o encerramento',
    (tester) async {
      participated = true;
      localKey = false;
      now = closing;
      await example(tester);
      expect(find.text('Participação registrada'), findsOneWidget);
      expect(find.text('Recibo validado'), findsNothing);
      await tap(tester, 'Ver recibo');
      expect(find.text('Recibo validado'), findsOneWidget);
      expect(find.text('Registro nº 1'), findsOneWidget);
      expect(
        find.text('Inclusão verificada no quadro público.'),
        findsOneWidget,
      );
      expect(find.text('Aurora'), findsNothing);
      registered = false;
      participated = false;
      await tap(tester, 'Voltar ao Início');
      expect(find.text('Ver recibo'), findsOneWidget);
      expect(find.text('Habilitar para votar'), findsNothing);
      includeEntry = false;
      await tap(tester, 'Ver recibo');
      expect(find.text('Inclusão verificada no quadro público.'), findsNothing);
      expect(find.text('Recibo validado'), findsNothing);
      expect(sent, isEmpty);
    },
  );
}
