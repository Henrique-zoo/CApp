# Política de Encerramento Controlado e Recuperação do Backend

Esta especificação técnica define a arquitetura, o ciclo de vida e os procedimentos operacionais para encerramento gracioso (*graceful shutdown*) e recuperação do backend CApp.

---

## 1. Visão Geral e Motivação

Em arquiteturas modernas baseadas em contêineres e orquestradores (como Docker, Kubernetes e systemd), processos de serviço são frequentemente reiniciados, redimensionados ou atualizados por meio de novas implantações (*rolling updates*).

Um encerramento abrupto (como `kill -9` ou término imediato sem drenagem) provoca:
- Interrupção repentina de requisições HTTP em voo, resultando em respostas incompletas ou erros `502 Bad Gateway` / `504 Gateway Timeout` nos clientes;
- Aborto de transações SQL pendentes no PostgreSQL, gerando *rollbacks* inesperados e possíveis inconsistências temporárias de cache ou estado;
- Encerramento não anunciado em balanceadores de carga, que continuam encaminhando novos pacotes para um socket já inexistente.

Para mitigar esses riscos, o backend CApp implementa um protocolo estruturado de encerramento controlado em 3 fases, com suporte portável a sinais de terminação, timeout de drenagem configurável e liberação sequencial de recursos compartilhados.

---

## 2. Tratamento Portável de Sinais de Sistema Operacional

O subsistema em `backend::api::shutdown::wait_for_shutdown_signal` captura de maneira assíncrona os sinais de encerramento emitidos pelo ambiente:

| Plataforma | Sinal / Evento | Origem | Ação no Backend |
|---|---|---|---|
| **Linux / Unix / macOS** | `SIGINT` | Terminal interativo (`Ctrl+C`) | Inicia o protocolo de encerramento gracioso. |
| **Linux / Unix / macOS** | `SIGTERM` | Orquestrador (`docker stop`, Kubernetes Pod eviction, `systemctl stop`) | Inicia o protocolo de encerramento gracioso. |
| **Windows** | `Ctrl+C` / `Ctrl+Break` | Terminal de comando / console | Inicia o protocolo de encerramento gracioso. |
| **Windows** | `Ctrl+Close` | Fechamento da janela do terminal | Inicia o protocolo de encerramento gracioso. |
| **Windows** | `Ctrl+Shutdown` | Desligamento ou reinício do sistema operacional | Inicia o protocolo de encerramento gracioso. |

A captura utiliza a infraestrutura assíncrona do Tokio (`tokio::signal`), sem bloquear threads do runtime e sem recorrer a primitivas inseguras ou rotinas síncronas de baixo nível.

---

## 3. Ciclo de Vida do Encerramento em Três Fases

Quando um sinal é interceptado, o processo de desligamento executa estritamente as três etapas a seguir:

```
                  Sinal Recebido (SIGINT / SIGTERM)
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│ FASE 1: Interrupção de Recepção & Sinalização de Prontidão     │
│ - Fecha o listener TCP para novas conexões externas             │
│ - Marca AppState::is_shutting_down = true                       │
│ - Endpoint GET /ready passa a retornar HTTP 503                 │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│ FASE 2: Drenagem de Conexões Ativas (Drain Period)              │
│ - Aguarda término das requisições em voo                        │
│ - Monitorado por deadline: SHUTDOWN_TIMEOUT_SECS (padrão 15s)   │
│ - Concluído antes do prazo: segue normalmente                   │
│ - Prazo expirado: força término e cancela conexões pendentes    │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│ FASE 3: Liberação Ordenada de Recursos Compartilhados           │
│ - Encerra pool PostgreSQL de forma assíncrona (PgPool::close)   │
│ - Desconecta sockets de banco após conclusão das requisições    │
│ - Registra log de finalização e encerra processo com código 0   │
└─────────────────────────────────────────────────────────────────┘
```

### 3.1. Fase 1: Interrupção de Recepção e Sinalização de Prontidão

1. **Rejeição de novos acessos:** o servidor Axum fecha o socket de recepção (*listener*) e deixa de aceitar novas requisições TCP. Conexões novas são imediatamente recusadas em nível de rede.
2. **Atualização do estado atômico:** a flag `is_shutting_down` mantida no `AppState` (`Arc<AtomicBool>`) é alternada para `true`.
3. **Transição de prontidão (`GET /ready`):** balanceadores de carga (como Ingress NGINX, AWS ALB ou Kubernetes Kubelet) que realizam *readiness probes* recebem imediatamente `HTTP 503 Service Unavailable`, retirando o contêiner dos nós ativos de roteamento.

### 3.2. Fase 2: Drenagem de Requisições Ativas (*Drain Period*)

1. **Atendimento prioritário:** requisições que já estavam em trânsito no momento da recepção do sinal continuam sua execução normal, acessando os serviços de aplicação e o banco de dados.
2. **Deadline configurável:** o tempo concedido para a drenagem é delimitado pela variável de ambiente `SHUTDOWN_TIMEOUT_SECS` (padrão: 15 segundos; faixa permitida: 1 a 300 segundos).
3. **Encerramento forçado por expiração:** se alguma requisição for excessivamente longa, entrar em travamento (*hang*) ou exceder o prazo limite:
   - O cronômetro de timeout expira;
   - Um alerta estruturado no nível `warn` é emitido em `tracing` informando a expiração do deadline;
   - O futuro do servidor HTTP é abortado, liberando as conexões presas para não impedir a parada do processo.

