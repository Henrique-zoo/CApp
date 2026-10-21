# CApp Backend

Backend da plataforma **CApp**, responsável por fornecer a API utilizada pelo aplicativo móvel e por futuros clientes web.

O backend concentra as regras de negócio da plataforma, incluindo:

- autenticação e identidade acadêmica;
- gerenciamento de Centros Acadêmicos;
- contexto ativo de CA;
- perfis administrativos e permissões;
- notícias;
- eventos;
- demandas;
- materiais acadêmicos;
- eleições;
- FAQ;
- apadrinhamento;
- funcionalidades de governança.

O sistema é desenvolvido em **Rust** seguindo uma arquitetura de **Modular Monolith**, organizada por domínios de negócio e baseada na separação entre regras de negócio, casos de uso e detalhes de infraestrutura.

---

# Arquitetura

O backend é organizado em módulos de negócio:

```
src/

├── modules/
├── infrastructure/
├── api/
├── shared/
└── config/
```

Cada módulo possui suas próprias camadas:

```
module/

├── domain/
├── application/
├── infrastructure/
└── api/
```

As dependências entre camadas apontam para as regras de negócio e os contratos internos. No esquema abaixo, `A → B` significa que o código de `A` conhece e depende de `B`:

```
API → Application → Domain
Infrastructure → Application
Infrastructure → Domain
```

Os contratos de persistência e de acesso a serviços externos são definidos como traits nas camadas internas. Contratos necessários aos casos de uso ficam em `Application`; contratos que fazem parte do modelo de domínio podem ficar em `Domain`. A `Infrastructure` implementa esses contratos. O `Domain` permanece independente de `Application` e `Infrastructure`.

A direção das dependências é diferente do fluxo de chamadas durante a execução. Por exemplo, um caso de uso em `Application` pode chamar `UserRepository::save`, cuja implementação em `Infrastructure` executa SQL. O caso de uso conhece apenas a trait `UserRepository`; a implementação concreta conhece essa trait e os detalhes de acesso ao banco.

A inicialização da aplicação atua como ponto de composição: instancia os adaptadores de infraestrutura e os fornece aos casos de uso. Assim, os casos de uso recebem suas dependências sem precisar construir um repositório PostgreSQL ou um cliente de serviço externo.

---

# Princípios arquiteturais

## Modular Monolith

O backend é implementado como um único serviço dividido internamente em módulos independentes.

Essa abordagem permite:

- reduzir complexidade operacional;
- manter consistência entre funcionalidades;
- compartilhar regras comuns;
- facilitar evolução futura.

Caso o sistema cresça significativamente, módulos específicos podem ser extraídos para serviços independentes.

---

# Separação de responsabilidades

## Domain

Contém as regras de negócio do sistema.

Entidades e regras aplicam as validações e transições do negócio. A coordenação de consultas e persistência normalmente cabe aos casos de uso em `Application`.

As entidades atuais de identidade usam `sqlx::FromRow` e `sqlx::Type` para mapeamento direto. Esse é um acoplamento explícito e limitado aos metadados de persistência; consultas SQL, pools e transações ficam na infraestrutura. Portanto, a implementação não segue independência absoluta de bibliotecas de persistência.

As regras de domínio não executam consultas nem conhecem:

- conexões ou transações de banco de dados;
- HTTP;
- Firebase;
- serviços externos.

Exemplos:

- regras de publicação;
- permissões;
- estados eleitorais;
- validações de negócio.

---

## Application

Contém os casos de uso da aplicação.

Coordena as regras de `Domain` e as operações de persistência ou integração por meio dos contratos internos, sem depender das implementações concretas de `Infrastructure`.

Responsável por coordenar operações como:

- criar notícia;
- aprovar publicação;
- cadastrar evento;
- realizar inscrição;
- executar processos eleitorais.

---

## Infrastructure

Contém as implementações técnicas dos contratos definidos em `Application` ou `Domain`, incluindo consultas SQL e comunicação com serviços externos.

Exemplos:

- PostgreSQL;
- Firebase Authentication;
- Microsoft Graph;
- armazenamento de arquivos.

---

## API

Responsável pela comunicação externa.

Inclui:

- rotas HTTP;
- handlers;
- DTOs;
- respostas.

---

# Estrutura de diretórios

```
src/

├── modules/
├── infrastructure/
├── api/
├── shared/
└── config/
```

---

# Módulos de negócio

## identity

Responsável pela identidade dos usuários.

Inclui:

- autenticação institucional;
- integração com Firebase Authentication;
- integração com Microsoft;
- provisionamento de usuários;
- dados acadêmicos.

O módulo de identidade representa a identidade institucional do usuário e suas informações acadêmicas.

Ele não representa uma relação de pertencimento a um Centro Acadêmico.

