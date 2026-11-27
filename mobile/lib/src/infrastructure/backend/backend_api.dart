/// Composição do transporte HTTP compartilhado pelas APIs do backend.
///
/// Esta biblioteca concentra configuração e propriedade do cliente Dio. Clientes
/// de endpoints recebem esse transporte pronto; os módulos do domínio o consomem
/// por adaptadores próprios, sem levar Dio ou seus DTOs para a aplicação.
///
/// ## Configuração e uso
///
/// A construção define URL base, timeouts e interceptors de App Check e Bearer. As fachadas
/// identity e monitoring são criadas no primeiro acesso, compartilhando a mesma
/// instância de Dio. Construir a composição ou acessar uma fachada não faz I/O.
///
/// Cada endpoint marca firebaseAuth e firebaseAppCheck para consultar os provedores
/// correspondentes. App Check comprova o aplicativo; Bearer identifica a conta.
/// A composição solicita prontidão em paralelo à sessão na abertura. A sondagem
/// não condiciona o login; saúde continua disponível para consultas explícitas.
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
import 'auth/firebase_app_check_interceptor.dart';
import 'elections/ballot_api.dart';
import 'elections/registration_api.dart';
import 'identity/identity.dart';
import 'materials/materials_api.dart';
import 'monitoring/monitoring.dart';
import 'tls/backend_trust_stub.dart'
    if (dart.library.io) 'tls/backend_trust_io.dart';

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
  /// [baseUrl] deve apontar para a raiz HTTPS do servidor, sem prefixo `/v1`.
  /// [trustedBackendCertPemBase64] aceita o certificado PEM público do backend
  /// de demonstração codificado em base64. A confiança adicional se aplica
  /// apenas a esta API e preserva as raízes da plataforma.
  /// [getFirebaseIdToken] é consultado somente nas operações protegidas.
  /// [getFirebaseAppCheckToken] fornece o atestado do aplicativo nessas operações.
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
    required FirebaseAppCheckTokenProvider getFirebaseAppCheckToken,
    String trustedBackendCertPemBase64 = const String.fromEnvironment(
      'BACKEND_TLS_TRUST_PEM_BASE64',
    ),
    Dio? dio,
  }) : _dio = dio ?? Dio() {
    final server = Uri.tryParse(baseUrl);
    if (server == null ||
        !server.isScheme('https') ||
        server.host.isEmpty ||
        server.userInfo.isNotEmpty) {
      throw ArgumentError(
        'A URL do backend deve usar HTTPS com um host válido e sem credenciais.',
      );
    }
    if (trustedBackendCertPemBase64.isNotEmpty) {
      configureBackendTrust(_dio, trustedBackendCertPemBase64);
    }
    _dio.options.baseUrl = baseUrl;
    _dio.options.connectTimeout = const Duration(seconds: 5);
    // O serviço institucional pode consumir 15 segundos no backend.
    _dio.options.receiveTimeout = const Duration(seconds: 20);

    _dio.interceptors.insertAll(0, [
      InterceptorsWrapper(
        onRequest: (request, handler) {
          // Um caminho absoluto ou uma opção local do Dio não pode desviar
          // os tokens Firebase nem o conteúdo de um POST para outro servidor.
          final destination = request.uri;
          if (!destination.isScheme('https') ||
              destination.origin != server.origin ||
              destination.userInfo.isNotEmpty) {
            handler.reject(
              DioException(
                requestOptions: request,
                error: const FormatException(
                  'Destino fora da origem HTTPS do backend.',
                ),
              ),
            );
            return;
          }
          request.followRedirects = false;
          handler.next(request);
        },
      ),
      FirebaseAppCheckInterceptor(getFirebaseAppCheckToken),
      FirebaseBearerInterceptor(getFirebaseIdToken),
    ]);
  }

  final Dio _dio;

  /// Operações de verificação da identidade institucional.
  /// A criação tardia só encapsula o transporte; nenhuma chamada ocorre no acesso.
  late final IdentityApi identity = IdentityApi(_dio);

  late final RegistrationApi voterRegistration = RegistrationApi(_dio);
  late final BallotApi ballots = BallotApi(_dio);

  late final MaterialsApi materials = MaterialsApi(_dio);

  /// Verificações públicas de saúde e prontidão do backend.
  /// A composição consulta prontidão na abertura, sem bloquear login ou cache.
  late final MonitoringApi monitoring = MonitoringApi(_dio);

  /// Libera recursos do cliente quando ele deixa de ser utilizado.
  ///
  /// Também fecha um [Dio] recebido pelo construtor. Com [force] habilitado,
  /// encerra imediatamente as conexões ativas.
  /// Não executa logout nem cancela a persistência local. Os efeitos de chamadas
  /// repetidas e de requisições posteriores ao fechamento seguem o contrato do Dio.
  void close({bool force = false}) => _dio.close(force: force);
}
