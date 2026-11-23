/// O navegador usa o repositório de confiança da própria plataforma.
library;

import 'package:dio/dio.dart';

void configureDemoCa(Dio dio, String publicCaPemBase64) {
  throw UnsupportedError(
    'A CA local da demonstração exige o transporte nativo.',
  );
}
