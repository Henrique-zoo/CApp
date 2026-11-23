import 'package:capp/src/infrastructure/backend/elections/ballot_api.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../support/http_fakes.dart';

void main() {
  final id = UuidValue.withFormatValidation(
    '558260d1-5b98-4ab1-ad5e-cc6f8b9ebf8c',
  );
  BallotApi client(String text, {int status = 200}) {
    final http = FakeHttpAdapter(
      (_) => ResponseBody.fromString(
        text,
        status,
        headers: {
          Headers.contentTypeHeader: ['application/json'],
        },
      ),
    );
    return BallotApi(createTestDio(http));
  }

  final ambiguous = throwsA(
    isA<DioException>().having((e) => e.error, 'causa', isA<FormatException>()),
  );

  test('recusa nomes equivalentes no mesmo objeto de um quadro', () async {
    final api = client(r'{"entries":[{"index":0,"\u0069ndex":1}]}');
    await expectLater(api.board(id), ambiguous);
  });

  test('recusa duas versões do recibo na resposta ao voto', () async {
    final api = client(
      '{"recovered":false,"receipt":{},"receipt":{}}',
      status: 201,
    );
    await expectLater(api.submit(id, '{}', () {}), ambiguous);
  });

  test(
    'aceita nomes em objetos distintos e pontuação escapada em strings',
    () async {
      final api = client(r'{"entries":[{"v":"}:\"\\{"},{"v":2}]}');
      final board = await api.board(id);
      expect(board['entries'], [
        {'v': r'}:"\{'},
        {'v': 2},
      ]);
    },
  );

  test('preserva o corpo JSON e o status de conflito do backend', () async {
    final api = client('{"error":{"code":"already_voted"}}', status: 409);
    await expectLater(
      api.submit(id, '{}', () {}),
      throwsA(
        isA<DioException>()
            .having((e) => e.response?.statusCode, 'HTTP', 409)
            .having(
              (e) => e.response?.data['error']['code'],
              'erro',
              'already_voted',
            ),
      ),
    );
  });
}
