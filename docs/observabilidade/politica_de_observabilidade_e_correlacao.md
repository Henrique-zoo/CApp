# Política de Observabilidade, Rastreabilidade e Correlação de Requisições

## 1. Visão Geral e Objetivos

Esta política estabelece os padrões e mecanismos de observabilidade, rastreabilidade e telemetria do backend do projeto CApp. O objetivo central é assegurar visibilidade completa sobre o ciclo de vida das requisições HTTP, permitindo:

1. **Correlação Ponta a Ponta:** Rastrear qualquer transação individual desde a chegada ao servidor até sua conclusão ou falha, associando logs e métricas a um identificador exclusivo.
2. **Diagnóstico Rápido e Não Invasivo:** Facilitar a investigação de erros (4xx e 5xx) e lentidões sem expor dados confidenciais.
3. **Compatibilidade com Coletores de Log Modernos:** Permitir ingestão estruturada em pipelines de observabilidade (ex: Datadog, Grafana Loki, CloudWatch, Elasticsearch) com saída em JSON delimitado por quebra de linha.
4. **Isolamento e Segurança Operacional:** Mascarar preventivamente segredos, credenciais e tokens em qualquer formato de log emitido.

---

## 2. Identificador de Correlação (`x-request-id`)

O identificador de correlação é o elo unificador de rastreabilidade do sistema.

### 2.1 Regras de Entrada e Validação
- **Cabeçalho:** `x-request-id` (case-insensitive conforme RFC 7230 / RFC 9110).
- **Preservação de IDs de Clientes:** Se o cliente ou gateway upstream fornecer um valor válido, o backend preserva-o integralmente para viabilizar rastreamento distribuído.
- **Critérios de Validade:**
  - O valor deve ser não-vazio.
  - O tamanho não pode exceder 128 caracteres (`MAX_REQUEST_ID_LENGTH`), prevenindo ataques de saturação de memória ou injeção de cabeçalhos.
  - O conteúdo deve ser composto estritamente por caracteres ASCII gráficos visíveis (`33..=126`), rejeitando espaços, quebras de linha (`\r`, `\n`) e caracteres de controle.
- **Geração de Novo Identificador:** Caso o cabeçalho esteja ausente ou viole os critérios de validação, o backend gera um novo identificador exclusivo baseado no padrão **UUID v4**.

### 2.2 Propagação e Injeção de Saída
- O cabeçalho `x-request-id` é **obrigatoriamente injetado em 100% das respostas HTTP**, abrangendo sucessos (2xx), redirecionamentos (3xx), erros de cliente (4xx, incluindo rotas inexistentes) e falhas internas (5xx).
- O identificador é disponibilizado internamente através da extensão de requisição `backend::api::request_id::RequestId`, podendo ser injetado diretamente em handlers ou serviços via extratores do Axum.

---

## 3. Instrumentação HTTP e Spans Tracing

A telemetria HTTP é implementada por meio de uma camada customizada baseada em `tower_http::trace::TraceLayer`.

### 3.1 Estrutura do Span `http_request`
Cada requisição atendida pelo servidor cria um span assíncrono vinculado denominado `http_request`, composto pelos seguintes campos contextuais:

| Campo Contextual | Tipo | Descrição |
| :--- | :--- | :--- |
| `request_id` | String | Identificador de correlação validado ou gerado. |
| `method` | String | Método HTTP da requisição (ex: `GET`, `POST`, `PUT`, `DELETE`). |
| `uri` | String | Caminho e parâmetros de consulta (`path` e `query`). |
| `status` | u16 (opcional) | Código de status HTTP final (preenchido no término da requisição). |
| `latency_ms` | u64 (opcional) | Tempo decorrido em milissegundos desde o recebimento até a resposta. |

### 3.2 Herança de Contexto em Eventos de Log
Como o pipeline de processamento da requisição é executado no escopo do span assíncrono `http_request`, **qualquer log emitido durante a execução** (seja nos handlers, regras de negócio ou camadas de persistência) herda automaticamente o `request_id` correspondente. Não é necessário repassar o identificador manualmente entre funções.

---

## 4. Formatos de Log e Configuração em Tempo de Execução

A saída do subscriber de logs é configurável via variáveis de ambiente, sem necessidade de recompilação do binário:

### 4.1 Variável `LOG_FORMAT`

- `LOG_FORMAT=json`: Saída estruturada em JSON (uma linha por evento). Cada linha contém `timestamp` (ISO-8601 UTC), `level`, `target`, `message`, `request_id` destacado e a cadeia de spans ativos (`spans`). Indicado para contêineres e produção.
- `LOG_FORMAT=compact`: Saída condensada em linha única, priorizando o nível e a mensagem sem detalhes excessivos.
- `LOG_FORMAT=pretty`: Saída identada e com cores ANSI, facilitando a leitura humana durante o desenvolvimento interativo local.
- `LOG_FORMAT=full` (padrão): Saída tradicional detalhada do `tracing-subscriber`.

> Em ambientes onde `ENVIRONMENT=production` ou `APP_ENV=production` e `LOG_FORMAT` não for explicitamente informada, o formato `json` é adotado automaticamente.

### 4.2 Variável `RUST_LOG`
Controla os filtros de nível de severidade via `tracing_subscriber::EnvFilter`. Se omitida ou inválida, adota `info` como padrão. Exemplos:
- `RUST_LOG=info`
- `RUST_LOG=info,backend::api=debug`
- `RUST_LOG=warn,backend=trace`

---

## 5. Política de Proteção e Sanitização de Dados Sensíveis

Para garantir conformidade com as diretrizes de governança e proteção à privacidade:

1. **Mascaramento Preventivo de Chaves Sensíveis:**
   Campos com nomes sensíveis (tais como `authorization`, `proxy-authorization`, `token`, `access_token`, `refresh_token`, `secret`, `client_secret`, `password`, `api_key`, `cookie`, `set-cookie`) são automaticamente mascarados com o valor `[REDACTED]`.
2. **Higienização de Tokens Bearer:**
   Valores em texto que contenham prefixos de autenticação (como `Bearer <token>`) são automaticamente higienizados para `Bearer [REDACTED]`.
3. **Ausência de Corpos e Cabeçalhos Críticos em Logs HTTP:**
   O middleware `TraceLayer` registra exclusivamente método, URI, status e latência. Corpos de requisições e cabeçalhos de autorização não são incluídos nos eventos de telemetria padrão.
4. **Invariante Institucional:**
   É estritamente proibida a emissão em logs de senhas em claro, segredos de infraestrutura ou dados que violem o princípio do menor privilégio.

---

## 6. Verificação Técnica e Testes Automatizados

A eficácia dos componentes de observabilidade é garantida por meio de testes automatizados na esteira de integração contínua:

- **Testes Unitários (`backend/src/api/logging.rs` e `backend/src/api/request_id.rs`):**
  - Validação sintática de identificadores e limites de tamanho;
  - Preservação de identificadores válidos e geração determinística de UUID v4 para casos inválidos;
  - Mascaramento de dados e chaves sensíveis;
  - Serialização e formato dos campos JSON.
- **Testes de Integração (`backend/tests/observability_tests.rs`):**
  - Confirmação de injeção do cabeçalho `x-request-id` em respostas HTTP 200 e 404;
  - Propagação correta de IDs enviados pelo cliente;
  - Substituição de IDs com espaços, caracteres de controle ou comprimento excessivo;
  - Verificação de herança do `request_id` nos logs estruturados gerados durante o processamento.
