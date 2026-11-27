import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:url_launcher/url_launcher.dart';

import '../application/materials.dart';

final class PlatformMaterialDevice implements MaterialDevice {
  const PlatformMaterialDevice();

  @override
  Future<PdfSelection?> choosePdf() async {
    final file = await FilePicker.pickFile(
      type: FileType.custom,
      allowedExtensions: ['pdf'],
    );
    if (file == null) return null;
    if ((file.lengthSync() ?? 0) > maxMaterialPdfBytes) {
      throw const MaterialFailure('O PDF ultrapassa o limite de 10 MiB.');
    }
    final bytes = BytesBuilder(copy: false);
    await for (final chunk in file.readAsByteStream()) {
      if (chunk.length > maxMaterialPdfBytes - bytes.length) {
        throw const MaterialFailure('O PDF ultrapassa o limite de 10 MiB.');
      }
      bytes.add(chunk);
    }
    return (name: file.name, bytes: bytes.takeBytes());
  }

  @override
  Future<bool> savePdf(String name, Uint8List bytes) async =>
      await FilePicker.saveFile(
        fileName: name,
        bytes: bytes,
        mimeType: 'application/pdf',
      ) !=
      null;

  @override
  Future<bool> openLink(Uri uri) async {
    if ((!uri.isScheme('https') && !uri.isScheme('http')) ||
        uri.host.isEmpty ||
        uri.userInfo.isNotEmpty) {
      throw const MaterialFailure('Endereço externo inválido.');
    }
    return launchUrl(uri, mode: LaunchMode.externalApplication);
  }
}
