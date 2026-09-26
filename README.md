# CApp

Plataforma digital para Centros Acadêmicos, composta por um aplicativo
móvel e um backend compartilhado.

O objetivo do CApp é fornecer uma infraestrutura comum para que
diferentes Centros Acadêmicos possam disponibilizar informações,
organizar atividades, gerenciar processos internos e melhorar a
comunicação com estudantes.

O projeto é estruturado como um monorepo contendo os diferentes
componentes da aplicação:

    capp/
    │
    ├── backend/
    │
    ├── mobile/
    │
    ├── features/
    │
    └── README.md

------------------------------------------------------------------------

# Arquitetura do projeto

O CApp segue uma arquitetura de múltiplos clientes utilizando um backend
centralizado.

                        Backend Rust

                             ↑

              ┌──────────────┴──────────────┐

              │                             │

        Flutter Mobile                 Web Frontend
         (Android/iOS)                  (futuro)

O backend concentra:

-   regras de negócio;
-   autenticação;
-   autorização;
-   persistência;
-   integração com serviços externos.

Os clientes são responsáveis apenas pela interface e experiência do
usuário.

------------------------------------------------------------------------

# Estrutura do monorepo

## backend/

Contém a API principal da aplicação.

Tecnologias principais:

-   Rust;
-   Axum;
-   Tokio;
-   SQLx;
-   PostgreSQL.

Responsabilidades:

-   implementar regras de negócio;
-   disponibilizar endpoints HTTP;
-   controlar permissões;
-   gerenciar dados persistentes.

Documentação específica:

    backend/README.md

------------------------------------------------------------------------

## mobile/

Contém o aplicativo móvel.

Tecnologias principais:

-   Flutter;
-   Dart;
-   Kotlin (Android);
-   Swift (iOS).

O Flutter é responsável pela maior parte da interface compartilhada.

Código específico de plataforma fica separado quando necessário:

    mobile/

    ├── lib/
    │
    ├── android/
    │
    ├── ios/
    │
    └── integration_test/

Responsabilidades:

-   telas;
-   navegação;
-   experiência do usuário;
-   integração com a API.

------------------------------------------------------------------------

## features/

Contém as especificações comportamentais do produto utilizando BDD.

Exemplo:

    features/

    ├── autenticacao.feature
    ├── noticias.feature
    ├── eventos.feature
    └── eleicoes.feature

Esses arquivos representam o comportamento esperado do sistema,
independentemente da implementação.

Eles servem como fonte comum para:

-   backend;
-   aplicativo móvel;
-   futuros clientes.

Os cenários descrevem comportamentos observáveis do produto na linguagem do
domínio. Cada projeto implementa suas próprias step definitions para verificar
esses comportamentos no contexto correspondente.

As verificações técnicas de PostgreSQL, prontidão e isolamento da infraestrutura
de testes ficam em `backend/tests/features/`, com steps e execução exclusivos do
backend. Elas não fazem parte do contrato funcional compartilhado em `features/`.

------------------------------------------------------------------------

# BDD

O projeto utiliza Behavior Driven Development.

O fluxo de desenvolvimento é:

    Especificação Gherkin

            ↓

    Implementação dos testes

            ↓

    Implementação da funcionalidade

            ↓

    Validação do comportamento

Os arquivos `.feature` descrevem comportamentos do produto.

Exemplo:

``` gherkin
# language: pt

Funcionalidade: Gerenciamento de notícias

Como usuário do CApp
Quero visualizar notícias do CA ativo
Para acompanhar informações relevantes
```

As convenções para os cenários compartilhados são:

- Declare `# language: pt` no início dos arquivos que usam palavras-chave em português.
- Organize os arquivos por domínio funcional e use nomes descritivos em `snake_case`.
- Descreva cada `Cenário` pelo comportamento ou resultado esperado, com atores e
  termos do produto.
- Use `Dado` para o contexto, `Quando` para a ação e `Então` para o resultado
  observável; complemente com `E` quando necessário.
- Mantenha os cenários independentes, sem depender da ordem de execução ou dos
  dados deixados por outro cenário.
- Deixe HTTP, SQL, containers e preparação dos dados nas step definitions de cada
  componente. As verificações técnicas do backend permanecem em sua suíte própria.

Os cenários concretos de cada funcionalidade serão adicionados nas milestones
correspondentes. A configuração inicial das suítes não representa cobertura
funcional dos arquivos que ainda contêm somente descrições de escopo.

O backend implementa os comportamentos relacionados a:

-   regras de negócio;
-   permissões;
-   API;
-   persistência.

