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

O fluxo de dependências segue:

```
API

↓

Application

↓

Domain

↓

Infrastructure
```

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

Não possui conhecimento sobre:

- banco de dados;
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

Responsável por coordenar operações como:

- criar notícia;
- aprovar publicação;
- cadastrar evento;
- realizar inscrição;
- executar processos eleitorais.

---

## Infrastructure

Contém implementações técnicas.

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

Inclui:

- pool de conexões;
- transações;
- persistência.

---

## firebase

Responsável pela integração com Firebase Authentication.

Responsabilidades:

- validação de tokens;
- identificação de usuários autenticados.

Não contém regras de autorização da aplicação.

---

## microsoft

Responsável pela integração com Microsoft Graph.

Utilizado para obtenção de informações acadêmicas.

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

Atualmente o executável lê `DATABASE_URL` e `RUST_LOG` diretamente do ambiente.
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

Os valores de usuário e senha acima destinam-se ao ambiente local de desenvolvimento.
Para personalizá-los, na primeira configuração e somente se ainda não existir
um `.env`, copie o exemplo a partir da raiz:

```bash
cp .env.example .env
```

Edite os valores necessários. O arquivo `.env` é ignorado pelo Git; mantenha o
[`.env.example`](../.env.example) como referência compartilhada.
O Compose também funciona sem `.env`, usando os padrões da tabela.

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

Ainda não há modelo de dados nem migrations de domínio. O diretório contém
somente `.keep`, e executar o migrador prepara apenas sua tabela de controle.
As primeiras migrations serão adicionadas junto da implementação do modelo.

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

O Dockerfile copia `build.rs` e `migrations/` para a imagem. Após adicionar uma
migration, execute novamente `docker compose --profile backend up -d --build`
na raiz do monorepo para reconstruir e iniciar a versão atualizada.

Os alvos BDD usam o mesmo `MIGRATOR` para preparar sua base-modelo antes da fixture.
A aplicação das migrations nos testes fica restrita aos containers descartáveis
criados pelas suítes. Para validar uma nova migration, execute os testes do backend
conforme a [seção BDD](#bdd).

---

# Desenvolvimento

## Pré-requisitos

- Docker com daemon acessível e o comando `docker compose` disponível.
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
docker compose --profile backend up -d --build
```

O perfil `backend` inclui a API, enquanto o PostgreSQL participa do ambiente por
padrão. O Compose aguarda o healthcheck do banco antes de iniciar o backend e
fornece sua `DATABASE_URL` automaticamente.

Acompanhe o estado dos serviços e os logs:

```bash
docker compose --profile backend ps
docker compose logs -f backend
```

O container do backend executa `cargo run`, portanto a primeira inicialização
pode levar algum tempo compilando dependências. Espere a compilação terminar
antes de consultar a API. `Ctrl+C` encerra apenas o acompanhamento dos logs.
Após alterar o código, execute novamente o comando com `--build`, pois o código
é copiado para a imagem e não está montado como um volume.

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
DATABASE_URL='postgres://capp:capp@localhost:5432/capp' RUST_LOG=info cargo run --locked
```

Se personalizou usuário, senha, banco ou porta, ajuste a URL desse comando.
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

Para executar os testes, no diretório `backend`:

```bash
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
   `docker compose --profile backend up -d --build` após alterações para
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
O Dockerfile atual executa `cargo run` e serve ao desenvolvimento local; a forma
de execução no ambiente hospedado será definida junto dessa implantação.

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
Docker disponível para o usuário atual. Na primeira execução, o Docker precisa
conseguir obter a imagem `postgres:17-alpine`.

No diretório `backend`, verifique o Docker e execute os comportamentos do produto:

```bash
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
credenciais próprias no container descartável. Os hooks liberam os recursos dos
cenários, e o encerramento da suíte remove o container.

Os resultados de cada suíte aparecem na saída do comando, com a quantidade de
cenários e steps executados e os detalhes de eventuais falhas. O Cargo identifica
cada alvo antes de executá-lo, e o Cucumber apresenta o resumo da respectiva suíte.

No GitHub Actions, abra o workflow **Backend CI**, selecione a execução e consulte
**Format, lint, and test → Run tests**. O runner Ubuntu fornece Docker para os
containers criados pelo Testcontainers. O step executa diretamente o comando acima,
preservando seu código de saída para que falhas nos testes reprovem o job.

As instruções de organização e escrita dos cenários compartilhados estão na
[seção BDD do README do projeto](../README.md#bdd).

A aprovação dos testes técnicos não representa cobertura dos comportamentos do
produto. Enquanto as especificações funcionais não tiverem cenários, o alvo
`bdd` pode terminar com zero cenários executados, sem validar regras de negócio.

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
