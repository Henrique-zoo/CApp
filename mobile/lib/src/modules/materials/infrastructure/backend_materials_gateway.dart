import 'dart:typed_data';

import 'package:dio/dio.dart';

import '../../../infrastructure/backend/materials/materials_api.dart';
import '../application/materials.dart';

final class BackendMaterialsGateway implements MaterialsGateway {
  const BackendMaterialsGateway(this._api, this._guard);
  final MaterialsApi _api;
  final void Function() _guard;

  Future<T> _request<T>(Future<T> Function() operation) async {
    _guard();
    try {
      final result = await operation();
      _guard();
      return result;
    } on DioException catch (error) {
      final embedded = error.response?.data;
      if (error.error case final MaterialFailure failure) throw failure;
      if (embedded case {'error': {'message': final String message}}) {
        throw MaterialFailure(message);
      }
      throw const MaterialFailure(
        'Não foi possível acessar o servidor de materiais.',
      );
    } on FormatException {
      throw const MaterialFailure(
        'O servidor retornou dados de materiais inválidos.',
      );
    } on TypeError {
      throw const MaterialFailure(
        'O servidor retornou dados de materiais inválidos.',
      );
    }
  }

  @override
  Future<List<MaterialCenter>> centers() => _request(
    () async => [
      for (final item in await _api.centers(_guard))
        (
          id: item['id'] as String,
          name: item['name'] as String,
          canPublish: item['canPublish'] as bool,
          canUploadPdf: item['canUploadPdf'] as bool,
        ),
    ],
  );

  @override
  Future<List<MaterialDiscipline>> disciplines(String caId) => _request(
    () async => [
      for (final item in await _api.disciplines(caId, _guard))
        (id: item['id'] as String, name: item['name'] as String),
    ],
  );

  @override
  Future<PublishedMaterialPage> list(
    String caId, {
    String? disciplineId,
    int offset = 0,
  }) => _request(() async {
    final response = await _api.list(caId, disciplineId, offset, _guard);
    return (
      items: [
        for (final item in response['items'] as List<dynamic>)
          _material(item, caId),
      ],
      nextOffset: response['nextOffset'] as int?,
    );
  });

  @override
  Future<PublishedMaterial> details(String caId, String id) => _request(
    () async => _material(await _api.details(caId, id, _guard), caId),
  );

  @override
  Future<Uint8List> download(String caId, String id) => _request(() async {
    final pdf = await _api.download(caId, id, _guard);
    if (pdf.isEmpty || pdf.length > maxMaterialPdfBytes) {
      throw const FormatException('Tamanho de PDF inválido.');
    }
    return pdf;
  });

  @override
  Future<PublishedMaterial> publishLink(
    String caId,
    MaterialDraft draft,
    String url,
    String requestId,
  ) => _request(
    () async => _material(
      await _api.publishLink(caId, _metadata(draft), url, requestId, _guard),
      caId,
    ),
  );

  @override
  Future<PublishedMaterial> publishPdf(
    String caId,
    MaterialDraft draft,
    PdfSelection pdf,
    String requestId,
  ) => _request(
    () async => _material(
      await _api.publishPdf(
        caId,
        _metadata(draft),
        pdf.name,
        pdf.bytes,
        requestId,
        _guard,
      ),
      caId,
    ),
  );

  Map<String, Object> _metadata(MaterialDraft draft) => {
    'title': draft.title,
    'description': draft.description,
    'disciplineId': draft.disciplineId,
  };

  PublishedMaterial _material(Map<String, dynamic> data, String caId) {
    if (data['caId'] != caId) {
      throw const FormatException('Material em contexto inesperado.');
    }
    final content = switch (data['content']) {
      {'type': 'external_link', 'url': final String value} => MaterialLink(
        Uri.parse(value),
      ),
      {
        'type': 'file',
        'fileName': final String name,
        'sizeBytes': final int size,
      } =>
        MaterialPdf(name, size),
      _ => throw const FormatException('Tipo de material desconhecido.'),
    };
    return (
      id: data['id'] as String,
      disciplineId: data['disciplineId'] as String,
      title: data['title'] as String,
      description: data['description'] as String,
      content: content,
    );
  }
}
