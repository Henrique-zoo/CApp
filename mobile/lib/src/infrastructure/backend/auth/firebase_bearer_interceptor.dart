/// Inclusão de Firebase ID tokens nas requisições HTTP protegidas.
///
/// Esta biblioteca mantém a obtenção de credenciais no transporte, evitando
/// que cada cliente de endpoint repita acesso ao provedor e montagem do cabeçalho.
/// Pertence à infraestrutura compartilhada e recebe uma função, sem depender
/// diretamente do SDK Firebase ou de um módulo do domínio.
///
/// ## Fluxo de uma requisição
///
/// O cliente do endpoint marca extra['firebaseAuth'] com o booleano true.
/// O interceptor consulta FirebaseIdTokenProvider, exige token não nulo e não
/// vazio e substitui Authorization por Bearer seguido da credencial atual.
/// Requisições sem esse marcador seguem sem consultar o provedor.
///
/// O token não é conservado entre requisições. A política de restauração e
/// renovação da credencial pertence ao provedor injetado pela composição.
///
/// ## Falhas e fronteiras
///
/// Ausência de token rejeita a requisição antes do envio, com
/// MissingFirebaseIdTokenException encapsulada em DioException.
/// Falha de obtenção conserva o objeto e a pilha originais no erro do transporte.
/// O adaptador do módulo interpreta esse diagnóstico segundo sua aplicação.
///
/// O interceptor não valida assinatura do JWT, abre login, repete requisições
/// ou interpreta códigos públicos do backend. Também não concede autorização:
/// o servidor continua verificando cada operação protegida.
library;

import 'package:dio/dio.dart';

/// Obtém o Firebase ID Token atual, sem armazenar um token permanente.
///
/// Retorna `null` quando não existe uma sessão autenticada.
/// Falhas de obtenção devem permanecer como falhas do Future, sem conversão em
/// token ausente. O provedor define sua política de renovação e não deve registrar
/// o valor retornado. A implementação da composição consulta o SDK por requisição.
typedef FirebaseIdTokenProvider = Future<String?> Function();

/// Adiciona Authorization: Bearer somente nas operações protegidas.
///
/// O marcador `firebaseAuth` é definido explicitamente por cada endpoint.
/// A API key de configuração do Firebase SDK não é enviada ao backend.
/// Não armazena uma credencial permanente, não repete requisições e não interpreta
/// os códigos da resposta HTTP. Operações públicas não consultam o provedor.
final class FirebaseBearerInterceptor extends Interceptor {
  /// Cria o interceptor com a função que consulta o token da sessão atual.
  /// Não executa a função durante a construção.
  FirebaseBearerInterceptor(this._provideToken);

  final FirebaseIdTokenProvider _provideToken;

  /// Inclui o token no cabeçalho das requisições marcadas como protegidas.
  ///
  /// Rejeita a requisição com [DioException] se o token for nulo, vazio ou
  /// se sua obtenção falhar. Requisições públicas seguem sem consultar o token.
  ///
  /// ## Efeitos e falhas
  ///
  /// Somente o booleano `true` ativa a autenticação; o token retornado substitui
  /// `Authorization`. Não remove um cabeçalho preexistente de operações públicas.
  /// Token nulo ou vazio gera `DioExceptionType.unknown` com [MissingFirebaseIdTokenException]; falha
  /// do provedor é encapsulada com objeto e pilha originais preservados.
  ///
  /// A assinatura `void` pertence ao hook do Dio. O processamento assíncrono
  /// conclui chamando `handler.next` ou `handler.reject`, sem retornar um token
  /// ou lançar uma resposta de negócio diretamente ao chamador da fachada.
  @override
  void onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    if (options.extra['firebaseAuth'] != true) {
      handler.next(options);
      return;
    }

    try {
      final token = await _provideToken();
      if (token == null || token.isEmpty) {
        handler.reject(
          DioException(
            requestOptions: options,
            type: DioExceptionType.unknown,
            error: const MissingFirebaseIdTokenException(),
          ),
        );
        return;
      }
      options.headers['Authorization'] = 'Bearer $token';
      handler.next(options);
    } catch (error, stackTrace) {
      handler.reject(
        DioException(
          requestOptions: options,
          type: DioExceptionType.unknown,
          error: error,
          stackTrace: stackTrace,
        ),
      );
    }
  }
}

/// Não há Firebase ID token disponível para uma requisição protegida.
///
/// Representa ausência de credencial no instante da requisição.
/// Não contém token, UID ou dependências do SDK Firebase.
final class MissingFirebaseIdTokenException implements Exception {
  /// Cria o diagnóstico de credencial ausente.
  const MissingFirebaseIdTokenException();

  /// Descreve a ausência sem incluir informações da conta.
  @override
  String toString() {
    return 'Não há credencial Firebase disponível para a requisição';
  }
}
