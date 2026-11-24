/// O navegador usa o repositório de confiança da própria plataforma.
library;

import 'package:dio/dio.dart';

/// Sinaliza que o navegador não permite adicionar confiança a um cliente Dio.
void configureBackendTrust(Dio dio, String publicPemBase64) {
  throw UnsupportedError(
    'Configure a confiança no certificado do backend no próprio navegador.',
  );
}
