import 'package:go_router/go_router.dart';

import '../features/authentication/application/authentication_controller.dart';
import '../features/authentication/presentation/authentication_pages.dart';

/// Cria as rotas e impede acesso à área autenticada sem identidade validada.
GoRouter createAppRouter(AuthenticationController controller) {
  return GoRouter(
    initialLocation: '/',
    refreshListenable: controller,
    redirect: (context, routeState) {
      final location = routeState.matchedLocation;
      final status = controller.state.status;

      switch (status) {
        case AuthenticationStatus.restoring:
          return location == '/loading' ? null : '/loading';

        case AuthenticationStatus.signedOut:
          return location == '/login' ? null : '/login';

        case AuthenticationStatus.authenticated:
          return location == '/home' ? null : '/home';

        case AuthenticationStatus.signingIn:
          return location == '/login' ? null : '/login';

        case AuthenticationStatus.signingOut:
          return location == '/home' ? null : '/home';
      }
    },
    routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => const AuthenticationLoadingPage(),
      ),
      GoRoute(
        path: '/loading',
        builder: (context, state) => const AuthenticationLoadingPage(),
      ),
      GoRoute(
        path: '/login',
        builder: (context, state) =>
            InstitutionalLoginPage(controller: controller),
      ),
      GoRoute(
        path: '/home',
        builder: (context, state) =>
            AuthenticatedHomePage(controller: controller),
      ),
    ],
  );
}
