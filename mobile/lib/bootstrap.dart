/// Preparação dos serviços externos antes da composição da aplicação.
///
/// Esta biblioteca separa a configuração dos SDKs da construção dos casos de uso
/// e da interface. Assim, o ponto de entrada e os testes de integração podem
/// preparar o mesmo aplicativo Firebase sem montar widgets ou abrir login.
///
/// ## Participação na inicialização
///
/// O chamador prepara o binding Flutter e aguarda initializeServices.
/// Quando nenhum aplicativo Firebase está registrado, a função inicializa o padrão
/// com as opções da plataforma em firebase_options.dart. Somente depois dessa
/// etapa a composição pode obter FirebaseAuth.instance e conectar os adaptadores.
///
/// ## Alcance da guarda
///
/// A presença de qualquer aplicativo Firebase faz a função retornar. Essa guarda
/// evita a inicialização comum repetida, mas não verifica se o aplicativo padrão
/// existe ou corresponde à configuração esperada e não serializa chamadas
/// concorrentes.
///
/// Falhas do SDK e plataformas sem configuração propagam. A conclusão comprova
/// apenas a inicialização solicitada; autenticação, conectividade com o backend
/// e configuração operacional do provedor Microsoft são etapas posteriores.
library;

import 'package:firebase_core/firebase_core.dart';

import 'firebase_options.dart';

/// Inicializa o aplicativo Firebase padrão quando não há aplicativos registrados.
///
/// O chamador deve inicializar o binding Flutter antes de usar os plugins.
/// Utiliza as opções da plataforma atual e propaga falhas de inicialização.
///
/// ## Comportamento e erros
///
/// Só chama `Firebase.initializeApp` se `Firebase.apps` estiver vazio. Se houver
/// qualquer aplicativo registrado, retorna sem conferir se o aplicativo padrão
/// ou suas opções correspondem ao ambiente esperado. A guarda não serializa
/// chamadas concorrentes. Falhas de configuração/plataforma ou do SDK propagam.
/// O sucesso não confirma conectividade com o backend ou login Microsoft.
Future<void> initializeServices() async {
  if (Firebase.apps.isEmpty) {
    await Firebase.initializeApp(
      options: DefaultFirebaseOptions.currentPlatform,
    );
  }
}
