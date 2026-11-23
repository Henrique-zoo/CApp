import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

/// A decodificação JSON do Dio valida a sintaxe, mas sua conversão para Map
/// perderia nomes repetidos. Confira os nomes no texto original antes disso.
String _decodeEvidence(
  List<int> bytes,
  RequestOptions _,
  ResponseBody response,
) {
  final text = utf8.decode(bytes);
  if (response.statusCode < 200 || response.statusCode >= 300) return text;
  final names = <Set<String>>[];
  for (var i = 0; i < text.length; i++) {
    switch (text.codeUnitAt(i)) {
      case 0x7b: // {
        names.add(<String>{});
      case 0x7d: // }
        if (names.isEmpty) throw const FormatException('Objeto JSON inválido');
        names.removeLast();
      case 0x22: // "
        final start = i;
        i++;
        while (i < text.length && text.codeUnitAt(i) != 0x22) {
          if (text.codeUnitAt(i) == 0x5c) i++; // Pula o caractere escapado.
          i++;
        }
        if (i >= text.length) {
          throw const FormatException('String JSON incompleta');
        }
        var next = i + 1;
        while (next < text.length && text.codeUnitAt(next) <= 0x20) {
          next++;
        }
        if (next < text.length && text.codeUnitAt(next) == 0x3a) {
          final name = jsonDecode(text.substring(start, i + 1)) as String;
          if (names.isEmpty || !names.last.add(name)) {
            throw const FormatException(
              'Nome de campo inválido ou repetido na evidência',
            );
          }
        }
    }
  }
  return text;
}

/// O quadro é público. O voto e o recibo pessoal usam a sessão autenticada.
final class BallotApi {
  BallotApi(this._dio);
  final Dio _dio;

  void _requireHttps() {
    if (!Uri.parse(_dio.options.baseUrl).isScheme('https')) {
      throw StateError('A votação exige HTTPS');
    }
  }

  Future<Map<String, dynamic>> board(UuidValue id) async {
    _requireHttps();
    final response = await _dio.get<Map<String, dynamic>>(
      '/elections/${id.uuid}/board',
      options: Options(
        followRedirects: false,
        responseDecoder: _decodeEvidence,
      ),
    );
    return response.data ?? (throw const FormatException('Quadro vazio'));
  }

  Future<Map<String, dynamic>> submit(
    UuidValue id,
    String envelope,
    void Function() guard,
  ) async {
    _requireHttps();
    guard();
    final response = await _dio.post<Map<String, dynamic>>(
      '/elections/${id.uuid}/ballots',
      data: envelope,
      options: _authenticated(guard),
    );
    guard();
    final data = response.data;
    if (data == null ||
        (response.statusCode != 201 && response.statusCode != 200) ||
        data['recovered'] != (response.statusCode == 200)) {
      throw const FormatException('Confirmação de voto inválida');
    }
    return data;
  }

  Future<Map<String, dynamic>> receipt(
    UuidValue id,
    void Function() guard,
  ) async {
    _requireHttps();
    guard();
    final response = await _dio.get<Map<String, dynamic>>(
      '/elections/${id.uuid}/receipts/me',
      options: _authenticated(guard),
    );
    guard();
    return response.data ?? (throw const FormatException('Recibo vazio'));
  }

  Options _authenticated(void Function() guard) => Options(
    contentType: Headers.jsonContentType,
    followRedirects: false,
    responseDecoder: _decodeEvidence,
    extra: {
      'firebaseAuth': true,
      'firebaseAppCheck': true,
      'firebaseAuthGuard': guard,
    },
  );
}
