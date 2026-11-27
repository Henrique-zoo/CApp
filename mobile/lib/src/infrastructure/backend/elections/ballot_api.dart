import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

import '../serialization/election_json.dart';
import 'ballot_dto.dart';

/// A decodificação JSON do Dio valida a sintaxe, mas sua conversão para Map
/// perderia nomes repetidos. Confira os nomes no texto original antes disso.
String _decodeEvidence(
  List<int> bytes,
  RequestOptions _,
  ResponseBody response,
) {
  final text = utf8.decode(bytes);
  if (response.statusCode < 200 || response.statusCode >= 300) return text;
  return checkElectionJson(text);
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

  /// Lê as contribuições públicas e o resultado confirmado da eleição.
  Future<Map<String, dynamic>> tally(UuidValue id) async {
    _requireHttps();
    final response = await _dio.get<Map<String, dynamic>>(
      '/elections/${id.uuid}/tally',
      options: Options(
        followRedirects: false,
        responseDecoder: _decodeEvidence,
      ),
    );
    return response.data ?? (throw const FormatException('Apuração ausente'));
  }

  Future<RecordedBallotDto> submit(
    UuidValue id,
    BallotSubmissionDto envelope,
    void Function() guard,
  ) async {
    _requireHttps();
    guard();
    final response = await _dio.post<Map<String, dynamic>>(
      '/elections/${id.uuid}/ballots',
      data: jsonEncode(envelope.toJson()),
      options: _authenticated(guard),
    );
    guard();
    final data = RecordedBallotDto.fromJson(response.data);
    if ((response.statusCode != 201 && response.statusCode != 200) ||
        data.recovered != (response.statusCode == 200)) {
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

  /// Consulta a designação de autoridade da conta autenticada.
  Future<Map<String, dynamic>> authority(
    UuidValue id,
    void Function() guard,
  ) async {
    _requireHttps();
    guard();
    final response = await _dio.get<Map<String, dynamic>>(
      '/elections/${id.uuid}/authority/me',
      options: _authenticated(guard),
    );
    guard();
    return response.data ?? (throw const FormatException('Designação ausente'));
  }

  /// Cadastra a credencial criptográfica da autoridade designada.
  Future<Map<String, dynamic>> registerAuthority(
    UuidValue id,
    String envelope,
    void Function() guard,
  ) => _submitAuthority('/elections/${id.uuid}/authority/me', envelope, guard);

  /// Publica o manifesto a pedido da comissão desta eleição.
  Future<Map<String, dynamic>> publishManifest(
    UuidValue id,
    void Function() guard,
  ) => _submitAuthority('/elections/${id.uuid}/manifest', null, guard);

  /// Atualiza o período vigente pela conta que representa a comissão.
  Future<void> reschedule(
    UuidValue id,
    DateTime startsAt,
    DateTime endsAt,
    String reason,
    void Function() guard,
  ) async {
    await _submitAuthority(
      '/elections/${id.uuid}/period',
      jsonEncode({
        'startsAt': startsAt.toUtc().toIso8601String().replaceFirst(
          '.000Z',
          'Z',
        ),
        'endsAt': endsAt.toUtc().toIso8601String().replaceFirst('.000Z', 'Z'),
        'reason': reason,
      }),
      guard,
    );
  }

  /// Registra o motivo do cancelamento pela comissão.
  Future<void> cancel(
    UuidValue id,
    String reason,
    void Function() guard,
  ) async {
    await _submitAuthority(
      '/elections/${id.uuid}/cancellation',
      jsonEncode({'reason': reason}),
      guard,
    );
  }

  /// Entrega a contribuição assinada para o termo de encerramento.
  Future<Map<String, dynamic>> contribute(
    UuidValue id,
    UuidValue authority,
    String envelope,
    void Function() guard,
  ) => _submitAuthority(
    '/elections/${id.uuid}/tally/shares/${authority.uuid}',
    envelope,
    guard,
  );

  Future<Map<String, dynamic>> _submitAuthority(
    String path,
    String? envelope,
    void Function() guard,
  ) async {
    _requireHttps();
    guard();
    final response = await _dio.put<Map<String, dynamic>>(
      path,
      data: envelope,
      options: _authenticated(guard),
    );
    guard();
    return response.data ??
        (throw const FormatException('Confirmação de autoridade ausente'));
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
