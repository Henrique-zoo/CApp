# CApp Mobile

Cliente Flutter do CApp. A arquitetura organiza o código por módulos do domínio,
com contratos de aplicação independentes da interface e das integrações externas.
Este documento descreve a estrutura geral a ser repetida nos novos módulos.

Os fluxos concretos, suas motivações e seus detalhes operacionais ficam nos
Dartdocs das bibliotecas responsáveis. O módulo de identidade é a implementação
atual desse padrão; sua sessão não define a organização dos demais domínios.

## Visão geral

A estrutura combina módulos verticais e uma pequena infraestrutura compartilhada:

- **app** integra os módulos, compõe dependências e possui a interface global.
- **application** contém modelos, contratos e coordenação dos casos de uso de
  cada módulo. Apresentação, transporte e persistência permanecem nas camadas
  responsáveis por esses detalhes.
- **infrastructure** de cada módulo implementa suas portas e traduz dados externos
  para o vocabulário da aplicação.
- **presentation** de cada módulo apresenta estados e encaminha intenções do
  usuário à aplicação.
- **presentation** na raiz de src oferece componentes visuais compartilhados
  por meio de uma fachada independente dos módulos.
- **infrastructure** na raiz de src contém mecanismos compartilhados, como
  transporte HTTP e contratos de rede.

Integrações com serviços, transporte e persistência atravessam adaptadores que
preservam os contratos da aplicação. Bibliotecas auxiliares podem ser usadas
diretamente quando seu uso pertence à responsabilidade da camada. Uma interação
visual atravessa a API da aplicação antes de produzir efeitos externos.

## Estrutura de diretórios

~~~text
lib/
├── main.dart                         # Ponto de entrada e ordem de inicialização.
├── bootstrap.dart                    # Preparação dos SDKs de plataforma.
├── firebase_options.dart             # Configuração gerada pelo FlutterFire.
└── src/
    ├── src.dart                      # API de montagem usada pelo ponto de entrada.
    ├── app/
    │   ├── app.dart                  # Fachada dos componentes de composição.
    │   ├── app_dependencies.dart      # Construção e propriedade das dependências.
    │   ├── app_root.dart              # Propriedade da composição na árvore Flutter.
    │   ├── capp.dart                  # Tema e composição visual global.
    │   └── ...                       # Integração de navegação e ciclo de vida.
    ├── presentation/
    │   ├── presentation.dart          # API dos componentes visuais compartilhados.
    │   └── widgets/                   # Avisos e demais componentes genéricos.
    ├── infrastructure/
    │   └── backend/
    │       ├── backend_api.dart       # Transporte compartilhado e suas APIs.
    │       ├── auth/                  # Credenciais das requisições HTTP.
    │       ├── converters/            # Conversões JSON compartilhadas pelos DTOs.
    │       ├── error/                 # Envelope público de erros do protocolo.
    │       ├── identity/              # Cliente e DTOs HTTP de identidade.
    │       └── monitoring/            # Endpoints técnicos de diagnóstico.
    └── modules/
        └── identity/
            ├── identity.dart         # API pública de aplicação do módulo.
            ├── identity_widgets.dart # API pública dos componentes Flutter.
            ├── application/
            │   ├── models/           # Tipos próprios usados pelos casos de uso.
            │   ├── ports/            # Contratos requeridos pela aplicação.
            │   ├── errors/           # Falhas expressas no vocabulário do módulo.
            │   └── ...               # Serviços, controladores, estados e resultados.
            ├── infrastructure/
            │   ├── backend/          # Implementação das portas sobre clientes HTTP.
            │   ├── firebase/         # Adaptação do SDK às portas do módulo.
            │   ├── microsoft/        # Inicialização MSAL e credenciais Graph.
            │   └── preferences/      # Persistência e codec do formato local.
            └── presentation/         # Telas, componentes e mensagens do módulo.
~~~

Cada novo domínio ocupa seu próprio diretório em modules/. Seus adaptadores
variam conforme as integrações necessárias; firebase/ e preferences/ são escolhas
do módulo atual, não diretórios obrigatórios para todos.

## Papéis das camadas

