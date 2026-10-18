import 'package:flutter/material.dart';

import '../application/authentication_controller.dart';

/// Tela transitória exibida enquanto a sessão persistida é validada.
class AuthenticationLoadingPage extends StatelessWidget {
  const AuthenticationLoadingPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 20),
            Text('Verificando sua sessão...'),
          ],
        ),
      ),
    );
  }
}

/// Tela inicial de autenticação institucional.
class InstitutionalLoginPage extends StatelessWidget {
  const InstitutionalLoginPage({required this.controller, super.key});

  final AuthenticationController controller;

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: controller,
      builder: (context, _) {
        final state = controller.state;
        final isSigningIn = state.status == AuthenticationStatus.signingIn;

        return Scaffold(
          body: SafeArea(
            child: Center(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(24),
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 440),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Icon(
                        Icons.school_outlined,
                        size: 68,
                        color: Theme.of(context).colorScheme.primary,
                      ),
                      const SizedBox(height: 24),
                      Text(
                        'CApp',
                        textAlign: TextAlign.center,
                        style: Theme.of(context).textTheme.headlineLarge
                            ?.copyWith(fontWeight: FontWeight.bold),
                      ),
                      const SizedBox(height: 12),
                      Text(
                        'Entre com sua conta institucional',
                        textAlign: TextAlign.center,
                        style: Theme.of(context).textTheme.titleLarge,
                      ),
                      const SizedBox(height: 12),
                      Text(
                        'Use sua conta institucional da Universidade de Brasília (UnB) para acessar os serviços acadêmicos.',
                        textAlign: TextAlign.center,
                        style: Theme.of(context).textTheme.bodyLarge,
                      ),
                      const SizedBox(height: 32),
                      if (state.message != null) ...[
                        Semantics(
                          liveRegion: true,
                          child: Container(
                            padding: const EdgeInsets.all(12),
                            decoration: BoxDecoration(
                              color: Theme.of(context)
                                  .colorScheme
                                  .errorContainer,
                              borderRadius: BorderRadius.circular(12),
                            ),
                            child: Text(
                              state.message!,
                              key: const ValueKey('authentication-error'),
                              style: TextStyle(
                                color: Theme.of(context)
                                    .colorScheme
                                    .onErrorContainer,
                              ),
                              textAlign: TextAlign.center,
                            ),
                          ),
                        ),
                        const SizedBox(height: 16),
                      ],
                      FilledButton(
                        key: const ValueKey('microsoft-login-button'),
                        onPressed: isSigningIn ? null : controller.signIn,
                        child: isSigningIn
                            ? const Row(
                                mainAxisAlignment: MainAxisAlignment.center,
                                children: [
                                  SizedBox(
                                    width: 20,
                                    height: 20,
                                    child: CircularProgressIndicator(
                                      strokeWidth: 2,
                                    ),
                                  ),
                                  SizedBox(width: 12),
                                  Text('Entrando...'),
                                ],
                              )
                            : const Row(
                                mainAxisAlignment: MainAxisAlignment.center,
                                children: [
                                  Icon(Icons.login),
                                  SizedBox(width: 10),
                                  Text('Entrar com a Microsoft'),
                                ],
                              ),
                      ),
                      const SizedBox(height: 16),
                      Text(
                        'Somente identidades aceitas pelo serviço de autenticação podem acessar o CApp.',
                        textAlign: TextAlign.center,
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

/// Área inicial exibida depois que o backend confirma a identidade.
class AuthenticatedHomePage extends StatelessWidget {
  const AuthenticatedHomePage({required this.controller, super.key});

  final AuthenticationController controller;

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: controller,
      builder: (context, _) {
        final state = controller.state;
        final identity = state.identity;
        final isSigningOut = state.status == AuthenticationStatus.signingOut;

        return Scaffold(
          appBar: AppBar(title: const Text('CApp')),
          body: SafeArea(
            child: Center(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(24),
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 480),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Icon(
                        Icons.verified_user_outlined,
                        size: 64,
                        color: Theme.of(context).colorScheme.primary,
                      ),
                      const SizedBox(height: 24),
                      Text(
                        'Sessão autenticada',
                        textAlign: TextAlign.center,
                        style: Theme.of(context).textTheme.headlineSmall,
                      ),
                      const SizedBox(height: 12),
                      Text(
                        identity?.email ?? 'Conta institucional autenticada',
                        key: const ValueKey('authenticated-email'),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 24),
                      if (state.message != null) ...[
                        Semantics(
                          liveRegion: true,
                          child: Text(
                            state.message!,
                            textAlign: TextAlign.center,
                          ),
                        ),
                        const SizedBox(height: 16),
                      ],
                      OutlinedButton(
                        key: const ValueKey('logout-button'),
                        onPressed: isSigningOut ? null : controller.signOut,
                        child: isSigningOut
                            ? const SizedBox(
                                width: 20,
                                height: 20,
                                child: CircularProgressIndicator(
                                  strokeWidth: 2,
                                ),
                              )
                            : const Text('Sair da conta'),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