O aplicativo implementa os comportamentos relacionados a:

-   navegação;
-   telas;
-   interação do usuário.

------------------------------------------------------------------------

# Desenvolvimento do backend

O backend é desenvolvido preferencialmente em ambiente Linux.

A recomendação é utilizar:

-   WSL 2;
-   Docker;
-   containers Linux.

Motivos:

-   o ambiente de produção será Linux;
-   ferramentas de backend possuem melhor integração nesse ambiente;
-   evita diferenças entre desenvolvimento e deploy;
-   facilita uso de PostgreSQL, Docker e ferramentas de infraestrutura.

Estrutura recomendada:

    Windows

    └── WSL 2

        └── Projeto CApp

            └── backend

O backend deve ser executado dentro do WSL 2.

------------------------------------------------------------------------

# Desenvolvimento do aplicativo móvel

O aplicativo Flutter deve ser desenvolvido preferencialmente no Windows.

Motivos:

-   Flutter possui excelente suporte para desenvolvimento no Windows;
-   ferramentas como Android Studio, emuladores Android e hot reload
    funcionam bem;
-   integração com dispositivos físicos Android é mais simples.

Estrutura recomendada:

    Windows

    └── Projeto CApp

        └── mobile

O desenvolvimento iOS exige macOS e Xcode quando for necessário compilar
ou testar nativamente para iPhone.

------------------------------------------------------------------------

# Por que não desenvolver tudo no WSL 2?

Apesar de o WSL 2 ser excelente para backend, ele não é ideal para todo
o projeto.

O aplicativo possui necessidades diferentes:

Backend:

    Linux
     ↓
    Docker
     ↓
    Servidor
     ↓
    Banco

Mobile:

    Windows
     ↓
    Flutter
     ↓
    Android Studio
     ↓
    Emulador/dispositivo

Misturar os dois ambientes gera dificuldades desnecessárias:

-   acesso a emuladores;
-   integração USB;
-   ferramentas gráficas;
-   arquivos montados entre Windows e Linux.

Por isso, a recomendação é:

  Parte                 Ambiente recomendado
  --------------------- ----------------------
  Backend Rust          WSL 2
  Docker                WSL 2
  PostgreSQL local      WSL 2/Docker
  Flutter               Windows
  Android Emulator      Windows
  Desenvolvimento iOS   macOS

------------------------------------------------------------------------

# Fluxo de desenvolvimento

Um desenvolvedor pode trabalhar assim:

    Windows

    ├── VS Code
    │
    ├── Flutter
    │
    └── Mobile App


    WSL 2

    ├── Rust Backend
    │
    ├── Docker
    │
    └── Banco de dados

O VS Code pode integrar os dois ambientes utilizando:

-   Remote WSL;
-   extensões Flutter/Dart;
-   extensões Rust Analyzer.

------------------------------------------------------------------------

# Comunicação entre componentes

O fluxo normal será:

    Flutter App

          |

          | HTTP/HTTPS + JSON

          ↓

    Rust Backend

          |

          ↓

    PostgreSQL

Autenticação:

    Flutter

    ↓

    Firebase Authentication

    ↓

    Backend valida identidade

    ↓

    Aplicação aplica permissões

------------------------------------------------------------------------

# Tecnologias principais

  Tecnologia                Uso
  ------------------------- ---------------------------
  Rust                      Backend
  Axum                      API HTTP
  Tokio                     Runtime assíncrono
  SQLx                      Banco de dados
  PostgreSQL                Persistência
  Docker                    Infraestrutura local
  Flutter                   Aplicativo móvel
  Dart                      Interface compartilhada
  Kotlin                    Código Android específico
  Swift                     Código iOS específico
  Firebase Authentication   Identidade
  Microsoft Graph           Dados institucionais

------------------------------------------------------------------------

# Futuro frontend web

O backend foi projetado para ser reutilizado por diferentes clientes.

Quando o site for desenvolvido, ele poderá ser adicionado ao monorepo:

    capp/

    ├── backend/

    ├── mobile/

    ├── web/

    ├── features/

    └── README.md

O frontend web consumirá a mesma API utilizada pelo aplicativo móvel.

As regras de negócio permanecerão centralizadas no backend.

------------------------------------------------------------------------

# Princípios do projeto

-   Backend independente dos clientes.
-   Regras de negócio centralizadas.
-   BDD como especificação do comportamento.
-   Arquitetura modular.
-   Separação entre domínio e infraestrutura.
-   Ambiente de desenvolvimento próximo ao ambiente de produção.