| Área | Responsabilidade | Dependências esperadas |
| --- | --- | --- |
| main.dart e bootstrap.dart | Preparar a plataforma e iniciar a aplicação na ordem correta. | Flutter, configuração dos SDKs e API de montagem. |
| app/ | Integrar módulos, tema, navegação, ciclo de vida e propriedade dos recursos. | APIs dos módulos; implementações concretas no ponto de composição. |
| application/models/ | Representar fatos e valores usados pelos casos de uso. | Tipos da aplicação e bibliotecas compatíveis com a responsabilidade dos modelos. |
| application/ports/ | Definir operações externas necessárias ao módulo. | Modelos, resultados e falhas da aplicação. |
| Demais bibliotecas de application/ | Coordenar casos de uso, concorrência, estado e políticas locais. | Contratos da aplicação e bibliotecas compatíveis com essa coordenação. |
| infrastructure/ do módulo | Adaptar serviços, SDKs e persistência às portas da aplicação. | Aplicação do módulo, bibliotecas externas e infraestrutura compartilhada. |
| presentation/ do módulo | Renderizar estados e encaminhar ações do usuário. | Flutter, aplicação do próprio módulo e API visual compartilhada. |
| src/presentation/ | Disponibilizar componentes visuais genéricos que recebem dados e callbacks. | Flutter e bibliotecas da própria apresentação compartilhada. |
| src/infrastructure/ | Oferecer mecanismos e contratos técnicos compartilhados. | Bibliotecas externas e outras bibliotecas da infraestrutura compartilhada. |

Os modelos atuais ficam em application/models/. Não há uma camada domain/
separada na implementação. Regras puras podem permanecer junto aos modelos e aos
casos de uso enquanto essa separação adicional não for necessária.

Um **serviço de aplicação** coordena uma operação usando portas e modelos.
Um **controlador** acrescenta coordenação com estado observável, concorrência
ou ciclo de vida quando o fluxo exige essas responsabilidades. Não é necessário
criar ambos para toda operação.

Uma **porta** é uma interface definida segundo a necessidade da aplicação.
Um **adaptador** implementa essa interface usando uma tecnologia concreta.
Um **mapper** converte representações entre fronteiras sem fazer I/O.
Um **codec** define como modelos são representados em um formato externo,
como o armazenamento local.

Componentes como gates pertencem à composição visual quando escolhem entre
áreas da aplicação. Eles apresentam uma decisão publicada pela aplicação;
a política que produz essa decisão permanece no controlador ou caso de uso.

Widgets de ligação em app/ também podem observar coordenações globais e traduzir
seus estados em dados visuais e callbacks. A aparência reutilizável pertence a
src/presentation/ e é consumida pela sua fachada. Esses componentes compartilhados
não importam app/, controladores dos módulos ou clientes HTTP. Ser global não é,
por si só, motivo para colocar a implementação visual na raiz de composição.

Coordenações técnicas globais podem receber operações como callbacks quando isso
expressar todo o contrato necessário. Não precisam ocupar um módulo de domínio.
SDKs de serviços externos e clientes de infraestrutura continuam conectados em
AppDependencies; os demais componentes de app/ usam as APIs públicas dos módulos
e da apresentação, além do framework e das bibliotecas de navegação e estado.

## Dependências e APIs dos módulos

As setas abaixo representam dependências de código, não a ordem de execução:

~~~text
presentation ──────────────> application
presentation dos módulos ──> API da apresentação compartilhada
infrastructure do módulo ──> application
infrastructure do módulo ──> infraestrutura compartilhada e SDKs

app / composição ──────────> APIs dos módulos e implementações concretas
~~~

A aplicação define as portas, e a infraestrutura as implementa. Por isso, um
caso de uso pode chamar um serviço externo em execução sem importar seu SDK:
a composição injeta uma implementação da interface requerida.

### Bibliotecas e responsabilidades

A escolha de uma biblioteca considera seu uso e a direção das dependências entre
camadas. Sua origem externa ao projeto não exige, por si só, um adaptador, uma
implementação própria ou uma exceção arquitetural. Essa orientação vale para
todas as camadas.

Por exemplo, os modelos de identidade usam `UuidValue`, do pacote `uuid`, como
tipo de valor. A interpretação do JSON HTTP, a representação persistente e as
chamadas a serviços continuam nos adaptadores correspondentes. A biblioteca de
UUID não transfere essas responsabilidades para os modelos.

As verificações automáticas fiscalizam as relações entre bibliotecas do projeto.
A adequação do uso de bibliotecas externas às responsabilidades de cada camada
também faz parte da revisão do código; ela não é determinada por uma lista de
pacotes permitidos.

