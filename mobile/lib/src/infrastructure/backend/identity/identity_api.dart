/// Cliente de protocolo da verificação institucional do backend.
///
/// Esta biblioteca implementa o endpoint HTTP descrito no contrato OpenAPI.
/// Pertence ao transporte compartilhado: conhece caminho, formato da requisição,
/// política de status e DTOs, mas não determina o efeito da resposta sobre
/// a sessão do aplicativo.
///
/// ## Caminho de uma chamada
///
/// IdentityApi recebe Dio já configurado e envia POST /auth/institutional/verify.
/// O JSON contém microsoftAccessToken; o marcador firebaseAuth solicita ao
/// interceptor que acrescente o ID token Firebase em Authorization.
///
/// Somente HTTP 200 é tratado como sucesso. O corpo precisa ser um objeto JSON
/// e é desserializado em InstitutionalVerificationResponse.
/// Corpos de respostas rejeitadas continuam disponíveis ao gateway.
///
/// ## Fronteira com o módulo
///
/// BackendInstitutionalIdentityGateway converte o DTO em modelo de aplicação
/// e classifica falhas reconhecidas. O cliente HTTP propaga DioException e
/// falhas de formato; não decide se a tela permanece na entrada.
///
/// A operação não provisiona usuário interno ou cria uma sessão independente
/// no backend. O cliente também não persiste o perfil, renova tokens Microsoft
/// ou repete automaticamente requisições canceladas.
library;

import 'package:dio/dio.dart';

import 'dto/institutional_auth_request.dart';
import 'dto/institutional_verification_response.dart';

/// Endpoints HTTP do módulo de identidade definidos no OpenAPI.
final class IdentityApi {
  /// Cria o cliente com um transporte configurado para autenticação Firebase.
  /// Não altera a URL ou instala interceptors. Um [Dio] sem o interceptor ainda
  /// envia o marcador em `extra`, mas não obtém por isso um cabeçalho de autorização.
  const IdentityApi(this._dio);

  final Dio _dio;

  /// Verifica a identidade institucional pelo endpoint existente.
  ///
  /// Envia [request] com autenticação Firebase solicitada ao interceptor.
  /// [cancelToken] permite cancelar o transporte, sem garantir reversão
  /// de operações já concluídas no servidor.
  ///
  /// ## Retorno e efeitos
  ///
  /// Retorna a resposta desserializada quando o endpoint responde com HTTP 200,
  /// único status considerado bem-sucedido.
  ///
  /// Respostas de erro permanecem disponíveis para interpretação pelo adaptador.
  /// Não persiste dados nem decide a prontidão da sessão.
  ///
  /// ## Pré-condições e erros
  ///
  /// Exige autenticação Firebase disponível para o interceptor.
  ///
  /// Lança [DioException] para falhas de transporte ou status inesperado.
  /// Lança [FormatException] quando o corpo de sucesso não é um objeto JSON.
  /// Falhas de campos do objeto propagam pela desserialização checked.
  ///
  /// O cancelamento do transporte não garante a reversão de operações
  /// já executadas no backend.
  Future<InstitutionalVerificationResponse> verifyInstitutionalIdentity({
    required InstitutionalAuthRequest request,
    CancelToken? cancelToken,
  }) async {
    final response = await _dio.post<Object?>(
      '/auth/institutional/verify',
      data: request.toJson(),
      options: Options(
        contentType: Headers.jsonContentType,
        responseType: ResponseType.json,
        receiveDataWhenStatusError: true,
        validateStatus: (status) => status == 200,
        extra: const {'firebaseAuth': true},
      ),
      cancelToken: cancelToken,
    );

    final data = response.data;

    if (data is! Map<String, dynamic>) {
      throw const FormatException('Resposta institucional sem objeto JSON');
    }
    return InstitutionalVerificationResponse.fromJson(data);
  }
}
