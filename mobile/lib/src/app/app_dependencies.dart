/// Composição dos contratos, adaptadores e recursos compartilhados do CApp.
///
/// Esta biblioteca é a raiz de composição: conhece as implementações concretas
/// para que aplicação e apresentação possam trabalhar com contratos. As decisões
/// sobre qual SDK, transporte e armazenamento utilizar ficam concentradas aqui,
/// em vez de serem repetidas em telas ou casos de uso.
///
/// ## Construção do grafo
///
/// A criação assíncrona inicializa o MSAL em modo múltiplo e restaura o índice de
/// contas Firebase. FirebaseAccountAuth cria contêineres nomeados do mesmo projeto;
/// cada delegado FirebaseMicrosoftAuth recebe o provedor MSAL associado ao objeto
/// Microsoft dessa conta, preservando as sessões das demais.
///
/// A composição conecta uma instância FirebaseAccountAuth às portas de autenticação
/// e login Microsoft e à obtenção de ID tokens do transporte. A mesma identidade
/// é, portanto, usada na observação da conta e nas requisições protegidas.
///
/// BackendApi fornece o cliente HTTP ao gateway institucional. O gateway e o
/// adaptador Firebase alimentam InstitutionalAuthService; esse serviço e o
/// SessionStore alimentam SessionController. O serviço permanece interno à
/// composição, preservando o controlador como entrada das operações da interface.
///
/// ## Criação, início e encerramento
///
/// create deve ser aguardado depois do bootstrap, sem autenticar ou consultar o
/// backend. start registra o acompanhamento da sessão e solicita uma consulta de
/// prontidão em paralelo. Ambos publicam resultados assíncronos independentes.
/// AppRoot recebe a composição iniciada e solicita seu dispose.
/// A restauração da sessão pode iniciar renovação silenciosa conforme a evidência
/// encontrada; essa decisão pertence ao controlador, não à composição.
///
/// A consulta GET /ready informa disponibilidade ao usuário, sem condicionar
/// restauração, login ou acesso ao perfil válido em cache. Não consulta /health
/// e não repete sondagens ao retornar do navegador Microsoft.
///
/// A composição possui controlador, adaptador de seleção e transporte. O SDK possui
/// as sessões persistentes e o aplicativo Firebase global. O encerramento é compartilhado entre chamadas: aguarda o controlador
/// e fecha o transporte mesmo se aquele falhar. Não executa logout ou limpa cache;
/// I/O local já iniciado pode concluir sem publicar estado depois do descarte.
library;

import 'package:firebase_auth/firebase_auth.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_app_check/firebase_app_check.dart';

import '../infrastructure/firebase/firebase_app_check_setup.dart';

import 'package:shared_preferences/shared_preferences.dart';

import '../infrastructure/backend/backend_api.dart';
import '../modules/identity/application/institutional_auth_service.dart';
import '../modules/identity/identity.dart';
import '../modules/identity/infrastructure/backend/backend_institutional_identity_gateway.dart';
import '../modules/identity/infrastructure/firebase/firebase_account_auth.dart';
import '../modules/identity/infrastructure/preferences/account_registry.dart';
import '../modules/identity/infrastructure/preferences/preferences_account_profile_store.dart';
import '../modules/identity/infrastructure/preferences/preferences_session_store.dart';
import '../modules/identity/infrastructure/microsoft/msal_client.dart';
import '../modules/identity/infrastructure/microsoft/msal_microsoft_token_provider.dart';
import 'backend_readiness_controller.dart';

/// Proprietário do transporte e dos controladores compartilhados.
///
/// Deve ser criado depois do binding Flutter e do bootstrap Firebase.
/// O chamador inicia com [start] e assume [dispose], normalmente por AppRoot.
///
/// O serviço institucional fica interno à composição para que a interface
/// solicite entrada pelo controlador, preservando estado, cache e concorrência.
final class AppDependencies {
  /// Assume os recursos construídos pela fábrica.
  AppDependencies._({
    required this._backendApi,
    required this._accountAuth,
    required this.sessionController,
    required this.backendReadinessController,
  });

  final BackendApi _backendApi;
  final FirebaseAccountAuth _accountAuth;

  /// Coordenação exposta à interface pela API pública do módulo.
  ///
  /// Construir a composição não inicia seu acompanhamento.
  final SessionController sessionController;

  /// Diagnóstico global de prontidão, independente da coordenação da sessão.
  ///
  /// A consulta inicial é solicitada por [start]; a interface pode pedir nova
  /// tentativa. A composição possui o controlador e interrompe suas notificações
  /// antes de fechar o transporte, ignorando conclusões tardias.
  final BackendReadinessController backendReadinessController;

