/// Configura um cliente TLS isolado para a demonstração no Android.
library;

import 'dart:convert';
import 'dart:io';

import 'package:dio/dio.dart';
import 'package:dio/io.dart';

void configureDemoCa(Dio dio, String publicCaPemBase64) {
  final context = SecurityContext(withTrustedRoots: true)
    ..setTrustedCertificatesBytes(base64Decode(publicCaPemBase64));
  dio.httpClientAdapter = IOHttpClientAdapter(
    createHttpClient: () => HttpClient(context: context),
  );
}
