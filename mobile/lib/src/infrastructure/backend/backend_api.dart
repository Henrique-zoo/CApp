/// Composição do transporte HTTP compartilhado pelas APIs do backend.
///
/// Esta biblioteca concentra configuração e propriedade do cliente Dio. Clientes
/// de endpoints recebem esse transporte pronto; os módulos do domínio o consomem
/// por adaptadores próprios, sem levar Dio ou seus DTOs para a aplicação.
///
/// ## Configuração e uso
///
/// A construção define URL base, timeouts e FirebaseBearerInterceptor. A fachada
/// identity é criada no primeiro acesso e utiliza o mesmo Dio. Construir a
/// composição ou acessar uma fachada não faz I/O.
///
/// Cada endpoint escolhe se exige credencial por meio do marcador firebaseAuth.
/// O interceptor consulta o provedor de ID token somente para requisições marcadas.
///
/// ## Limites e ciclo de vida
///
/// O transporte conhece protocolo, credenciais e falhas HTTP, mas não decide
/// prontidão da sessão, navegação ou capacidades do usuário. A tradução dessas
/// falhas para os contratos de aplicação pertence aos adaptadores de cada módulo.
///
/// AppDependencies possui esta fachada e solicita close no encerramento.
/// A propriedade também inclui um Dio injetado. A biblioteca não executa logout,
/// limpa cache ou implementa repetição automática de requisições.
library;

import 'package:dio/dio.dart';

import 'auth/firebase_bearer_interceptor.dart';
import 'identity/identity.dart';

/// Fachada e ponto de configuração das operações HTTP do backend CApp.
///
/// `baseUrl` aponta para o servidor Rust, sem prefixo `/v1`.
/// A mesma instância do Dio é compartilhada por todos os clientes HTTP.
/// O transporte conhece credenciais de uma requisição, não um perfil persistido
/// do usuário. Não possui cache institucional, lógica de navegação ou repetição
/// automática após erro de autenticação.
final class BackendApi {
  /// Configura a URL, os timeouts e a autenticação do cliente HTTP.
  ///
  /// [baseUrl] deve apontar para a raiz do servidor, sem prefixo `/v1`.
  /// [getFirebaseIdToken] é consultado somente nas operações protegidas.
  /// Quando [dio] é fornecido, suas opções são alteradas e o interceptor
  /// Firebase é inserido antes dos interceptors existentes.
  ///
  /// ## Efeitos e limites
  ///
  /// Define conexão de cinco segundos e espera de resposta de vinte segundos.
  /// Não fixa `sendTimeout` nem valida disponibilidade da URL. Não adiciona um
  /// prefixo de versão aos caminhos. O chamador deve evitar reutilizar o mesmo
  /// [dio] para múltiplas construções se não quiser acumular interceptors.
  ///
  /// Assume a responsabilidade de fechar também um transporte injetado por
  /// [dio]. A política de status e os interceptors adicionais desse transporte
  /// continuam relevantes para o comportamento das fachadas.
  BackendApi({
    required String baseUrl,
    required FirebaseIdTokenProvider getFirebaseIdToken,
    Dio? dio,
  }) : _dio = dio ?? Dio() {
    _dio.options.baseUrl = baseUrl;
    _dio.options.connectTimeout = const Duration(seconds: 5);
    // O serviço institucional pode consumir 15 segundos no backend.
    _dio.options.receiveTimeout = const Duration(seconds: 20);
    _dio.interceptors.insert(0, FirebaseBearerInterceptor(getFirebaseIdToken));
  }

  final Dio _dio;

  /// Operações de verificação da identidade institucional.
  /// A criação tardia só encapsula o transporte; nenhuma chamada ocorre no acesso.
  late final IdentityApi identity = IdentityApi(_dio);

  /// Libera recursos do cliente quando ele deixa de ser utilizado.
  ///
  /// Também fecha um [Dio] recebido pelo construtor. Com [force] habilitado,
  /// encerra imediatamente as conexões ativas.
  /// Não executa logout nem cancela a persistência local. Os efeitos de chamadas
  /// repetidas e de requisições posteriores ao fechamento seguem o contrato do Dio.
  void close({bool force = false}) => _dio.close(force: force);
}