A implementação atual cobre verificação institucional, registro da evidência e um extractor reutilizável para acesso protegido. Provisionamento e associação acadêmica permanecem fora desse recorte. Consulte [Autenticação institucional implementada](#autenticação-institucional-implementada) para o fluxo, contratos HTTP e limites atuais.

---

## ca

Responsável pelo gerenciamento dos Centros Acadêmicos e das relações administrativas entre usuários e CAs.

Inclui:

- criação e gerenciamento de CAs;
- informações do CA;
- perfis administrativos;
- cargos;
- permissões;
- associações administrativas entre usuários e CAs.

O modelo utilizado é:

```
Usuário

↓

Perfil administrativo

↓

Centro Acadêmico

↓

Permissões
```

Um usuário não pertence necessariamente a um CA. Um usuário pode possuir diferentes perfis administrativos em diferentes CAs.

---

## contexto_ca

Responsável pelo conceito de contexto ativo do aplicativo.

Inclui:

- CA favorito;
- seleção do CA ativo;
- troca de contexto.

O CA favorito é uma preferência de navegação utilizada como contexto inicial da aplicação.

Ele não representa pertencimento a um CA.

Fluxo:

```
Usuário inicia o aplicativo

↓

CA favorito

↓

Contexto ativo

↓

Informações exibidas
```

Durante o uso, o usuário pode alterar o CA ativo para visualizar informações de outros CAs disponíveis.

---

## news

Responsável pelo sistema de notícias.

Inclui:

- visualização de notícias do CA ativo;
- criação de notícias;
- publicação direta;
- proposição;
- homologação;
- compartilhamento entre CAs.

---

## events

Responsável pelo gerenciamento de eventos.

Inclui:

- criação de eventos;
- publicação;
- proposição;
- homologação;
- inscrições;
- controle de vagas.

---

## demands

Responsável pelo sistema de demandas.

Inclui dois contextos principais:

### Demandas institucionais

Relacionadas ao curso e à interação entre estudantes, CA e instituição.

### Demandas do espaço físico do CA

Relacionadas a problemas e necessidades de infraestrutura.

---

## materials

Responsável pelos materiais acadêmicos.

Inclui:

- publicação de materiais;
- arquivos;
- links;
- organização por disciplinas;
- acesso pelos estudantes.

---

## faq

Responsável pela base de conhecimento.

Inclui:

- cadastro de perguntas frequentes;
- consulta;
- suporte futuro a chatbot.

---

## elections

Responsável pelo sistema eleitoral.

Inclui:

- comissão eleitoral;
- configuração de eleições;
- chapas;
- candidatos;
- votação;
- apuração;
- criação de nova gestão.

---

## governance

Responsável pelas funcionalidades de transparência e participação.

Inclui:

- histórico de gestões;
- prestação de contas;
- consultas estudantis;
- mecanismos de governança.

---

## mentorship

Responsável pelo programa de apadrinhamento.

Inclui:

- participantes;
- padrinhos;
- afilhados;
- pareamentos;
- critérios de matching.

---

## strikes

Responsável pelo gerenciamento de strikes.

Inclui:

- registro de strikes individuais;
- registro de strikes associados a CAs;
- configuração de regras e consequências.

---

# Infraestrutura compartilhada

Localizada em:

```
src/infrastructure/
```

Contém recursos utilizados por múltiplos módulos.

---

## database

Responsável pelo acesso ao PostgreSQL.

O módulo `src/infrastructure/database/` contém `MIGRATOR` e `PgDatabase`. Este último encapsula o pool; as implementações dos repositórios ficam na infraestrutura de cada módulo de negócio. Criar ou clonar o adaptador não abre um novo pool, não inicia transação e não aplica migrations.

---

## firebase

O adaptador executável está em `src/modules/identity/infrastructure/firebase/`. O diretório global `src/infrastructure/firebase/` permanece reservado.

Responsabilidades:

- validação de tokens;
- identificação de usuários autenticados.

Não contém regras de autorização da aplicação.

---

## microsoft

O adaptador executável está em `src/modules/identity/infrastructure/microsoft/`; o diretório global permanece reservado. Consulta perfil, tenant e participação no grupo de estudantes. Recebe o access token por operação e o grupo a partir do cadastro institucional; não armazena credenciais de usuários.

---

## storage

Responsável pelo armazenamento de arquivos.

Exemplos:

- PDFs;
- documentos;
- imagens.

---

## notifications

Responsável por mecanismos de comunicação.

Exemplos:

- notificações push;
- integrações futuras.

---

# API global

Localizada em:

```
src/api/
```

Responsável por:

- registro de rotas;
- middleware;
- respostas padronizadas.

---

# Autenticação institucional implementada

## Componentes e responsabilidades

| Componente | Localização | Responsabilidade |
|---|---|---|
| Composição | `src/main.rs` e `src/api/state.rs` | Configurar clientes HTTP, verificador Firebase, adaptador PostgreSQL e estado. |
| HTTP | `src/modules/identity/api/` | Extrair credenciais, converter DTOs/erros e aplicar timeout. |
| Casos de uso | `src/modules/identity/application/services/authentication.rs` | Verificar a identidade e o vínculo; reutilizar uma evidência válida. |
| Contratos | `src/modules/identity/application/ports.rs` | Definir acesso aos provedores e à persistência sem dependência de HTTP. |
| Provedores | `src/modules/identity/infrastructure/{firebase,microsoft}/` | Validar Firebase ID tokens e consultar Microsoft Graph. |
| Persistência | `src/modules/identity/infrastructure/repository.rs` | Implementar as traits de repositório para `PgDatabase`. |

Os serviços são funções livres que recebem referências a traits. `AppState` guarda os adaptadores concretos e `Clients`, sem armazenar identidade ou token de um usuário. Seus clones compartilham o pool PostgreSQL, as conexões HTTP e o cache de chaves Firebase.

## Verificar ou renovar o acesso institucional

A rota disponível é `POST /auth/institutional/verify`. Ela não exige confirmação institucional anterior: precisa aceitar o primeiro acesso e a renovação de evidência vencida.

```http
POST /auth/institutional/verify
Authorization: Bearer <FIREBASE_ID_TOKEN>
Content-Type: application/json

{
  "microsoftAccessToken": "<MICROSOFT_ACCESS_TOKEN>"
}
```

Os tokens devem corresponder à mesma identidade. O serviço:

1. Valida assinatura e claims do Firebase ID token e obtém o UID e o objeto Microsoft vinculado.
2. Consulta `/me` no Graph e compara seu ID com o vínculo Firebase.
3. Consulta `/organization?$select=id` com o mesmo access token e busca a instituição pelo tenant.
4. Consulta `/me/checkMemberGroups` para `institution.microsoft_student_group_id`.
5. Persiste o resultado conclusivo em `institutional_verification` e, se positivo e aceito pelo repositório, retorna o perfil e a validade.

O cadastro institucional é necessário, mas o usuário interno ainda não precisa existir. A configuração do grupo é lida no login, não carregada na inicialização. O nome do grupo e o domínio do e-mail não substituem a consulta. `institution.status` não é usado pelo serviço como autorização, e não há restrição fixa ao tenant da UnB no código atual: o conjunto permitido corresponde às instituições cadastradas. Os [cenários compartilhados](../features/identidade_e_acesso/autenticacao_institucional.feature) distinguem instituição cadastrada, instituição desconhecida e vínculo estudantil recusado. A UnB é o suporte inicial; a instituição fictícia dos exemplos especifica a extensão desse comportamento, sem afirmar que outra integração já esteja disponível. A automação desses cenários permanece pendente.

Nome, e-mail, UPN e curso podem ser nulos. O curso vem de `jobTitle`; ausência ou texto vazio não impede a autenticação. O serviço não cria perfil interno, associa curso ou emite um token próprio.

Resposta HTTP 200, com dados ilustrativos:

```json
{
  "profile": {
    "microsoftObjectId": "00000000-0000-0000-0000-000000000001",
    "name": "Estudante",
    "email": null,
    "userPrincipalName": "estudante@example.edu",
    "courseName": null
  },
  "expiresAt": "2026-10-20T21:00:00Z"
}
```

`expiresAt` é a validade UTC atribuída à evidência persistida, serializada em RFC 3339. Não é o vencimento do ID token nem da sessão Firebase. O mobile pode guardar o perfil e antecipar a renovação por esse prazo; o backend continua responsável pela decisão de acesso.

O handler aplica timeout total de 15 segundos ao serviço. JSON rejeitado, access token ausente, nulo ou vazio resultam em 400. O normalizador atual não remove espaços: uma string composta só por espaços continua sendo encaminhada como credencial. Um timeout não garante reversão de uma escrita já concluída no banco.

## Reutilização da confirmação

O serviço `authenticate_institutional_request` valida o Firebase ID token, consulta uma evidência positiva vigente e confronta o objeto Microsoft vinculado. Não consulta o Graph, não altera o registro e não estende seu prazo. O extractor `AuthenticatedInstitutionalIdentity`, em `api/extractors.rs`, adapta essa operação ao Axum e limita o serviço a dez segundos.

Handlers protegidos devem receber explicitamente esse extractor. A mera existência dele não protege o router; nenhuma rota atual o utiliza. Não há endpoint adicional de identidade ou situação de login registrado. O handler de verificação usa apenas o parser Bearer e o serviço completo, evitando exigir a confirmação que ele próprio cria.

Quando o extractor for usado, as requisições protegidas precisarão somente de `Authorization: Bearer <FIREBASE_ID_TOKEN>`. Uma evidência ausente, negativa, vencida, incompatível com a configuração ou com outro objeto Microsoft resulta em `institutional_verification_required`. O mobile deverá obter credenciais adequadas, repetir a verificação institucional e só então tentar novamente a operação. Esse erro não determina, por si só, encerramento da sessão Firebase.

## Persistência, validade e concorrência

A tabela `institutional_verification` tem chave primária `firebase_uid`, sem FK para um usuário interno. Guarda objeto Microsoft, instituição, tenant/grupo consultados, resultado, início da consulta (`checked_at`) e expiração (`expires_at`). O UID é uma string opaca de até 128 caracteres; não é convertido em UUID.

O TTL atual é de **24 horas**, definido em `VERIFICATION_TTL_HOURS`. Começa antes das consultas ao Graph; mudar a constante afeta novas verificações, sem reescrever prazos já armazenados. A evidência não é um cache de sessão. Uma remoção do grupo pode permanecer desconhecida até nova consulta ou vencimento da confirmação anterior.

O UPSERT substitui todos os campos somente quando `checked_at` é estritamente posterior ao valor existente. Isso prioriza consultas iniciadas mais recentemente e pressupõe relógios sincronizados entre instâncias. Uma resposta negativa conclusiva também é gravada; indisponibilidade do provedor preserva o último resultado sem renovar sua validade. Se uma positiva não substituir um registro mais recente, o serviço pede nova verificação.

A leitura combina a evidência com `institution` e exige tenant e grupo iguais aos atuais. Mudar qualquer um deles impede a reutilização da confirmação antiga, sem reiniciar o cliente Graph. O serviço ainda compara o objeto Microsoft e verifica a expiração após a consulta. A tabela não guarda histórico nem remove automaticamente registros expirados. Excluir a instituição remove suas evidências por `ON DELETE CASCADE`.

## Cache das chaves e acesso aos provedores

O verificador Firebase aceita RS256 e verifica audiência, emissor, UID e claims temporais sem tolerância adicional (`leeway = 0`). Não consulta revogação ou situação atual da conta. O login e o gerenciamento da sessão permanecem no Firebase/mobile; a evidência institucional não os substitui.

As chaves públicas usam cache em memória compartilhado entre clones, com leituras por `RwLock` e atualização serializada por `Mutex`. O TTL considera `Cache-Control`, `Age` e o tempo de processamento. Tentativas por falha ou chave desconhecida são contidas por até 30 segundos; um TTL positivo menor antecipa a próxima atualização. Um conjunto recém-baixado pode atender à chamada corrente mesmo com TTL zero, mas não fica válido para reutilização posterior.

O cliente HTTP de produção não segue redirecionamentos, tem timeout de conexão de três segundos e padrão de dez segundos. As chamadas de JWKS e Graph impõem cinco segundos por requisição. O Graph espera HTTP 200; 401 vira credencial inválida e outros status, inclusive 403 e 429, viram indisponibilidade. Nenhum desses erros é convertido em resposta negativa de participação. A interpretação da participação pressupõe que a credencial tenha permissão para avaliar o grupo configurado.

## Erros públicos

As respostas produzidas por `ApiError` têm este envelope:

```json
{
  "error": {
    "code": "institutional_verification_required",
    "message": "É necessário verificar novamente seu vínculo institucional.",
    "details": null
  }
}
```

| HTTP | Código | Condição |
|---|---|---|
| 400 | `bad_request` | Corpo rejeitado ou access token obrigatório ausente/vazio. |
| 401 | `unauthorized` | Cabeçalho Bearer ausente, repetido ou malformado. |
| 401 | `invalid_token` | Credencial inválida; inclui 401 recebido do Graph. |
| 401 | `expired_token` | Expiração reconhecida pelo verificador Firebase. |
| 401 | `identity_mismatch` | IDs Microsoft válidos que não correspondem. |
| 401 | `invalid_microsoft_id` | Vínculo Microsoft ausente ou incompatível com UUID. |
| 403 | `unknown_institution` | Tenant sem instituição cadastrada. |
| 403 | `student_membership_required` | Consulta não confirmou participação no grupo. |
| 403 | `institutional_verification_required` | Evidência não reutilizável ou atualização positiva superada. |
| 503 | `authentication_provider_unavailable` | Provedor não permitiu concluir a validação. |
| 503 | `service_unavailable` | Timeout aplicado pela camada HTTP. |
| 500 | `internal_server_error` | Falha interna ou de persistência. |

Respostas 401 incluem `WWW-Authenticate: Bearer`. Somente a recusa de vínculo acrescenta `details.institution_name`; os demais erros acima usam `null`. O mobile deve decidir pelo código estável, usando mensagem e detalhes para apresentação. Diagnósticos SQL e tokens não fazem parte do corpo público. Os construtores genéricos de `ApiError` também estão disponíveis a outros handlers.

---

# Shared

Localizada em:

```
src/shared/
```

Contém código compartilhado entre módulos.

Exemplos:

- erros comuns;
- tipos reutilizados;
- paginação;
- utilidades genéricas.

---

# Configuração

Atualmente o executável lê `DATABASE_URL`, `FIREBASE_PROJECT_ID` e `RUST_LOG` diretamente do ambiente.
O módulo `src/config/` está reservado para centralizar essa configuração no futuro.
O backend não carrega arquivos `.env` automaticamente.

O [Compose](../compose.yaml) utiliza os valores abaixo e permite personalizá-los
no arquivo `.env` da raiz do monorepo. As variáveis já definidas no terminal têm
precedência sobre esse arquivo.

| Variável | Padrão local | Uso |
|---|---|---|
| `POSTGRES_DB` | `capp` | Nome do banco criado na inicialização do PostgreSQL. |
| `POSTGRES_USER` | `capp` | Usuário inicial do PostgreSQL. |
| `POSTGRES_PASSWORD` | `capp` | Senha local do usuário inicial. |
| `POSTGRES_PORT` | `5432` | Porta do PostgreSQL publicada no host. |
| `BACKEND_PORT` | `3000` | Porta da API publicada no host pelo Compose. |
| `RUST_LOG` | `info` | Filtro dos eventos de diagnóstico do backend. |
| `FIREBASE_PROJECT_ID` | Sem padrão válido | Obrigatória para o backend e os runners BDD; define audiência e emissor esperados nos ID tokens. O valor de `.env.example` é um placeholder. |

Os valores de usuário e senha acima destinam-se ao ambiente local de desenvolvimento.
Para personalizá-los, na primeira configuração e somente se ainda não existir
um `.env`, copie o exemplo a partir da raiz:

```bash
cp .env.example .env
```

Edite os valores necessários. O arquivo `.env` é ignorado pelo Git; mantenha o
[`.env.example`](../.env.example) como referência compartilhada.
O Compose pode receber os valores pelo ambiente sem `.env`. Subir somente o PostgreSQL dispensa `FIREBASE_PROJECT_ID`. Para iniciar o backend, defina um ID não vazio e sem espaços: quando ausente, o Compose repassa uma string vazia e o próprio backend encerra com erro de configuração. `docker compose config` valida o Compose, mas não a existência do projeto Firebase nem essa exigência da aplicação. O ID identifica o projeto; tokens, senhas e chaves privadas são credenciais distintas e não devem ser versionados.

Dentro da rede do Compose, o backend recebe uma `DATABASE_URL` construída com
usuário, senha e banco configurados, usando o endereço `postgres:5432`.
Ao executar com Cargo no host, forneça a URL explicitamente, usando
`localhost` e a porta publicada em `POSTGRES_PORT`.

`BACKEND_PORT` altera somente o mapeamento do Compose. O processo Rust escuta em
`0.0.0.0:3000`, tanto no container quanto na execução direta com Cargo.

---

# Banco de dados

O ambiente local usa PostgreSQL 17 Alpine. Na primeira inicialização de um volume
vazio, a imagem cria o usuário e o banco definidos pelas variáveis `POSTGRES_*`.
Os dados ficam no volume nomeado `postgres_data` do Compose e são reutilizados
nas próximas execuções.

Alterar essas variáveis no `.env` não modifica usuários, senhas ou bancos já
existentes no volume. Esse comportamento de inicialização é descrito na
[documentação da imagem PostgreSQL](https://hub.docker.com/_/postgres).

## Migrations

O versionamento do esquema usa o SQLx. Os arquivos SQL ficam em `migrations/`
e são incorporados ao binário pelo `MIGRATOR` de
[`src/infrastructure/database/mod.rs`](src/infrastructure/database/mod.rs).
O servidor aplica as versões pendentes após conectar ao PostgreSQL e antes de
abrir a porta HTTP. Uma falha de validação ou aplicação encerra a inicialização
com erro; a API não começa a atender com migrations pendentes por essa falha.

O SQLx mantém o histórico e os checksums na tabela `_sqlx_migrations` e coordena
execuções concorrentes com seu lock de migrations. O usuário da conexão precisa
poder criar essa tabela e executar as alterações SQL previstas. O banco deve
existir previamente; no ambiente local, o Compose cuida de sua criação.

As migrations atuais criam:

- `catalog_status`: enum PostgreSQL com `active` e `inactive`;
- `institution`: cadastro com tenant Microsoft único e grupo de estudantes;
- `institutional_verification`: última evidência por UID Firebase, com resultado, configuração verificada e validade.

Elas não inserem a UnB nem qualquer outra instituição automaticamente. Para autenticação real, cadastre uma instituição com tenant e grupo corretos antes do login. O enum `status` representa disponibilidade no catálogo; o serviço atual não o usa como filtro de autorização. Os demais componentes do [modelo relacional proposto](../MODELO_ENTIDADE_RELACIONAMENTO.md) ainda não são criados por essas migrations.

### Criar uma migration quando houver uma alteração de esquema

Instale a CLI na versão usada atualmente pelo projeto. Ela é uma ferramenta
opcional de desenvolvimento: o servidor e os testes aplicam migrations sem
precisar dela.

```bash
cargo install sqlx-cli --version 0.8.6 --locked --no-default-features --features rustls,postgres
```

Quando houver uma mudança real para versionar, execute no diretório `backend`,
substituindo `descricao_da_alteracao` por um nome curto em inglês e `snake_case`:

```bash
sqlx migrate add descricao_da_alteracao
```

Edite o arquivo gerado em `migrations/<timestamp>_<descricao>.sql` com a alteração
necessária e versione-o junto da implementação correspondente. O projeto usa
migrations simples, aplicadas em ordem de versão. Depois que uma migration for
integrada e aplicada em um ambiente compartilhado, preserve seu nome e conteúdo;
correções devem ser feitas em uma nova versão. O fluxo padrão evolui o esquema
para a frente, sem exigir um script de reversão para toda alteração.

### Aplicar e consultar versões

Iniciar o backend com Cargo ou Compose já aplica as migrations pendentes.
Para usar a CLI sem iniciar a API, mantenha o PostgreSQL disponível e execute no
diretório `backend`, usando os valores locais padrão:

```bash
DATABASE_URL='postgres://capp:capp@localhost:5432/capp' sqlx migrate run
DATABASE_URL='postgres://capp:capp@localhost:5432/capp' sqlx migrate info
```

Ajuste a URL se personalizou o ambiente. Reexecutar aplica somente as versões
pendentes e valida as já registradas. Consulte o
[`Migrator` do SQLx](https://docs.rs/sqlx/0.8.6/sqlx/migrate/struct.Migrator.html)
e a [documentação da CLI](https://github.com/launchbadge/sqlx/blob/v0.8.6/sqlx-cli/README.md)
para detalhes dos comandos e da validação.

### Compilação, Docker e testes

O `build.rs` instrui o Cargo a observar `migrations/`, incluindo arquivos novos.
Assim, um próximo `cargo build`, `cargo run` ou `cargo test` recompila o conteúdo
embutido quando esse diretório muda. O binário em execução continua usando as
migrations de sua compilação; reinicie-o com a versão recompilada.
Esse acompanhamento segue a
[orientação do SQLx para Rust stable](https://docs.rs/sqlx/0.8.6/sqlx/macro.migrate.html).

O Dockerfile copia `build.rs` e `migrations/` para o estágio de compilação. As migrations ficam embutidas no binário copiado para a imagem final. Após adicionar uma migration, execute novamente `docker compose --profile backend up -d --build --wait` na raiz do monorepo para reconstruir e iniciar a versão atualizada.

Os alvos BDD usam o mesmo `MIGRATOR` para preparar sua base-modelo antes da fixture.
A aplicação das migrations nos testes fica restrita aos containers descartáveis
criados pelas suítes. Para validar uma nova migration, execute os testes do backend
conforme a [seção BDD](#bdd).

---

# Desenvolvimento

## Pré-requisitos

- Docker com daemon acessível, `docker compose` e Buildx/BuildKit disponíveis para construir a imagem com caches de compilação.
- Acesso à internet na primeira execução para obter imagens e dependências.
- Portas locais livres; os padrões são `5432` para o banco e `3000` para a API.
- Para executar o backend ou os testes fora do container, Rust e Cargo da
  toolchain `stable`, instalados via `rustup`.
- `curl` para as verificações HTTP mostradas abaixo.

No Windows, execute os comandos do backend no WSL 2 com acesso ao Docker.
No Linux, use o terminal do próprio sistema. Não é necessário instalar PostgreSQL
no host; para executar toda a aplicação pelo Compose, o Rust é fornecido pela imagem.

## PostgreSQL e backend pelo Compose

Na raiz do monorepo, valide a configuração e inicie os serviços:

```bash
docker compose --profile backend config --quiet
docker compose --profile backend up -d --build --wait
```

O perfil `backend` inclui a API, enquanto o PostgreSQL participa do ambiente por
padrão. O Compose aguarda o healthcheck do banco antes de iniciar o backend e
fornece sua `DATABASE_URL` automaticamente. Com `--wait`, o comando também aguarda o backend ficar saudável: seu healthcheck consulta `http://127.0.0.1:3000/ready` dentro do container, verificando a API e sua conexão com PostgreSQL.

Acompanhe o estado dos serviços e os logs:

```bash
docker compose --profile backend ps
docker compose logs -f backend
```

O estágio de build executa `cargo build --release --locked --bin backend`. A primeira construção pode demorar; as seguintes reutilizam caches BuildKit de dependências e compilação. A imagem final contém o binário, certificados TLS e `curl` para a sondagem, sem o compilador ou os fontes. O processo executa diretamente o binário como usuário sem privilégios de root e aplica migrations antes de abrir a porta HTTP.

O healthcheck roda a cada dez segundos, limita a requisição a três segundos e permite cinco falhas consecutivas depois de uma tolerância inicial de vinte segundos. Ele não testa Firebase ou Microsoft Graph, nem exige uma instituição cadastrada. Marcar o container como `unhealthy` não o reinicia automaticamente.

`Ctrl+C` encerra apenas o acompanhamento dos logs. Após alterar o código ou as migrations, execute novamente o comando com `--build --wait`, pois não há montagem dos fontes como volume nem recompilação na inicialização do container.

## PostgreSQL pelo Compose e backend com Cargo

Para desenvolver diretamente com o compilador local, suba somente o banco a
partir da raiz e aguarde seu healthcheck:

```bash
docker compose config --quiet
docker compose up -d --wait postgres
```

Caso a API já esteja rodando pelo Compose, pare esse serviço antes de usar a
mesma porta com Cargo:

```bash
docker compose --profile backend stop backend
```

Depois, usando os valores locais padrão:

```bash
cd backend
DATABASE_URL='postgres://capp:capp@localhost:5432/capp' \
FIREBASE_PROJECT_ID='seu-project-id' \
RUST_LOG=info cargo run --locked
```

Substitua `seu-project-id` pelo projeto Firebase usado pelo aplicativo. Se personalizou usuário, senha, banco ou porta, ajuste a URL desse comando.
O `.env` da raiz é lido pelo Compose; ele não exporta variáveis para o processo
Cargo. Mantenha esse terminal aberto enquanto usar a API e utilize `Ctrl+C` para
encerrar o backend.

## Verificar a comunicação

Com a API em execução, use outro terminal. Estes endereços consideram a porta
padrão; ajuste-a se estiver usando outro `BACKEND_PORT` no Compose:

```bash
curl -i http://localhost:3000/health
curl -i http://localhost:3000/ready
```

`/health` retorna HTTP 200 quando a API responde. `/ready` executa uma consulta
no PostgreSQL e retorna HTTP 200 no sucesso ou HTTP 503 em erro ou timeout.

Para consultar o banco diretamente, na raiz do monorepo:

```bash
docker compose exec postgres sh -c 'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT 1;"'
```

Esse comando usa o `psql` do container e as configurações fornecidas ao serviço.

## Encerrar o ambiente e executar testes

Na raiz, para parar e remover os containers mantendo o volume com os dados:

```bash
docker compose --profile backend down
```

Para executar os testes, no diretório `backend`, forneça o projeto usado pela suíte:

```bash
export FIREBASE_PROJECT_ID='seu-project-id'
cargo test --all-features --locked --no-fail-fast
```

As suítes BDD criam seus próprios containers e não precisam que o ambiente Compose
esteja iniciado. Consulte a [seção BDD](#bdd) para executar cada suíte separadamente.

---

# Implantação

## Escopo da milestone 1

Para a fundação do projeto, o ambiente controlado é o ambiente local de
desenvolvimento, com a API Rust e o PostgreSQL. O Compose versiona a definição
dos serviços, e a [configuração](#configuração) documenta as variáveis utilizadas.
A execução local da API e do PostgreSQL já foi confirmada pelo mantenedor.

Esse escopo atende à infraestrutura inicial da
[issue #27](https://github.com/Henrique-zoo/CApp/issues/27), sem exigir uma VM
dedicada ou hospedagem pública. A execução das funcionalidades do aplicativo
será validada conforme elas forem implementadas nas entregas seguintes.

## Disponibilizar e atualizar a versão local

1. Confira os [pré-requisitos](#pré-requisitos) e a [configuração](#configuração).
2. Com o código da versão desejada no checkout, siga a
   [execução pelo Compose](#postgresql-e-backend-pelo-compose). Use novamente
   `docker compose --profile backend up -d --build --wait` após alterações para
   reconstruir a imagem e iniciar a versão atualizada. Como alternativa, siga
   a [execução com Cargo](#postgresql-pelo-compose-e-backend-com-cargo),
   encerrando o processo anterior antes de iniciar o novo.
3. Aguarde a inicialização e [verifique a comunicação](#verificar-a-comunicação)
   com `/health` e `/ready`. As [migrations](#migrations) pendentes são aplicadas
   pelo backend antes de abrir a porta HTTP.
4. Ao terminar, siga o [encerramento do ambiente](#encerrar-o-ambiente-e-executar-testes),
   que preserva o volume do banco para a próxima execução.

## Hospedagem externa

A implantação em servidor externo ainda não foi realizada. Ela será definida
quando uma entrega precisar de acesso remoto ou operação contínua. Nessa etapa,
este guia deverá registrar o destino escolhido, a configuração de acesso e
segredos, a persistência dos dados e os comandos de publicação e atualização.
A imagem já executa um binário compilado em modo release. O Compose continua destinado ao ambiente local; a configuração de operação no ambiente hospedado será definida junto dessa implantação.

---

# BDD

O projeto utiliza Behavior Driven Development.

Os arquivos de especificação comportamental ficam no diretório raiz do monorepo:

```
features/
```

Cada funcionalidade possui:

- descrição do comportamento esperado;
- cenários Gherkin;
- implementações de teste específicas para cada cliente.

O backend implementa os testes comportamentais relacionados às regras de negócio e API.

Essas especificações compartilhadas descrevem comportamentos do produto na
linguagem do domínio. O backend implementa seus steps em
`tests/step_definitions/`; cada cliente possui sua própria implementação dos
comportamentos aplicáveis ao seu contexto.

Os cenários técnicos que verificam `/ready`, PostgreSQL e o isolamento dos testes
ficam em `tests/features/`, com steps exclusivos em `tests/infrastructure_steps/`.
Eles validam a infraestrutura do backend e são executados por um alvo separado.

| Alvo Cargo | Cenários | Steps |
|---|---|---|
| `bdd` | `../features/` — comportamentos compartilhados do produto | `tests/step_definitions/` |
| `bdd_infrastructure` | `tests/features/` — verificações técnicas do backend | `tests/infrastructure_steps/` |

O runner exclui automaticamente cenários com `@pending_backend`, considerando
tags na feature, regra ou cenário. Essa exclusão também vale quando filtros
da CLI são informados; `@pending_mobile` não impede a execução no backend.
Remova a tag da plataforma quando sua automação estiver implementada.
A exclusão pode resultar em zero cenários e não comprova cobertura funcional.

Para executar os testes BDD, é necessário ter Rust/Cargo instalados e um daemon
Docker disponível para o usuário atual. Na primeira execução, o Docker precisa conseguir obter a imagem `postgres:17-alpine`.

Os runners exigem `FIREBASE_PROJECT_ID` no ambiente, inclusive para cenários técnicos, porque constroem o mesmo `AppState` com clientes reais. Eles não carregam `.env` automaticamente. Construir os clientes não faz login nem chama os provedores; um cenário que execute autenticação real precisa também de tokens válidos e de dados institucionais em seu banco. O ID do projeto sozinho não autentica os testes.

No diretório `backend`, verifique o Docker e execute os comportamentos do produto:

```bash
export FIREBASE_PROJECT_ID='seu-project-id'
docker info
cargo test --test bdd --locked
```

Para executar todos os cenários técnicos ou somente o de `/ready`:

```bash
cargo test --test bdd_infrastructure --locked
cargo test --test bdd_infrastructure --locked -- --tags @ready
```

Para executar todos os testes com o mesmo comando usado na CI:

```bash
cargo test --all-features --locked --no-fail-fast
```

O Cargo executa os alvos `bdd` e `bdd_infrastructure` uma única vez, além dos
outros testes do backend. `--no-fail-fast` permite executar os demais alvos mesmo
quando um deles falha; o comando continua retornando falha se qualquer alvo
falhar. A opção controla a execução entre alvos, não a estratégia interna dos
cenários do Cucumber.

Os dois alvos reutilizam o código de `tests/support/`, mas registram seus steps e
leem seus diretórios de cenários separadamente. Cada execução de um alvo cria um
único container PostgreSQL, reutilizado por todos os cenários daquela suíte, com
porta dinâmica. Executar os dois alvos cria dois containers independentes. A
preparação usa o mesmo `MIGRATOR` do servidor e aplica a fixture
`tests/fixtures/api_bdd.sql` a uma base-modelo.
Cenários consultivos compartilham uma cópia dessa base com transações de leitura
por padrão. Cenários que escrevem devem receber a tag `@isolated_database`, na
feature, regra ou cenário, para obter uma base própria clonada do modelo.

Os testes usam o router e o `AppState` reais em memória. Não é necessário iniciar
o servidor HTTP nem o PostgreSQL do Compose; a suíte não lê `DATABASE_URL` e usa
credenciais próprias no container descartável. Os clientes HTTP e o cache de chaves Firebase são compartilhados na suíte; a evidência institucional permanece no banco de cada cenário. O POST de verificação grava uma evidência e exige `@isolated_database` quando executado por um cenário. Os hooks liberam os recursos dos cenários, e o encerramento da suíte remove o container.

Os resultados de cada suíte aparecem na saída do comando, com a quantidade de
cenários e steps executados e os detalhes de eventuais falhas. O Cargo identifica
cada alvo antes de executá-lo, e o Cucumber apresenta o resumo da respectiva suíte.

No GitHub Actions, abra o workflow **Backend CI**, selecione a execução e consulte
**Format, lint, and test → Run tests**. O runner Ubuntu fornece Docker para os
containers criados pelo Testcontainers. O step executa diretamente o comando acima,
preservando seu código de saída para que falhas nos testes reprovem o job. Configure `FIREBASE_PROJECT_ID` em **Settings → Secrets and variables → Actions → Variables** do repositório; o workflow a fornece ao processo com `${{ vars.FIREBASE_PROJECT_ID }}`. Não é necessário gerar um `.env` no runner.

As instruções de organização e escrita dos cenários compartilhados estão na
[seção BDD do README do projeto](../README.md#bdd).

A aprovação dos testes técnicos não representa cobertura dos comportamentos do
produto. Os cenários de autenticação atualmente usam `@pending_backend` e passos pendentes; sua automação é uma entrega separada. O alvo `bdd` pode terminar com zero cenários executados, sem validar autenticação. Compilar a suíte ou aprovar os cenários técnicos não comprova integração real com Firebase/Graph.

---

# Documentação Rust

Os Rustdocs fazem parte da entrega de cada implementação. Devem acompanhar as
alterações de comportamento no mesmo PR, incluindo o código de testes.

- Use `//!` para explicar a responsabilidade de crates e módulos, suas relações
  e o estado atual de implementação. Módulos reservados devem ser identificados
  como tal, sem apresentar funcionalidades futuras como disponíveis.
- Use `///` em tipos, campos, constantes e funções, inclusive itens internos
  relevantes à manutenção. Em implementações de traits, aproveite o contrato do
  trait e documente os detalhes específicos quando necessário.
- Comece com um resumo curto. Explique contratos, invariantes, efeitos colaterais,
  configuração, concorrência e ciclo de vida dos recursos quando forem relevantes.
  Evite repetir a assinatura ou narrar cada linha da implementação.
- Mantenha o texto em português. Use seções `Parâmetros`, `Retorno`, `Erros` e
  `Exemplos` quando acrescentarem informação; use `Panics` para as condições que
  interrompem a execução por panic, separadamente dos erros retornados em `Result`.
  Funções `unsafe`, caso sejam introduzidas, devem explicar suas precondições em
  uma seção `Safety`.
- Prefira links Rustdoc para tipos e funções, como ``[`AppState`]``, e exemplos
  pequenos que compilem. Use `no_run` quando o exemplo depender de serviços
  externos; reserve `ignore` para casos que não possam ser compilados, explicando
  o motivo. Comandos de terminal e Gherkin devem indicar sua linguagem no bloco.

As convenções seguem o [Rustdoc Book](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
e as [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/documentation.html).

No diretório `backend`, gere a documentação da biblioteca, incluindo itens privados:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --lib --all-features --no-deps --document-private-items --locked
```

O índice fica em `target/doc/backend/index.html`. A documentação da biblioteca
não inclui os módulos dos testes; para gerar também as duas suítes, usando o
diretório `target/` padrão:

```bash
cargo build --lib --all-features --locked
cargo rustdoc --test bdd --all-features --locked -- --document-private-items -D warnings --extern backend=target/debug/libbackend.rlib
cargo rustdoc --test bdd_infrastructure --all-features --locked -- --document-private-items -D warnings --extern backend=target/debug/libbackend.rlib
```

O argumento `--extern` fornece a biblioteca do projeto que o Cargo não inclui
automaticamente nesta geração de documentação dos testes. Os índices ficam em
`target/doc/bdd/index.html` e `target/doc/bdd_infrastructure/index.html`.
Esses comandos documentam os testes sem executá-los e não precisam de Docker.

Para documentar somente a inicialização do executável, use:

```bash
cargo rustdoc --bin backend --all-features --locked -- --document-private-items -D warnings
```

O executável e a biblioteca têm o mesmo nome e usam o mesmo índice
`target/doc/backend/index.html`. O Cargo pode avisar sobre essa colisão;
execute novamente o comando da biblioteca para restaurar seu índice.

Valide os exemplos compiláveis da biblioteca com:

```bash
cargo test --doc --all-features --locked
```

O manifesto habilita avisos para itens públicos sem documentação, links Rustdoc
quebrados e HTML inválido. A CI existente usa Clippy com `-D warnings`, portanto
itens públicos sem Rustdoc impedem sua aprovação. A revisão dos itens privados e
da precisão dos contratos continua necessária; esses avisos não avaliam a
qualidade do texto. Os comandos de documentação acima também tratam avisos como
erros e devem ser executados ao alterar Rustdocs.

---

# Tecnologias

| Tecnologia | Uso |
|---|---|
| Rust | Backend |
| Axum | Framework HTTP |
| Tokio | Runtime assíncrono |
| SQLx | Banco de dados |
| PostgreSQL | Persistência |
| Firebase Authentication | Identidade |
| Microsoft Graph | Dados acadêmicos |
| Docker | Ambiente |

---

# Filosofia

O backend é uma API independente dos clientes.

O aplicativo Flutter é apenas um consumidor da API.

Futuramente, um frontend web poderá utilizar a mesma infraestrutura sem duplicar regras de negócio.

```
              Rust Backend

                    ↑

        ┌───────────┴───────────┐

        │                       │

Flutter Mobile             Web Frontend
```

As regras de negócio permanecem centralizadas no backend, enquanto diferentes clientes implementam apenas suas respectivas interfaces.
