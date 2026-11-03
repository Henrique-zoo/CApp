/// API de montagem consumida pelo ponto de entrada do CApp.
///
/// Esta fachada oferece a main somente os componentes necessários para compor
/// e apresentar a aplicação: AppDependencies, AppRoot e CApp. O ponto de entrada
/// não precisa conhecer os caminhos dos módulos, seus contratos ou seus SDKs.
///
/// ## Lugar na arquitetura
///
/// A fachada encaminha as exportações da camada app. O bootstrap continua fora
/// dela porque prepara os serviços de plataforma antes de existir a composição.
/// Os módulos do domínio possuem suas próprias fachadas e não consomem esta
/// biblioteca para obter dependências globais.
///
/// No fluxo de montagem, o chamador executa o bootstrap, cria e inicia
/// AppDependencies e entrega a composição a AppRoot. As exportações apenas
/// disponibilizam tipos; importar esta biblioteca não executa nenhuma dessas etapas.
library;

export 'app/app.dart' show AppDependencies, AppRoot, CApp;