### Fachadas e importações

Cada módulo oferece uma fachada de aplicação com seu próprio nome, como
identity.dart. Uma fachada separada, como identity_widgets.dart, oferece os
componentes Flutter. As exportações são explícitas e limitadas aos contratos
que os consumidores precisam conhecer.

A fachada de aplicação expõe modelos, resultados, falhas e operações consumíveis.
Serviços internos, portas de montagem e adaptadores não precisam ser reexportados.
A fachada visual expõe os widgets necessários à composição das telas.

Importações internas ao mesmo módulo podem usar caminhos relativos.
O limite arquitetural é o contrato atravessado: consumidores externos usam as
fachadas, enquanto o ponto de composição importa implementações explicitamente.
O nome src indica implementação interna por convenção; não impede tecnicamente
uma importação indevida.

### Colaboração entre módulos

Um módulo não acessa diretamente adaptadores, widgets ou arquivos internos
de outro. Quando precisar colaborar, deve usar uma API pública bem definida.
Se a colaboração envolver I/O, uma porta local pode expressar a necessidade
e um adaptador pode integrar a API do outro módulo.

As regras de importação atuais mantêm application independente de outros
módulos. A integração entre eles deve preservar esse limite por contratos
injetados, sem transformar application em um consumidor dos detalhes externos.

Dados pessoais, regras de negócio e casos de uso não passam a ser infraestrutura
global apenas porque dois módulos os utilizam. A responsabilidade deve continuar
atribuída ao módulo que possui aquele conceito.

### Transporte compartilhado e adaptadores do domínio

src/infrastructure/backend/ representa o protocolo: conexão, timeouts,
autenticação HTTP, clientes de endpoints e DTOs. Seus subdiretórios podem agrupar
APIs por domínio, mas não importam os módulos da aplicação.

`modules/<modulo>/infrastructure/backend/` representa o uso desse protocolo por
um módulo: implementa portas, converte DTOs e traduz códigos de erro para as falhas
da aplicação. Essa separação permite compartilhar o transporte sem centralizar
as regras de todos os domínios em um cliente HTTP global.

## Fluxo geral de uma operação

Uma operação iniciada na interface normalmente percorre estes passos:

1. O widget apresenta o estado recebido e encaminha a intenção do usuário.
2. O controlador ou serviço de aplicação valida as condições locais e coordena
   o caso de uso.
3. A aplicação chama uma porta para o efeito externo necessário.
4. O adaptador executa a integração e converte os dados para modelos próprios.
5. A aplicação interpreta o resultado e, quando houver estado, publica uma nova
   fotografia.
6. A apresentação renderiza essa fotografia ou o resultado recebido.

Este percurso descreve responsabilidades; não exige uma classe por etapa.
Leituras puras e operações sem estado podem ter uma organização mais simples.
A construção de widgets não deve iniciar implicitamente os efeitos do caso de uso.

## Modelos, protocolo e persistência

| Representação | Proprietário | Finalidade |
| --- | --- | --- |
| Modelo de aplicação | application/ do módulo | Expressar os dados e invariantes consumidos pelos casos de uso. |
| DTO HTTP | src/infrastructure/backend/ | Representar nomes de campos, variantes e formatos do contrato de rede. |
| Formato persistente | Adaptador e codec locais do módulo | Conservar dados conforme um contrato de armazenamento próprio. |
| Estado de apresentação | Coordenação da aplicação e apresentação | Informar atividade, resultado e falhas para renderização. |

Os modelos de aplicação expressam os dados usados pelos casos de uso.
A serialização do protocolo HTTP e do cache pertence à infraestrutura.
Os adaptadores conhecem as duas representações que traduzem; suas dependências
não precisam ser propagadas para os consumidores.

O formato do cache possui evolução própria. A semelhança com um JSON HTTP não
é motivo suficiente para reutilizar o DTO como registro persistente.
A escolha entre preferências, arquivos ou banco local pertence ao adaptador
e depende das necessidades do módulo.

Uma cópia local pode orientar a experiência de uso, mas não substitui a
autenticação nem a autorização executadas pelo servidor.

## Estado, falhas e ciclo de vida

A coordenação possui o estado e publica resultados observáveis. A interface
consome esses contratos e evita duplicar decisões com relógios, consultas ao SDK
ou interpretação de status HTTP dentro de widgets.