### 3.3. Fase 3: Liberação Ordenada de Recursos Compartilhados

1. **Desconexão do banco:** após o encerramento do servidor HTTP (seja por drenagem completa ou por expiração de timeout), o pool compartilhado PostgreSQL (`sqlx::PgPool`) é fechado por meio de `pool.close().await`.
2. **Preservação de transações:** o pool só é fechado **depois** que as requisições em trânsito concluíram sua execução, garantindo que operações de escrita em andamento consigam efetuar seus *commits* sem interrupção abrupta da camada de conexão.
3. **Encerramento limpo:** todos os recursos são desalocados e o processo finaliza com código de saída 0.

---

## 4. Variáveis de Ambiente e Configuração

As configurações de ciclo de vida são centralizadas no módulo `backend::config::ServerConfig`:

| Variável | Tipo | Padrão | Descrição |
|---|---|---|---|
| `SHUTDOWN_TIMEOUT_SECS` | Inteiro (`1..=300`) | `15` | Prazo máximo em segundos para drenagem de requisições antes do encerramento forçado. |
| `BACKEND_BIND_ADDRESS` | String | `0.0.0.0:3000` | Endereço IP e porta para escuta do servidor HTTP. |
| `DATABASE_URL` | String (URL) | *Obrigatória* | Cadeia de conexão aceita pelo SQLx para o PostgreSQL. |
| `RUST_LOG` | String | `info` | Filtro de eventos do subsistema de observabilidade `tracing`. |

Valores inválidos (não numéricos, zero ou superiores a 300 segundos) causam falha imediata na inicialização (*fail-fast*), impedindo a subida do servidor com parâmetros inconsistentes.

---

## 5. Alinhamento com Orquestradores (Kubernetes e Docker)

### 5.1. Relação entre Prazos de Encerramento

Ao configurar ambientes orquestrados, o prazo de terminação forçada do orquestrador (`terminationGracePeriodSeconds` no Kubernetes ou `--stop-timeout` no Docker) **deve ser superior** ao `SHUTDOWN_TIMEOUT_SECS` do backend.

- **Recomendação:**
  * `SHUTDOWN_TIMEOUT_SECS`: `15` segundos (backend drena e fecha o pool em até 15s).
  * `terminationGracePeriodSeconds`: `30` segundos (o orquestrador concede 30s antes de emitir `SIGKILL`).
  * Essa folga de 15 segundos absorve a propagação das regras de iptables/IPVS e o fechamento do pool de conexões com o PostgreSQL.

```
Tempo (segundos):
0s                   15s                            30s
├── SIGTERM          ├── Prazo drenagem backend    ├── SIGKILL (K8s)
│   (inicia drenagem)│   (fecha pool e sai com 0)  │   (somente se o processo travar)
```

---

## 6. Observabilidade e Diagnóstico Estruturado

Todas as etapas do encerramento emitem registros estruturados via `tracing`:

- **Sinal recebido:**
  ```text
  INFO backend::api::shutdown: Sinal de encerramento recebido. Interrompendo novos aceites e iniciando drenagem das requisições... timeout_secs=15
  ```
- **Prontidão rejeitada durante encerramento:**
  ```text
  WARN backend::api::router: Verificação de prontidão rejeitada: servidor em processo de encerramento.
  ```
- **Drenagem concluída dentro do prazo:**
  ```text
  INFO backend::api::shutdown: Drenagem de requisições HTTP concluída com sucesso dentro do prazo.
  ```
- **Deadline de drenagem expirado:**
  ```text
  WARN backend::api::shutdown: Prazo limite de drenagem de requisições HTTP expirou. Forçando encerramento do servidor. timeout_secs=15
  ```
- **Encerramento de pool de conexões:**
  ```text
  INFO backend::api::shutdown: Fechando pool de conexões com o PostgreSQL de forma ordenada...
  INFO backend::api::shutdown: Pool PostgreSQL fechado com sucesso. Recursos liberados.
  INFO backend: Processo backend finalizado com sucesso.
  ```

---

## 7. Procedimentos de Recuperação Operacional

### 7.1. Falha em Conexão com o Banco Durante o Encerramento
Caso o PostgreSQL fique inacessível enquanto requisições em voo tentam acessá-lo durante a drenagem, os handlers tratam a falha e retornam erro HTTP ao cliente normalmente. Ao expirar o deadline ou concluir as requisições, a tentativa de `pool.close().await` libera a infraestrutura local do pool sem causar *deadlock* no processo.

### 7.2. Processos Travados ou Zumbis
Se o processo backend não finalizar após o prazo esperado:
1. Verifique nos logs de contêiner se houve registro de expiração de deadline (`Prazo limite de drenagem de requisições HTTP expirou`);
2. Caso o processo permaneça em execução por tempo indeterminado após o prazo, o orquestrador aciona o sinal de força maior (`SIGKILL`), garantindo a limpeza do ambiente;
3. Analise no rastreamento se há tarefas assíncronas bloqueantes executando operações de E/S síncronas fora do pool de threads de bloqueio do Tokio (`spawn_blocking`).
