/// Prepara o Firebase e o App Check antes de montar os casos de uso.
///
/// initializeServices garante a existência do FirebaseApp padrão e ativa seu
/// provedor de App Check antes de usar autenticação ou transporte. A composição
/// reutiliza a mesma ativação para aplicativos nomeados do seletor de contas.
///
/// A guarda procura explicitamente o aplicativo padrão: uma instância nomeada
/// já existente não substitui essa inicialização. Ativações bem-sucedidas do
/// App Check são reaproveitadas; falhas podem ser repetidas e propagam ao chamador.
/// O binding Flutter deve estar preparado. O sucesso não significa login nem
/// comprova que provedores e tokens de depuração foram cadastrados no Console.
library;

import 'package:firebase_core/firebase_core.dart';

import 'firebase_options.dart';
import 'src/infrastructure/firebase/firebase_app_check_setup.dart';

/// Inicializa o aplicativo padrão e aguarda a ativação do App Check.
/// Falhas do SDK, de configuração ou de plataforma propagam antes da interface.
Future<void> initializeServices() async {
  if (!Firebase.apps.any((app) => app.name == '[DEFAULT]')) {
    await Firebase.initializeApp(
      options: DefaultFirebaseOptions.currentPlatform,
    );
  }

  await initializeFirebaseAppCheck(Firebase.app());
}
