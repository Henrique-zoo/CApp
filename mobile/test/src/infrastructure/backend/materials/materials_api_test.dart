import 'dart:convert';
import 'dart:typed_data';

import 'package:capp/src/infrastructure/backend/backend_api.dart';
import 'package:capp/src/modules/materials/infrastructure/backend_materials_gateway.dart';
import 'package:capp/src/modules/materials/materials.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../../support/http_fakes.dart';

void main() {
  test(
    'o transporte protege a publicação e envia o PDF com os metadados certos',
    () async {
      const ca = 'c5afee45-5544-40d9-8958-b9a89b055ada';
      const material = '02f3e7b7-29a1-49d3-abde-5fd86ad9aef0';
      const discipline = '8d758e3a-95cc-4876-92aa-bd0d3834585a';
      const idempotency = '66a3126e-d33c-4bf1-a953-33187a0bd821';
      final bytes = Uint8List.fromList([37, 80, 68, 70, 45]);
      final adapter = FakeHttpAdapter((request) {
        if (request.method == 'POST') {
          expect(request.path, '/academic-centers/$ca/materials');
          expect(request.headers['Idempotency-Key'], idempotency);
          final form = request.data as FormData;
          expect(jsonDecode(form.fields.single.value), {
            'title': 'Resumo',
            'description': '',
            'disciplineId': discipline,
          });
          final file = form.files.single;
          expect(file.key, 'file');
          expect(file.value.contentType.toString(), 'application/pdf');
          expect(file.value.filename, 'resumo.pdf');
          expect(file.value.length, bytes.length);
          return jsonTestResponse({
            'id': material,
            'caId': ca,
            'disciplineId': discipline,
            'title': 'Resumo',
            'description': '',
            'content': {
              'type': 'file',
              'fileName': 'resumo.pdf',
              'sizeBytes': bytes.length,
            },
          }, status: 201);
        }
        expect(request.path, '/academic-centers/$ca/materials/$material/file');
        return ResponseBody.fromBytes(
          bytes,
          200,
          headers: {
            Headers.contentTypeHeader: ['application/pdf'],
          },
        );
      });
      final api = BackendApi(
        baseUrl: 'https://backend.example.invalid',
        getFirebaseIdToken: () async => 'identity-test',
        getFirebaseAppCheckToken: () async => 'attestation-test',
        dio: createTestDio(adapter),
      );
      var signedIn = true;
      final gateway = BackendMaterialsGateway(api.materials, () {
        if (!signedIn) throw const MaterialFailure('Conta alterada.');
      });

      final result = await gateway.publishPdf(
        ca,
        (title: 'Resumo', description: '', disciplineId: discipline),
        (name: 'resumo.pdf', bytes: bytes),
        idempotency,
      );
      expect(result.id, material);
      expect(result.content, isA<MaterialPdf>());
      expect(await gateway.download(ca, material), bytes);

      for (final request in adapter.requests) {
        final headers = request.headers.map(
          (key, value) => MapEntry(key.toLowerCase(), value),
        );
        expect(headers['authorization'], 'Bearer identity-test');
        expect(headers['x-firebase-appcheck'], 'attestation-test');
        expect(request.followRedirects, isFalse);
      }

      signedIn = false;
      await expectLater(
        gateway.download(ca, material),
        throwsA(isA<MaterialFailure>()),
      );
      expect(adapter.requests, hasLength(2));
    },
  );
}
