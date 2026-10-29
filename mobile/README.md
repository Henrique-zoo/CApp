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
  cada módulo, sem depender de Flutter, SDKs ou transporte.
- **infrastructure** de cada módulo implementa suas portas e traduz dados externos
  para o vocabulário da aplicação.
- **presentation** de cada módulo apresenta estados e encaminha intenções do
  usuário à aplicação.
- **presentation** na raiz de src oferece componentes visuais compartilhados
  por meio de uma fachada independente dos módulos.
- **infrastructure** na raiz de src contém mecanismos compartilhados, como
  transporte HTTP e contratos de rede.

Uma dependência externa deve atravessar um adaptador antes de chegar aos modelos
da aplicação. Uma interação visual deve atravessar a API da aplicação antes de
produzir efeitos externos.

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
| application/models/ | Representar fatos e valores usados pelos casos de uso. | Dart e tipos da própria aplicação. |
| application/ports/ | Definir operações externas necessárias ao módulo. | Modelos, resultados e falhas da aplicação. |
| Demais bibliotecas de application/ | Coordenar casos de uso, concorrência, estado e políticas locais. | Dart e contratos da própria aplicação. |
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

Os modelos de aplicação não possuem métodos JSON ou referências a SDKs.
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
- [FirebaseMicrosoftAuth](lib/src/modules/identity/infrastructure/firebase/firebase_microsoft_auth.dart):
  observação do SDK, interação Microsoft e credenciais.
- [Gateway institucional](lib/src/modules/identity/infrastructure/backend/backend_institutional_identity_gateway.dart):
  tradução entre protocolo e contrato de aplicação.
- [SessionCacheCodec](lib/src/modules/identity/infrastructure/preferences/session_cache_codec.dart):
  formato persistente e compatibilidade.
- [SessionGate](lib/src/app/session_gate.dart):
  seleção visual e propriedade das subárvores.

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
│   │   └── session_gate_test.dart
│   ├── infrastructure/backend/
│   │   ├── auth/
│   │   │   └── firebase_bearer_interceptor_test.dart
│   │   └── identity/
│   │       └── identity_api_test.dart
│   └── modules/identity/
│       ├── application/
│       │   ├── institutional_auth_service_test.dart
│       │   └── session_controller_test.dart
│       ├── infrastructure/
│       │   ├── backend/
│       │   │   └── backend_institutional_identity_gateway_test.dart
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

[architecture_boundaries_test.dart](test/architecture/architecture_boundaries_test.dart)
verifica as diretivas de importação e exportação do módulo atual: aplicação
independente, apresentação sem adaptadores, transporte sem dependência dos módulos
e fachadas com exportações de suas respectivas camadas. Novos módulos devem
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

## Documentação e validação

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