Falhas operacionais devem permanecer explícitas. Ausência legítima de dado pode
ser representada por null; indisponibilidade, recusa ou conteúdo incompatível
exigem exceção ou resultado que preserve sua classificação e diagnóstico.
Categorias públicas podem orientar mensagens sem expor causas técnicas.

Construir uma dependência, iniciar seu acompanhamento e encerrá-la são etapas
distintas. A composição define quais instâncias são compartilhadas, quem pode
iniciá-las e quem possui seu descarte. Telas recebem dependências prontas,
em vez de criar seus próprios clientes ou localizar serviços globais.

Operações assíncronas devem conferir se seu resultado ainda pertence ao contexto
ativo antes de publicá-lo. Encerrar um observador ou descartar um widget não
garante cancelar efeitos externos já iniciados. Dados necessários à restauração
devem ser persistidos durante a operação correspondente, sem depender de um
callback garantido de fechamento do processo.

## Como acrescentar um módulo

Use esta estrutura como referência, criando somente os diretórios necessários:

~~~text
modules/<modulo>/
├── <modulo>.dart
├── <modulo>_widgets.dart
├── application/
│   ├── models/
│   ├── ports/
│   ├── errors/
│   └── ...                         # Casos de uso, estado e resultados.
├── infrastructure/
│   └── <integracao>/              # Adaptadores e conversões da fronteira.
└── presentation/
    └── ...                         # Telas e componentes do módulo.
~~~

1. Defina a responsabilidade do domínio e sua API de consumo.
2. Modele os dados, resultados e falhas sem acoplar a aplicação a um SDK.
3. Declare portas para efeitos externos realmente necessários.
4. Implemente os adaptadores e suas conversões. Endpoints novos acrescentam
   clientes e DTOs ao transporte compartilhado quando isso for aplicável.
5. Construa a apresentação sobre os contratos da aplicação.
6. Exporte apenas a superfície necessária nas fachadas do módulo.
7. Conecte as implementações no ponto de composição e defina sua propriedade,
   início e encerramento.
8. Acrescente testes nos caminhos correspondentes e documente motivação,
   responsabilidades, fluxo, falhas e limites nas bibliotecas.

Fachadas visuais e controladores só são necessários quando houver consumidores
ou comportamento que os justifique. Evite diretórios vazios e exportações amplas
de todas as bibliotecas internas.

## Onde consultar os fluxos concretos

O README mantém o mapa arquitetural. As decisões específicas de identidade
e sessão estão nos Dartdocs, principalmente:

- [SessionController](lib/src/modules/identity/application/session_controller.dart):
  restauração, entrada, prazo, concorrência, persistência e encerramento.
- [InstitutionalAuthService](lib/src/modules/identity/application/institutional_auth_service.dart):
  associação da operação à identidade autenticada.
- [FirebaseAccountAuth](lib/src/modules/identity/infrastructure/firebase/firebase_account_auth.dart):
  seleção de contas, credenciais internas e políticas de saída.
- [FirebaseMicrosoftAuth](lib/src/modules/identity/infrastructure/firebase/firebase_microsoft_auth.dart):
  observação Firebase e integração entre login e credenciais Microsoft.
- [MicrosoftTokenProvider](lib/src/modules/identity/application/ports/microsoft_token_provider.dart):
  contrato de obtenção silenciosa, autorização interativa e limpeza da conta.
- [MsalMicrosoftTokenProvider](lib/src/modules/identity/infrastructure/microsoft/msal_microsoft_token_provider.dart):
  cache de credenciais pelo SDK e tradução de falhas MSAL.
- [Gateway institucional](lib/src/modules/identity/infrastructure/backend/backend_institutional_identity_gateway.dart):
  tradução entre protocolo e contrato de aplicação.
- [SessionCacheCodec](lib/src/modules/identity/infrastructure/preferences/session_cache_codec.dart):
  formato persistente e compatibilidade.
- [SessionGate](lib/src/app/session_gate.dart):
  seleção visual e propriedade das subárvores.

## Conta interna e contas salvas

