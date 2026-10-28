/// Ponto de entrada e transferência da aplicação para a árvore Flutter.
///
/// Esta biblioteca conecta a inicialização da plataforma à composição do CApp.
/// Concentrar essa sequência em um único ponto evita que telas inicializem plugins
/// ou construam adaptadores enquanto são renderizadas.
///
/// ## Ordem de inicialização
///
/// 1. Prepara o binding Flutter, necessário aos plugins de plataforma.
/// 2. Aguarda o bootstrap dos serviços externos.
/// 3. Cria AppDependencies com a URL do backend e inicia o controlador de sessão.
/// 4. Entrega a composição já iniciada a AppRoot por meio de runApp.
///
/// Iniciar o acompanhamento não significa concluir a restauração. A árvore visual
/// é montada enquanto o controlador aguarda a autenticação e, se aplicável, o cache;
/// os estados publicados determinam qual interface pode ser apresentada.
///
/// ## Configuração e propriedade
///
/// BACKEND_BASE_URL é recebido por --dart-define. O padrão
/// http://10.0.2.2:3000 atende ao emulador Android; outros ambientes precisam
/// informar um endereço alcançável pelo dispositivo.
///
/// AppRoot assume o encerramento dos recursos da composição. Falhas anteriores
/// à montagem propagam ao chamador de main. Não há consulta preventiva a health
/// ou ready: cada operação externa informa sua própria falha.
library;

import 'package:capp/src/src.dart';
import 'package:flutter/widgets.dart';

import 'bootstrap.dart';

/// Inicializa os serviços e transfere a composição para [AppRoot].
///
/// Prepara o binding Flutter, aguarda a inicialização do Firebase, cria
/// [AppDependencies] e inicia o acompanhamento da sessão antes de montar
/// a interface.
///
/// O retorno de `start()` não significa que a restauração inicial terminou.
/// A interface deve observar o controlador para acompanhar seus resultados.
///
/// Falhas de inicialização propagam antes da montagem da interface.
/// Após a montagem, [AppRoot] assume a solicitação de encerramento das
/// dependências quando for descartado ou receber outra composição.
///
/// O Future retornado representa apenas a inicialização. Sua conclusão
/// não corresponde ao fechamento do aplicativo nem ao descarte dos recursos.
Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await initializeServices();

  final dependencies = AppDependencies.create(
    backendBaseUrl: const String.fromEnvironment(
      'BACKEND_BASE_URL',
      defaultValue: 'http://10.0.2.2:3000',
    ),
  )..start();

  // Inicializa a interface Flutter.
  runApp(AppRoot(dependencies: dependencies));
}
