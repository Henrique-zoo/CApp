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

Localizada em:

```
src/config/
```

Responsável por carregar configurações externas.

Exemplos:

```
DATABASE_URL
FIREBASE_PROJECT_ID
MICROSOFT_CLIENT_ID
```

---

# Banco de dados

As alterações do banco são controladas por migrations.

Exemplo:

```
migrations/

001_initial_schema.sql
002_create_news.sql
003_create_events.sql
```

Cada migration representa uma evolução versionada do banco.

---

# Desenvolvimento

Executar aplicação:

```bash
cargo run
```

Executar testes:

```bash
cargo test
```

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
preparação aplica as migrations e a fixture `tests/fixtures/api_bdd.sql` a uma base-modelo.
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
