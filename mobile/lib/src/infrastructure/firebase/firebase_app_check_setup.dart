/// Ativação do App Check antes dos serviços Firebase de cada aplicativo nomeado.
///
/// O seletor de contas usa vários FirebaseApp do mesmo projeto. Cada instância
/// precisa de um provedor antes de acessar FirebaseAuth; ativar só o aplicativo
/// padrão não prepara os contêineres das outras contas. A composição compartilha
/// esta inicialização com o bootstrap, sem levar SDKs à camada de aplicação.
///
/// Builds debug usam o provedor oficial de depuração; seu token precisa ser
/// cadastrado no Console por quem executa o app. Builds profile/release usam
/// Play Integrity no Android e App Attest com fallback Device Check em Apple.
/// Nenhum token de depuração é embutido no código. O SDK possui cache e renovação.
/// A fiscalização dos produtos Firebase continua sendo configurada no Console.
/// FIREBASE_APP_CHECK_DEBUG_TOKEN permite reutilizar um segredo local entre
/// os contêineres de contas no modo debug, evitando novos cadastros por instância.
library;

import 'package:firebase_app_check/firebase_app_check.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:flutter/foundation.dart';

final _initializations = <String, Future<void>>{};

const _debugToken = String.fromEnvironment('FIREBASE_APP_CHECK_DEBUG_TOKEN');

/// Prepara uma instância uma única vez e permite repetir uma ativação que falhou.
/// Deve ser aguardado após initializeApp e antes de construir FirebaseAuth.
Future<void> initializeFirebaseAppCheck(FirebaseApp app) => _initializations
    .putIfAbsent(app.name, () => Future<void>(() => _activate(app)));

Future<void> _activate(FirebaseApp app) async {
  try {
    if (kIsWeb ||
        !{
          TargetPlatform.android,
          TargetPlatform.iOS,
          TargetPlatform.macOS,
        }.contains(defaultTargetPlatform)) {
      throw UnsupportedError(
        'App Check ainda não está configurado para esta plataforma.',
      );
    }

    final appCheck = FirebaseAppCheck.instanceFor(app: app);

    await appCheck.activate(
      providerAndroid: kDebugMode
          ? AndroidDebugProvider(
              debugToken: _debugToken.isEmpty ? null : _debugToken,
            )
          : const AndroidPlayIntegrityProvider(),
      providerApple: kDebugMode
          ? AppleDebugProvider(
              debugToken: _debugToken.isEmpty ? null : _debugToken,
            )
          : const AppleAppAttestWithDeviceCheckFallbackProvider(),
    );

    await appCheck.setTokenAutoRefreshEnabled(true);
  } catch (_) {
    _initializations.remove(app.name);
    rethrow;
  }
}