  /// Future compartilhado da primeira solicitação de encerramento.
  Future<void>? _disposeFuture;

  /// Inicializa o MSAL e conecta os recursos compartilhados.
  ///
  /// Deve ser aguardado depois do bootstrap Firebase.
  /// A inicialização não abre interfaces de autenticação.
  ///
  /// Falhas de configuração ou inicialização são propagadas.
  /// [backendBaseUrl] aponta para a raiz HTTP, sem prefixo de versão.
  /// [microsoftClientId] e [microsoftAndroidRedirectUri] são configurações públicas
  /// do registro Android; valores vazios são rejeitados antes de montar a interface.
  /// Os recursos retornados são compartilhados até [dispose].
  static Future<AppDependencies> create({
    required String backendBaseUrl,
    required String microsoftClientId,
    required String microsoftAndroidRedirectUri,
  }) async {
    final msalClient = await createMsalClient(
      clientId: microsoftClientId,
      androidRedirectUri: microsoftAndroidRedirectUri,
    );

    final preferences = SharedPreferencesAsync();
    final firebaseMicrosoftAuth = await FirebaseAccountAuth.create(
      registry: AccountRegistry(preferences),
      factory: (name) async {
        final existing = Firebase.apps.where((app) => app.name == name);
        final app = existing.isNotEmpty
            ? existing.first
            : await Firebase.initializeApp(
                name: name,
                options: Firebase.app().options,
              );

        await initializeFirebaseAppCheck(app);

        return FirebaseAuth.instanceFor(app: app);
      },
      tokensFor: (auth, {microsoftObjectId}) {
        final identities = auth.currentUser?.providerData.where(
          (p) => p.providerId == 'microsoft.com',
        );
        final objectId = identities == null || identities.isEmpty
            ? null
            : identities.first.uid;

        return MsalMicrosoftTokenProvider.forAccount(
          msalClient,
          accountId: objectId ?? microsoftObjectId,
        );
      },
    );
    final sessionStore = PreferencesSessionStore(preferences);

    final backendApi = BackendApi(
      baseUrl: backendBaseUrl,
      getFirebaseIdToken: firebaseMicrosoftAuth.getFirebaseIdToken,
      getFirebaseAppCheckToken: () => FirebaseAppCheck.instance.getToken(),
    );

    final identityGateway = BackendInstitutionalIdentityGateway(
      backendApi.identity,
    );

    final authService = InstitutionalAuthService(
      identityGateway: identityGateway,
      microsoftSignIn: firebaseMicrosoftAuth,
      authSession: firebaseMicrosoftAuth,
    );

    return AppDependencies._(
      backendApi: backendApi,
      accountAuth: firebaseMicrosoftAuth,
      backendReadinessController: BackendReadinessController(
        checkReadiness: backendApi.monitoring.getReadiness,
      ),
      sessionController: SessionController(
        authSession: firebaseMicrosoftAuth,
        sessionStore: sessionStore,
        authService: authService,
        accountAuthentication: firebaseMicrosoftAuth,
        accountGateway: identityGateway,
        accountProfileStore: PreferencesAccountProfileStore(preferences),
      ),
    );
  }

  /// Inicia a sessão e a consulta de prontidão sem aguardar os resultados.
  ///
  /// Cada controlador publica sua conclusão ou falha independentemente. Chamadas
  /// repetidas preservam as guardas de início e não repetem a consulta automática.
  /// Lança StateError depois de solicitar encerramento; outras falhas síncronas
  /// não capturadas propagam. Falha de prontidão não desfaz a restauração local.
  void start() {
    if (_disposeFuture != null) {
      throw StateError('As dependências estão sendo ou já foram encerradas');
    }

    sessionController.start();
    backendReadinessController.start();
  }

  /// Solicita encerramento e compartilha seu resultado entre chamadas.
  ///
  /// Interrompe notificações de prontidão, aguarda o controlador de sessão e
  /// fecha o transporte mesmo se aquele falhar. Não aguarda a consulta de rede.
  /// Não executa logout, remove cache ou desfaz efeitos no backend.
  /// O proprietário deve aguardar ou acompanhar a falha do Future.
  Future<void> dispose() => _disposeFuture ??= _disposeResources();

  /// Fecha os recursos próprios na ordem inversa de sua utilização.
  ///
  /// Falhas propagam; se o fechamento do transporte também lançar no finally,
  /// essa última exceção substitui a anterior segundo a semântica do Dart.
  Future<void> _disposeResources() async {
    backendReadinessController.dispose();

    try {
      await sessionController.dispose();
    } finally {
      try {
        await _accountAuth.dispose();
      } finally {
        _backendApi.close(force: true);
      }
    }
  }
}
