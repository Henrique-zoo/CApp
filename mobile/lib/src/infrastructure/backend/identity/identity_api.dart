/// Cliente de protocolo do acesso institucional e do perfil interno no backend.
///
/// Esta biblioteca implementa os endpoints de identidade descritos no OpenAPI.
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
/// Somente HTTP 200 é tratado como sucesso. O corpo da verificação precisa ser
/// um objeto JSON e é desserializado em [InstitutionalAuthResponse].
/// Corpos de respostas rejeitadas continuam disponíveis ao gateway.
/// A restauração usa GET /auth/session e conserva a possibilidade de evidência
/// nula. A conclusão usa PUT /users/me/profile depois da autenticação por senha;
/// o e-mail escolhido vem do JWT no backend, não do corpo desse formulário.
/// Todas as operações marcam a necessidade de Firebase Auth e App Check para
/// os interceptors configurados no transporte.
///
/// ## Fronteira com o módulo
///
/// BackendInstitutionalIdentityGateway converte o DTO em modelo de aplicação
/// e classifica falhas reconhecidas. O cliente HTTP propaga DioException e
/// falhas de formato; não decide se a tela permanece na entrada.
///
/// A verificação devolve a conta provisionada e a evidência na mesma resposta.
/// O Firebase continua administrando a sessão. A sugestão de e-mail do Graph
/// acompanha apenas a resposta de cadastro, separada da evidência. O cliente não
/// persiste perfis, renova tokens Microsoft ou repete requisições canceladas.
library;

import 'package:dio/dio.dart';

import 'dto/institutional_auth_request.dart';
import 'dto/account_session_response.dart';
import 'dto/institutional_auth_response.dart';

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
  Future<InstitutionalAuthResponse> verifyInstitutionalIdentity({
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
        extra: const {'firebaseAuth': true, 'firebaseAppCheck': true},
      ),
      cancelToken: cancelToken,
    );

    final data = response.data;

    if (data is! Map<String, dynamic>) {
      throw const FormatException('Resposta institucional sem objeto JSON');
    }

    return InstitutionalAuthResponse.fromJson(data);
  }

  /// Recupera a conta e a evidência reutilizável por GET /auth/session.
  ///
  /// O transporte deve estar associado à conta Firebase atual e possuir os
  /// interceptors de identidade e App Check. Não recebe token Microsoft nem pede
  /// ao cliente uma nova autorização para iniciar esta chamada.
  ///
  /// Retorna [AccountSessionResponse] em HTTP 200. Evidência nula é um resultado
  /// válido para uma conta já provisionada e orienta sincronização na aplicação;
  /// conta ausente chega como HTTP 404, não como um DTO vazio. A operação não
  /// estende o prazo da evidência nem recupera a sugestão de e-mail do cadastro.
  ///
  /// Propaga [DioException] para falhas de transporte ou status diferente de 200,
  /// preservando a resposta de erro para o gateway. [FormatException] sinaliza
  /// envelope de sucesso incompatível; falhas das fábricas aninhadas também
  /// propagam. Não classifica esses erros como decisões de sessão ou escreve cache.
  Future<AccountSessionResponse> restoreSession() async =>
      AccountSessionResponse.fromJson(await _accountRequest('/auth/session'));

  /// Solicita a conclusão ou atualização do perfil por PUT /users/me/profile.
  ///
  /// [nickname] é o apelido escolhido, [currentSemester] é o período atual e
  /// [institutionEntrySemester] é o primeiro ingresso em AAAA.1/AAAA.2. [name]
  /// só deve completar uma ausência do Graph; nulo omite a propriedade do JSON.
  /// Os valores são enviados sem normalização ou validação de domínio local.
  ///
  /// A credencial de senha precisa ter sido vinculada/autenticada no mesmo UID
  /// Firebase, e o backend exige confirmação institucional vigente. O transporte
  /// obtém ID token e App Check pelos interceptors. Senha, UID, estado, curso e
  /// e-mail não são incluídos no formulário; o servidor obtém o endereço do JWT.
  ///
  /// Retorna [InternalUserResponse] para HTTP 200, que o gateway ainda deve
  /// converter para o modelo validado. [DioException] preserva falhas HTTP/rede;
  /// [FormatException] e falhas de leitura dos DTOs propagam para classificação.
  /// Uma falha não é convertida em sucesso nem repetida automaticamente. Esta
  /// biblioteca não muda a sessão, salva perfil ou altera credenciais Firebase.
  Future<InternalUserResponse> completeProfile({
    required String nickname,
    required int currentSemester,
    required String institutionEntrySemester,
    String? name,
  }) async => InternalUserResponse.fromJson(
    await _accountRequest(
      '/users/me/profile',
      data: {
        'nickname': nickname,
        'currentSemester': currentSemester,
        'institutionEntrySemester': institutionEntrySemester,
        'name': ?name,
      },
    ),
  );

  /// Compartilha a configuração de transporte das operações de conta.
  ///
  /// [path] é o caminho do endpoint selecionado pelo método público. Sem [data]
  /// usa GET; com dados, inclusive mapa vazio, usa PUT com conteúdo JSON. Solicita
  /// os dois interceptors em `extra` e aceita somente HTTP 200.
  ///
  /// Retorna o objeto JSON para a fábrica específica, sem interpretar estado de
  /// conta ou evidência. Lança [FormatException] se o sucesso não contiver objeto;
  /// erros de Dio propagam com resposta e diagnóstico originais. Não configura
  /// SDKs, controla a validade dos tokens ou repete a requisição.
  Future<Map<String, dynamic>> _accountRequest(
    String path, {
    Map<String, Object>? data,
  }) async {
    final response = await _dio.request<Object?>(
      path,
      data: data,
      options: Options(
        method: data == null ? 'GET' : 'PUT',
        contentType: Headers.jsonContentType,
        responseType: ResponseType.json,
        validateStatus: (status) => status == 200,
        receiveDataWhenStatusError: true,
        extra: const {'firebaseAuth': true, 'firebaseAppCheck': true},
      ),
    );

    if (response.data is! Map<String, dynamic>) {
      throw const FormatException('Resposta sem objeto JSON');
    }

    return response.data! as Map<String, dynamic>;
  }
}
