import 'dart:typed_data';

const maxMaterialPdfBytes = 10 * 1024 * 1024;

typedef MaterialCenter = ({
  String id,
  String name,
  bool canPublish,
  bool canUploadPdf,
});
typedef MaterialDiscipline = ({String id, String name});
typedef PdfSelection = ({String name, Uint8List bytes});

sealed class MaterialContent {
  const MaterialContent();
}

final class MaterialLink extends MaterialContent {
  const MaterialLink(this.url);
  final Uri url;
}

final class MaterialPdf extends MaterialContent {
  const MaterialPdf(this.name, this.sizeBytes);
  final String name;
  final int sizeBytes;
}

typedef PublishedMaterial = ({
  String id,
  String disciplineId,
  String title,
  String description,
  MaterialContent content,
});

typedef PublishedMaterialPage = ({
  List<PublishedMaterial> items,
  int? nextOffset,
});
typedef MaterialDraft = ({
  String title,
  String description,
  String disciplineId,
});

final class MaterialFailure implements Exception {
  const MaterialFailure(this.message);
  final String message;
}

/// Operações vinculadas à conta autenticada que abriu o módulo.
abstract interface class MaterialsGateway {
  Future<List<MaterialCenter>> centers();
  Future<List<MaterialDiscipline>> disciplines(String caId);
  Future<PublishedMaterialPage> list(
    String caId, {
    String? disciplineId,
    int offset = 0,
  });
  Future<PublishedMaterial> details(String caId, String id);
  Future<Uint8List> download(String caId, String id);
  Future<PublishedMaterial> publishLink(
    String caId,
    MaterialDraft draft,
    String url,
    String requestId,
  );
  Future<PublishedMaterial> publishPdf(
    String caId,
    MaterialDraft draft,
    PdfSelection pdf,
    String requestId,
  );
}

abstract interface class MaterialDevice {
  Future<PdfSelection?> choosePdf();
  Future<bool> savePdf(String name, Uint8List bytes);
  Future<bool> openLink(Uri uri);
}
