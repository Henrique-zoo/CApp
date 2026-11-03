/// Transporte e respostas HTTP em memória compartilhados pelos testes.
///
/// Mantém Dio e seus transformadores reais, substituindo apenas a rede.
/// As credenciais e identidades das fixtures são fictícias.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';

/// Cria um cliente de teste e registra seu encerramento na suíte.
///
/// [adapter] possui a resposta simulada. Nenhum socket é criado.
/// O cliente usa um domínio reservado e não instala autenticação implicitamente.
Dio createTestDio(FakeHttpAdapter adapter) {
  final dio = Dio(BaseOptions(baseUrl: 'https://backend.example.invalid'));
  dio.httpClientAdapter = adapter;
  addTearDown(() => dio.close(force: true));
  return dio;
}

/// Resposta institucional do protocolo, independente dos modelos da aplicação.
///
/// [student] seleciona a variante estudantil, preservando curso nulo.
Map<String, Object?> backendVerificationBody({bool student = false}) => {
  'profile': {
    'microsoftObjectId': '00000000-0000-0000-0000-000000000001',
    'name': 'Pessoa Exemplo',
    'email': null,
    'affiliation': student
        ? <String, Object?>{'type': 'student', 'courseName': null}
        : <String, Object?>{'type': 'nonStudent'},
  },
  'expiresAt': '2026-01-01T13:00:00Z',
};

/// Envelope público de erro com código controlado e sem diagnósticos internos.
Map<String, Object?> backendErrorBody(String code) => {
  'error': {
    'code': code,
    'message': 'Mensagem pública do backend',
    'details': null,
  },
};

/// Serializa [body] e identifica o conteúdo como JSON para os transformadores.
///
/// [status] controla a resposta e permite verificar a política de validateStatus.
ResponseBody jsonTestResponse(Object? body, {int status = 200}) =>
    ResponseBody.fromString(
      jsonEncode(body),
      status,
      headers: {
        Headers.contentTypeHeader: ['application/json'],
      },
    );

/// Implementa o limite de rede sem substituir serialização ou interceptors.
final class FakeHttpAdapter implements HttpClientAdapter {
  /// Recebe a função que entrega a resposta ou lança uma falha simulada.
  FakeHttpAdapter(this.respond);

  /// Comportamento controlado do servidor.
  final FutureOr<ResponseBody> Function(RequestOptions options) respond;

  /// Requisições que efetivamente alcançaram o transporte.
  final requests = <RequestOptions>[];

  /// Registra a requisição antes de entregar o comportamento simulado.
  ///
  /// Não consome o corpo transmitido e não implementa cancelamento próprio.
  /// Falhas da função recebida propagam para o processamento normal de Dio.
  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    requests.add(options);
    return respond(options);
  }

  /// Não há conexões externas para liberar.
  @override
  void close({bool force = false}) {}
}