O primeiro acesso Microsoft provisiona uma conta `Incomplete`. Para concluir o
cadastro, o usuário informa apelido, período atual do curso e semestre de ingresso
na instituição, além de configurar e-mail e senha. O e-mail institucional é uma
sugestão; outro endereço pode ser escolhido. A credencial é vinculada ao mesmo
UID Firebase, e o backend ativa a conta após validar o perfil e a autenticação
com senha. A verificação institucional devolve usuário e evidência juntos,
dispensando outra consulta de restauração após a verificação.

Dados não nulos do Microsoft Graph são preservados. Ausências podem receber
complementos declarados pelo usuário, armazenados no perfil CApp sem substituir
a evidência institucional.

A entrada permite selecionar uma conta salva, autenticar com e-mail/senha ou
iniciar o acesso Microsoft. Cada conta salva usa um FirebaseApp nomeado do mesmo
projeto, com persistência de credenciais administrada pelo SDK. O índice local
guarda referências e rótulos; senhas e tokens não são copiados para preferências.
Selecionar uma conta tenta reutilizar sua sessão, respeitando a política de saída
e a validade institucional. Sair não seleciona outra conta automaticamente.

O logout oferece três políticas:

- **Manter entrada rápida:** conserva a sessão local para uma futura seleção.
- **Exigir login:** encerra as credenciais locais e mantém a conta lembrada,
  exigindo senha na próxima seleção. É a opção padrão para contas com senha.
- **Esquecer conta:** encerra as credenciais e remove a referência e os caches de
  perfil e verificação daquela conta, preservando as demais contas e o cadastro
  remoto.

O retorno à entrada aguarda o término da tentativa de limpeza. Falhas são
informadas; a remoção dos caches pode ser repetida sem novo login.

A confirmação institucional vale **120 dias**, independentemente da sessão
Firebase. Após autenticar, inclusive pela opção Microsoft, o aplicativo consulta
`GET /auth/session`. Uma confirmação vigente é reutilizada sem solicitar token
MSAL, consultar o Graph ou estender o prazo. Cadastro incompleto retoma o formulário.
Conta ainda não provisionada (`404 account_not_provisioned`) ou confirmação ausente
exige verificação institucional; falhas temporárias repetem a restauração, sem
serem interpretadas como primeiro acesso. Quando é necessário sincronizar, o MSAL tenta obter autorização
silenciosamente e solicita interação quando exigida. Falhas temporárias seguem
as tentativas controladas do controlador. O backend continua responsável pela
validade da evidência e pela autorização das operações.

## Organização dos testes

A árvore de testes espelha os caminhos da implementação: quando um arquivo
`lib/<caminho>/<nome>.dart` possui uma suíte própria, ela fica em
`test/<caminho>/<nome>_test.dart`.
Isso inclui src/ e os subdiretórios dos adaptadores. Não é necessário criar
arquivos vazios para bibliotecas que ainda não possuem uma suíte própria.

~~~text
test/
├── src/
│   ├── app/
│   │   ├── capp_test.dart
│   │   ├── session_gate_test.dart
│   │   ├── backend_readiness_controller_test.dart
│   │   └── backend_readiness_notice_test.dart
│   ├── infrastructure/backend/
│   │   ├── auth/
│   │   │   └── firebase_bearer_interceptor_test.dart
│   │   ├── identity/
│   │   │   └── identity_api_test.dart
│   │   └── monitoring/
│   │       └── monitoring_api_test.dart
│   ├── presentation/widgets/
│   │   └── status_banner_test.dart
│   └── modules/identity/
│       ├── application/
│       │   ├── models/
│       │   │   └── institutional_verification_test.dart
│       │   ├── institutional_auth_service_test.dart
│       │   └── session_controller_test.dart
│       ├── infrastructure/
│       │   ├── backend/
│       │   │   └── backend_institutional_identity_gateway_test.dart
│       │   ├── firebase/
│       │   │   └── firebase_microsoft_auth_test.dart
│       │   ├── microsoft/
│       │   │   └── msal_microsoft_token_provider_test.dart
│       │   └── preferences/
│       │       ├── preferences_session_store_test.dart
│       │       └── session_cache_codec_test.dart
│       └── presentation/
│           └── institutional_login_page_test.dart
├── architecture/
│   └── architecture_boundaries_test.dart
├── bdd/
│   └── gherkin_pending_tags_test.dart
└── support/
    ├── identity_fakes.dart
    ├── http_fakes.dart
    └── preferences_fake.dart
~~~

