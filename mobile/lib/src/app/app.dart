/// API de composição e apresentação da aplicação Flutter.
///
/// Esta fachada reúne os componentes que integram módulos e plataforma na raiz
/// do aplicativo. Seu consumidor pode montar a aplicação sem importar cada
/// biblioteca de composição separadamente.
///
/// ## Divisão de responsabilidades
///
/// AppDependencies constrói os contratos e adaptadores e possui seus recursos.
/// AppRoot associa o encerramento dessa composição ao ciclo de vida da árvore.
/// CApp organiza tema, navegação e apresentação dos destinos da aplicação.
/// BackendReadinessController publica o diagnóstico global de prontidão em
/// paralelo à sessão; sua consulta não decide acesso às áreas do aplicativo.
///
/// A sequência esperada é preparar os serviços, criar e iniciar a composição
/// e então entregá-la ao proprietário visual. A restauração assíncrona continua
/// depois da montagem e é acompanhada pelos estados do controlador.
///
/// Esta API não substitui as fachadas dos módulos do domínio. O conhecimento de
/// implementações concretas permanece na composição, e a importação da fachada
/// não inicia serviços, registra listeners ou apresenta widgets.
library;

export 'capp.dart' show CApp;
export 'app_dependencies.dart' show AppDependencies;
export 'app_root.dart' show AppRoot;
export 'backend_readiness_controller.dart' show BackendReadinessController;
export 'backend_readiness_state.dart'
    show
        BackendReadinessState,
        BackendReadinessNotChecked,
        BackendReadinessChecking,
        BackendReadinessReady,
        BackendReadinessFailed;
