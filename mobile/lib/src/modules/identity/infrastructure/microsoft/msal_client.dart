/// Criação do cliente nativo Microsoft para autorização do Graph.
///
/// O MSAL precisa de configuração de plataforma antes de atender solicitações
/// de credenciais. Esta biblioteca concentra essa inicialização para que telas
/// e casos de uso recebam um adaptador pronto, sem conhecer assets ou redirect URIs.
///
/// AppDependencies chama createMsalClient depois do bootstrap Flutter/Firebase e
/// entrega o resultado a MsalMicrosoftTokenProvider. A configuração pública vem
/// da composição; o asset define o modo de múltiplas contas e a autorização pelo
/// navegador. A integração atual configura Android.
///
/// O SDK administra o cache de credenciais. Criar o cliente não autentica no
/// Firebase, verifica a instituição ou abre autorização Microsoft. Falhas de
/// configuração ou inicialização propagam antes da montagem da interface.
library;

import 'package:msal_auth/msal_auth.dart';

/// Inicializa o MSAL Android sem abrir a interface de autenticação.
///
/// Deve ser chamado depois da inicialização do binding Flutter.
/// A composição deve reutilizar a instância retornada.
///
/// [clientId] é o Application (client) ID do registro Microsoft.
/// [androidRedirectUri] deve corresponder à plataforma Android
/// configurada no portal e ao filtro do AndroidManifest.xml.
///
/// Falhas de inicialização do SDK são propagadas.
Future<MultipleAccountPca> createMsalClient({
  required String clientId,
  required String androidRedirectUri,
}) {
  if (clientId.trim().isEmpty) {
    throw ArgumentError('O clientId Microsoft deve ser informado.');
  }

  if (androidRedirectUri.trim().isEmpty) {
    throw ArgumentError('A redirect URI Android deve ser informada.');
  }

  return MultipleAccountPca.create(
    clientId: clientId,
    androidConfig: AndroidConfig(
      configFilePath: 'assets/auth/msal_config.json',
      redirectUri: androidRedirectUri,
    ),
  );
}