A tradução de erros do backend pertence ao gateway do módulo de identidade.
Sua suíte acompanha infrastructure/backend/ dentro desse módulo. Os testes em
src/infrastructure/backend/ verificam o transporte compartilhado e o interceptor
de credenciais, sem classificar regras de sessão.

src/app/capp_test.dart concentra os cenários que atravessam a composição visual.
As seleções de ramo e a navegação pertencem à suíte do gate; as mensagens e
interações de entrada ficam na apresentação. Os testes do codec não utilizam
preferências, e os do armazenamento exercitam o adaptador sobre preferências
controladas.

As verificações transversais de arquitetura ficam em architecture/; as do runner
Gherkin, em bdd/. Essas suítes não correspondem a um arquivo de lib/. Os fakes
compartilhados ficam em support/ e não fazem parte da implementação distribuída.

A renovação automática integra a suíte de SessionController. Ela verifica
temporização, espera progressiva após falhas temporárias, recuperação sem cache,
necessidade de interação, preservação do prazo após falha e isolamento entre
contas. A apresentação distingue confirmação Microsoft de repetição de conexão.
As suítes de FirebaseMicrosoftAuth e do provedor MSAL
controlam a fronteira dos SDKs sem executar autenticação nativa.

[architecture_boundaries_test.dart](test/architecture/architecture_boundaries_test.dart)
verifica as diretivas de importação e exportação do módulo atual: aplicação
sem dependência das outras camadas, apresentação sem adaptadores, transporte sem
dependência dos módulos ou de app, composição visual por fachadas e exportações
das respectivas camadas.
O teste verifica as dependências internas sem manter exceções por pacote externo.
As importações de SDKs externos e clientes de infraestrutura em app ficam na raiz
de composição AppDependencies. Novos módulos devem
receber verificações equivalentes. Os diretórios, por si só, não impõem esses
limites, e essas verificações não substituem a autorização das operações.

integration_test/ conserva app_test.dart, steps/authentication/, steps/shared/
e support/. É a árvore do runner de cenários compartilhados, não das verificações
unitárias e de widgets.

A execução padrão e a CI descobrem os arquivos de teste recursivamente.
É possível selecionar uma área, a partir de mobile:

~~~bash
flutter test --no-pub test/src/app
flutter test --no-pub test/src/infrastructure/backend
flutter test --no-pub test/src/modules/identity/application
~~~

## Configuração e execução Android

