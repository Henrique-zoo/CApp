/// Acrescenta confiança explícita ao transporte HTTPS nativo do backend.
library;

import 'dart:convert';
import 'dart:io';

import 'package:dio/dio.dart';
import 'package:dio/io.dart';

/// Configura [dio] com a confiança da plataforma e o PEM público informado.
/// O [HttpClient] continua verificando o servidor e o nome da URL no handshake.
void configureBackendTrust(Dio dio, String publicPemBase64) {
  final context = SecurityContext(withTrustedRoots: true)
    ..setTrustedCertificatesBytes(base64Decode(publicPemBase64));
  dio.httpClientAdapter = IOHttpClientAdapter(
    createHttpClient: () => HttpClient(context: context),
  );
}
