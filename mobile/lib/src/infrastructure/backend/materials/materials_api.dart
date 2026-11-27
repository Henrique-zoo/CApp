import 'dart:convert';
import 'dart:typed_data';

import 'package:dio/dio.dart';

/// Transporte de materiais sobre a mesma sessão, App Check e origem HTTPS.
final class MaterialsApi {
  const MaterialsApi(this._dio);
  final Dio _dio;

  String _collection(String caId) =>
      '/academic-centers/${Uri.encodeComponent(caId)}/materials';

  Options _options(
    void Function() guard, {
    String? requestId,
    ResponseType? responseType,
  }) => Options(
    responseType: responseType,
    headers: {'Idempotency-Key': ?requestId},
    extra: {
      'firebaseAuth': true,
      'firebaseAppCheck': true,
      'firebaseAuthGuard': guard,
    },
  );

  Future<List<dynamic>> centers(void Function() guard) async =>
      (await _dio.get<List<dynamic>>(
        '/materials/contexts',
        options: _options(guard),
      )).data!;

  Future<List<dynamic>> disciplines(String caId, void Function() guard) async =>
      (await _dio.get<List<dynamic>>(
        '${_collection(caId)}/disciplines',
        options: _options(guard),
      )).data!;

  Future<Map<String, dynamic>> list(
    String caId,
    String? disciplineId,
    int offset,
    void Function() guard,
  ) async => (await _dio.get<Map<String, dynamic>>(
    _collection(caId),
    queryParameters: {'offset': offset, 'disciplineId': ?disciplineId},
    options: _options(guard),
  )).data!;

  Future<Map<String, dynamic>> details(
    String caId,
    String id,
    void Function() guard,
  ) async => (await _dio.get<Map<String, dynamic>>(
    '${_collection(caId)}/${Uri.encodeComponent(id)}',
    options: _options(guard),
  )).data!;

  Future<Uint8List> download(
    String caId,
    String id,
    void Function() guard,
  ) async {
    final response = await _dio.get<List<int>>(
      '${_collection(caId)}/${Uri.encodeComponent(id)}/file',
      options: _options(guard, responseType: ResponseType.bytes),
    );
    if (response.headers.value('content-type')?.split(';').first !=
        'application/pdf') {
      throw const FormatException('O servidor não devolveu um PDF.');
    }
    return Uint8List.fromList(response.data!);
  }

  Future<Map<String, dynamic>> publishLink(
    String caId,
    Map<String, Object> metadata,
    String url,
    String requestId,
    void Function() guard,
  ) async => (await _dio.post<Map<String, dynamic>>(
    _collection(caId),
    data: {
      ...metadata,
      'content': {'type': 'external_link', 'url': url},
    },
    options: _options(guard, requestId: requestId)
      ..contentType = Headers.jsonContentType,
  )).data!;

  Future<Map<String, dynamic>> publishPdf(
    String caId,
    Map<String, Object> metadata,
    String name,
    Uint8List bytes,
    String requestId,
    void Function() guard,
  ) async => (await _dio.post<Map<String, dynamic>>(
    _collection(caId),
    data: FormData.fromMap({
      'metadata': jsonEncode(metadata),
      'file': MultipartFile.fromBytes(
        bytes,
        filename: name,
        contentType: DioMediaType('application', 'pdf'),
      ),
    }),
    options: _options(guard, requestId: requestId),
  )).data!;
}