No Firebase Console, habilite os provedores **Microsoft** e **E-mail/senha** e
configure a recuperação de senha. A política de senha do Firebase determina sua
força; as credenciais são associadas pelo fluxo de
[account linking](https://firebase.google.com/docs/auth/flutter/account-linking).

A composição atual mantém autenticação Firebase e usa MSAL para obter
credenciais Microsoft. O bootstrap Firebase precisa estar configurado; o registro
Microsoft também precisa contemplar o aplicativo Android e sua assinatura.
O filtro de `BrowserTabActivity` em `android/app/src/main/AndroidManifest.xml`
deve corresponder à redirect URI do registro. A configuração de múltiplas contas
fica em `assets/auth/msal_config.json`, declarado no `pubspec.yaml`.

A partir de `mobile`, forneça os valores públicos do seu registro Microsoft:

~~~bash
flutter run \
  --dart-define=MSAL_CLIENT_ID='<application-client-id>' \
  --dart-define=MSAL_ANDROID_REDIRECT_URI='<redirect-uri-do-registro-android>' \
  --dart-define=BACKEND_BASE_URL=http://10.0.2.2:3000
~~~

Use a redirect URI no formato fornecido pelo registro; o `android:path` do
manifesto contém o hash da assinatura sem a codificação percentual da URI.
Não há client secret no aplicativo. Os dois parâmetros MSAL são obrigatórios;
valores vazios interrompem a inicialização. O endereço de backend mostrado
atende ao emulador Android; dispositivos físicos precisam de um endereço
alcançável. A composição MSAL atual ainda não configura as outras plataformas.

## Documentação e validação

O README concentra arquitetura, configuração, execução e um resumo dos fluxos.
Detalhes operacionais e decisões dos componentes ficam nos Dartdocs das
bibliotecas responsáveis.

Cada biblioteca autoral descreve em Dartdoc sua motivação, posição na arquitetura,
responsabilidades, percurso dos dados e limites. Tipos e operações detalham
contratos, parâmetros, resultados, falhas e efeitos conforme necessário.
Arquivos gerados mantêm a documentação de seus geradores.

A partir de mobile:

~~~bash
flutter analyze --no-pub
flutter test --no-pub
dart doc
~~~

Após alterar DTOs anotados com `JsonSerializable` ou `CopyWith`, atualize os
arquivos gerados a partir de `mobile`:

~~~bash
dart run build_runner build
~~~

Use `dart run build_runner watch` para regenerá-los durante as edições.
Os arquivos `.g.dart` são produzidos a partir das classes Dart; uma alteração
isolada no OpenAPI não atualiza esses arquivos automaticamente.
O `build.yaml` configura o gerador para ignorar `unnecessary_cast` somente nas
saídas geradas, pois `copy_with_extension_gen` pode emitir casts redundantes
para campos `Object?`. O código autoral continua sujeito a essa verificação.

Os testes de sessão utilizam contratos simulados; os testes HTTP conservam Dio e
a desserialização com transporte em memória. Os testes de widget verificam os
ramos de entrada, erro, prontidão, aviso de cache e retomada. Eles não executam
login Microsoft real nem comprovam configuração externa do Firebase ou backend.

A geração padrão de Dartdoc cobre bibliotecas públicas fora de lib/src.
As APIs internas também ficam disponíveis nos comentários e na navegação da IDE.
Exemplos em comentários não são executados automaticamente.

No Windows, Dartdoc 9.0.6 pode falhar com CRLF ao processar doc imports.
A correção consta no [changelog do Dartdoc 9.0.8](https://github.com/dart-lang/dartdoc/blob/main/CHANGELOG.md#908).
A validação pode utilizar uma instalação isolada dessa ferramenta em .dart_tool,
sem alterar as dependências do aplicativo.

## Testes Gherkin

Os cenários compartilhados estão em `../features/`. Antes da análise estática
ou dos testes, gere o arquivo usado pelo runner de integração:

```bash
dart run tool/generate_gherkin.dart
```

As steps de domínio atuais ainda são pendentes. A existência do runner e uma
suíte unitária aprovada não comprovam execução desses fluxos de ponta a ponta.

O runner seleciona cenários `@mobile` e registra aqueles marcados com
`@pending_mobile` como ignorados, sem executar suas steps. As tags da feature
e da regra são herdadas pelos cenários gerados. `@pending_backend` não impede
a execução no aplicativo. Uma suíte em que todos os cenários estão pendentes
continua registrando testes ignorados em vez de falhar por ausência de testes.

A CI gera os cenários antes de `flutter analyze` e verifica os filtros com
os testes de `test/`; executar os cenários de integração exige um dispositivo
ou emulador Flutter disponível.


## Firebase App Check

O bootstrap ativa App Check antes de Firebase Auth. No Android, builds debug usam
AndroidDebugProvider; profile/release usam Play Integrity. Cada FirebaseApp
nomeado do seletor também é ativado antes do FirebaseAuth correspondente. O SDK
possui o cache e a renovação; não copiamos JWTs App Check para preferências.
Requisições de identidade levam `X-Firebase-AppCheck` e o Bearer da conta ativa.
As sondagens `/health` e `/ready` continuam públicas. Falhas da comprovação são
classificadas separadamente de senha incorreta e preservam os caches da conta.

Para desenvolvimento com várias contas, use **um token de depuração compartilhado**
por todas as instâncias. Gere um UUID uma vez, cadastre-o no Console e defina
`FIREBASE_APP_CHECK_DEBUG_TOKEN` no arquivo local usado pelo Flutter, por exemplo
`mobile/.env` (ignorado pelo Git). Sem valor explícito, o SDK gera segredos por
instância, o que exigiria cadastros adicionais ao criar contêineres de contas.
Nunca use o segredo de depuração como JWT no cabeçalho HTTP.

1. No Firebase Console do projeto `capp-a331b`, abra **Security > App Check > Apps**.
2. Cadastre o Android `br.com.capp`, App ID
   `1:13241645392:android:f9927433685734e2a45455`, com Play Integrity e o SHA-256
   do certificado usado na distribuição. Configure a vinculação com o projeto
   Cloud e os veredictos conforme o canal de distribuição (Play ou instalação externa).
   Na distribuição pelo Google Play, use o certificado de assinatura do aplicativo
   indicado no Play Console. O certificado identifica a assinatura do APK, não o
   computador ou o celular. Builds debug usam o segredo do próximo passo, sem
   atestado Play Integrity; cadastrar um SHA-256 debug não substitui esse segredo.
3. Em **Manage debug tokens**, cadastre o UUID privado do seu ambiente. Configure
   o mesmo valor em `mobile/.env`, junto aos parâmetros Microsoft já existentes:

   ```dotenv
   FIREBASE_APP_CHECK_DEBUG_TOKEN=UUID_PRIVADO_CADASTRADO_NO_CONSOLE
   ```

4. Execute com `flutter run --dart-define-from-file=.env` a partir de `mobile/`.
   Se usar parâmetros individuais, acrescente
   `--dart-define=FIREBASE_APP_CHECK_DEBUG_TOKEN=UUID_PRIVADO` ao comando habitual.
   O parâmetro é usado somente no modo debug; a produção continua usando Play Integrity.
5. Habilite enforcement dos produtos Firebase somente depois de cadastrar os apps
   utilizados e verificar as métricas. O backend Rust já fiscaliza suas próprias
   rotas e não depende desse botão do Console.

Se a obtenção do atestado retornar `403` com `API_KEY_SERVICE_BLOCKED` ou
`Requests to this API firebaseappcheck.googleapis.com ... are blocked`, confira
a chave Android usada por `firebase_options.dart` e `google-services.json` em
**Google Cloud > APIs e serviços > Credenciais**. Nas **Restrições de API** dessa
chave, acrescente **Firebase App Check API** (`firebaseappcheck.googleapis.com`),
preservando as demais APIs permitidas. Se a API não estiver disponível na lista,
habilite-a em **APIs e serviços > Biblioteca**. Esse bloqueio ocorre antes da
validação do segredo de depuração e é independente do cadastro do certificado.
Consulte a [lista oficial de APIs necessárias para cada serviço Firebase](https://firebase.google.com/docs/projects/api-keys#required-apis-for-firebase-services).

O login Microsoft atual usa `FirebaseAuth.signInWithProvider`, que abre uma
etapa OAuth no navegador antes da autorização Graph pelo MSAL. A chave também
é usada nessa etapa web. Restringi-la a **Apps Android** pode bloquear a consulta
de configuração do Firebase Auth com `API_KEY_ANDROID_APP_BLOCKED` e
`androidPackage: <empty>`, mesmo quando pacote e SHA-1 estão corretos: essa
requisição não envia a identificação Android. Nesse fluxo, mantenha
**Restrições do aplicativo > Nenhum** para a chave usada pelo Firebase Auth e
preserve as **Restrições de API**, incluindo Identity Toolkit API, Token Service
API e Firebase App Check API, além das demais APIs Firebase utilizadas.
O SHA-1 continua cadastrado no aplicativo Android do Firebase; remover a
restrição de aplicativo da chave não desativa App Check ou sua fiscalização
no backend. A [limitação da etapa web é documentada pelos mantenedores do Firebase](https://github.com/firebase/firebase-android-sdk/issues/1757#issuecomment-655777739).

Para confirmar que o backend em execução já aplica App Check, faça uma chamada
sem credenciais a `GET /auth/session`. A resposta deve ser `403` com código
`app_check_required`. Se retornar `401 unauthorized`, a autenticação Firebase foi
atingida antes dessa proteção: confira se está acessando o endereço correto e
recrie o serviço com `docker compose --profile backend up -d --build backend`
a partir da raiz do repositório.

O `.env` da raiz do repositório configura o backend com o número público do
projeto e a lista de App IDs permitidos, conforme `.env.example`. O arquivo local
do mobile configura o build Flutter; não passe o `.env` do banco ao Flutter.
Recompile e reinicie o backend após atualizar suas variáveis, e faça uma nova
execução do Flutter: um plugin nativo novo não é carregado por hot reload.

A preparação inclui provedores Apple debug/App Attest com fallback Device Check,
mas o fluxo MSAL atual segue focado em Android; web e desktop precisam de uma
configuração própria. Consulte a [ativação oficial Flutter](https://firebase.google.com/docs/app-check/flutter/default-providers),
o [provedor debug](https://firebase.google.com/docs/app-check/flutter/debug-provider)
e a [configuração Play Integrity](https://firebase.google.com/docs/app-check/android/play-integrity-provider).
