import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../features/authentication/application/authentication_controller.dart';
import '../features/authentication/data/firebase_authentication_repository.dart';
import '../features/authentication/domain/authentication_repository.dart';
import 'routes.dart';
import 'theme.dart';

/// Aplicativo CApp.
class CApp extends StatefulWidget {
  const CApp({super.key, this.authenticationRepository});

  /// Permite fornecer outro repositório, por exemplo em testes.
  final AuthenticationRepository? authenticationRepository;

  @override
  State<CApp> createState() => _CAppState();
}

class _CAppState extends State<CApp> {
  late final AuthenticationController _controller;
  late final GoRouter _router;

  @override
  void initState() {
    super.initState();

    final repository =
        widget.authenticationRepository ?? FirebaseAuthenticationRepository();

    _controller = AuthenticationController(repository);
    _router = createAppRouter(_controller);
    unawaited(_controller.restoreSession());
  }

  @override
  void dispose() {
    _router.dispose();
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      title: 'CApp',
      theme: buildAppTheme(),
      routerConfig: _router,
    );
  }
}
