/// Clientes das sondagens técnicas de saúde e prontidão do backend.
///
/// Esta biblioteca disponibiliza operações de diagnóstico independentes dos
/// casos de uso do domínio. Ela pertence ao transporte compartilhado porque
/// interpreta endpoints técnicos, sem estabelecer uma política de acesso ao app.
///
/// ## Operações disponíveis
///
/// getHealth solicita GET /health como texto e exige um corpo não vazio.
/// Isso confirma uma resposta do processo HTTP, sem verificar seus serviços
/// dependentes.
///
/// getReadiness solicita GET /ready, cujo backend consulta PostgreSQL.
/// O corpo é ignorado; somente HTTP 200 confirma prontidão. Redirecionamentos
/// não são seguidos para evitar tratar uma página intermediária como sucesso.
///
/// ## Uso e falhas
///
/// BackendApi expõe o cliente à composição, que consulta prontidão na abertura em
/// paralelo à restauração. Login e consultas ao domínio não exigem essa sondagem.
/// Uma resposta positiva não garante o sucesso de uma operação posterior.
///
/// O transporte recebido define URL e timeouts. Falhas HTTP propagam como
/// DioException, e health sem corpo utilizável produz FormatException.
/// A biblioteca não abre autenticação, persiste diagnósticos ou renova sessão.
library;

import 'package:dio/dio.dart';

/// Verificações técnicas do processo e de sua conexão PostgreSQL.
final class MonitoringApi {
  /// Cria o cliente com um transporte cuja URL base já esteja configurada.
  /// Não altera status aceitos, timeouts ou interceptors e não executa sondagens.
  const MonitoringApi(this._dio);

  /// Transporte compartilhado pertencente a BackendApi; este cliente não o fecha.
  final Dio _dio;

  /// GET /health: verifica o processo HTTP, sem verificar outros serviços.
  ///
  /// Retorna o corpo como texto quando ele está presente e não vazio.
  /// Falhas de transporte ou de status HTTP são propagadas como [DioException].
  /// Corpo nulo ou vazio lança [FormatException]. O método não verifica a mensagem
  /// literal prevista no contrato. O sucesso não comprova acesso ao PostgreSQL
  /// ou ao Microsoft Graph.
  Future<String> getHealth() async {
    final response = await _dio.get<String>(
      '/health',
      options: Options(responseType: ResponseType.plain),
    );

    final data = response.data;

    if (data == null || data.isEmpty) {
      throw const FormatException('Resposta de saúde sem corpo');
    }

    return data;
  }

  /// GET /ready: confirma que o backend consegue consultar PostgreSQL.
  ///
  /// Conclui sem valor somente para HTTP 200. Outros status, inclusive 503 e
  /// redirecionamentos, produzem [DioException]. Falhas de rede também propagam.
  /// O corpo é ignorado. O backend limita sua consulta de prontidão a dois
  /// segundos; esta operação limita a espera da resposta a cinco segundos. O
  /// timeout de conexão continua pertencendo ao transporte recebido.
  /// Não verifica cadastro institucional, migrations ou provedores externos.
  Future<void> getReadiness() async {
    await _dio.get<void>(
      '/ready',
      options: Options(
        receiveTimeout: const Duration(seconds: 5),
        followRedirects: false,
        validateStatus: (status) => status == 200,
      ),
    );
  }
}
