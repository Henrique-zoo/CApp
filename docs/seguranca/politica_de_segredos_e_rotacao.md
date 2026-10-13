# Política de Proteção, Governança e Rotação de Segredos — CApp

## 1. Visão Geral e Objetivo

Este documento estabelece as diretrizes de governança, proteção, segregação de ambientes e procedimentos operacionais padrão (SOP) de rotação, revogação e recuperação para segredos e credenciais utilizados na plataforma **CApp** (backend e aplicativo móvel).

O objetivo é mitigar riscos de vazamento, uso indevido de credenciais e exposição de dados acadêmicos, atendendo aos requisitos da **Issue #255** vinculada à entrega incremental **I01 ("Acesso e meu CA")**.

### Princípios Norteadores
- **Menor Privilégio (Least Privilege):** Cada serviço, componente e usuário deve possuir apenas os privilégios estritamente necessários para desempenhar sua função.
- **Defesa em Profundidade (Defense in Depth):** Múltiplas camadas de proteção (segregação de rede, isolamento de IAM, mascaramento em logs e validação em tempo de compilação/inicialização).
- **Zero Segredos no Código (Zero Hardcoded Secrets):** Nenhum segredo ou credencial privada pode ser comitado no repositório de código ou embarcado em artefatos clientes.
- **Diagnóstico Seguro (Fail-Safe Diagnostics):** Falhas decorrentes de variáveis ausentes ou inválidas devem interromper o processo imediatamente (*fail-fast*) sem nunca registrar ou ecoar o valor de credenciais em mensagens de erro ou logs.
- **Isolamento Estrito entre Ambientes:** As credenciais de desenvolvimento, homologação e produção são estritamente independentes; o reaproveitamento de credenciais entre ambientes é terminantemente proibido.

---

## 2. Inventário e Classificação de Credenciais

A tabela a seguir consolida todas as configurações e credenciais do ecossistema CApp, categorizando-as por finalidade, sensibilidade e política de embarque:

| Componente | Variável / Arquivo | Finalidade | Sensibilidade | Onde Reside | Pode Ser Embarcado no App Mobile? |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Backend** | `DATABASE_URL` / `POSTGRES_*` | Conexão autenticada ao PostgreSQL | **Crítica (Segredo Privado)** | Variáveis de runtime / Secrets Manager | **NUNCA** |
| **Backend** | `FIREBASE_SERVICE_ACCOUNT_KEY` | Autenticação administrativa no Firebase Admin SDK | **Crítica (Segredo Privado)** | Secrets Manager / Arquivo restrito (chmod 600) | **NUNCA** |
| **Backend** | `JWT_SECRET` | Chave de assinatura e verificação de tokens de sessão | **Crítica (Segredo Privado)** | Secrets Manager / Variável de runtime | **NUNCA** |
| **Backend** | `MICROSOFT_CLIENT_SECRET` | Segredo OAuth para autenticação institucional UnB | **Crítica (Segredo Privado)** | Secrets Manager / Variável de runtime | **NUNCA** |
| **Backend** | `SERVER_HOST` / `BACKEND_PORT` | Endereço e porta do listener HTTP | **Pública (Configuração)** | Variável de runtime / Compose | Não aplicável |
| **Backend** | `RUST_LOG` / `APP_ENV` | Nível de telemetria e identificador de ambiente | **Pública (Configuração)** | Variável de runtime / Compose | Não aplicável |
| **Mobile** | `google-services.json` | Identificadores do projeto Firebase para Android | **Pública (Configuração de Cliente)** | `mobile/android/app/` | **Sim (Público)** |
| **Mobile** | `GoogleService-Info.plist` | Identificadores do projeto Firebase para iOS | **Pública (Configuração de Cliente)** | `mobile/ios/Runner/` | **Sim (Público)** |
| **Mobile** | `firebase_options.dart` | Configurações do Firebase SDK (Dart) | **Pública (Configuração de Cliente)** | `mobile/lib/` | **Sim (Público)** |
| **Mobile** | `API_BASE_URL` | Endpoint da API backend CApp | **Pública (Configuração de Cliente)** | Variável de build / Arquivo de ambiente cliente | **Sim (Público)** |
| **CI/CD** | `GITHUB_TOKEN` / Deploy Keys | Automação de pipelines e publicação de imagens | **Crítica (Segredo de Automação)** | GitHub Actions Encrypted Secrets | **NUNCA** |

---

## 3. Distinção Estrita: Configurações Públicas do Mobile vs. Segredos Privados do Backend

Existe uma distinção fundamental de arquitetura entre identificadores de cliente e segredos privados:

### 3.1. Configurações Públicas do Cliente Mobile
Os arquivos `google-services.json`, `GoogleService-Info.plist` e `firebase_options.dart` contêm parâmetros como `apiKey`, `appId`, `projectId`, `messagingSenderId` e `storageBucket`.
- **Natureza:** No ecossistema do Google Firebase, essas chaves de API para clientes móveis e web funcionam exclusivamente como **identificadores de roteamento do projeto**, e **não** como chaves de autorização de segurança.
- **Garantia de Segurança:** O acesso a dados no Firebase não depende do sigilo dessas chaves, mas sim de:
  1. Validação de tokens pelo backend CApp antes de executar ações de domínio;
  2. Regras de segurança (Security Rules) do Cloud Storage e Firestore (quando utilizados);
  3. Restrições de chave de API configuradas no Google Cloud Console (restringindo o uso ao *package name* `br.com.capp` e assinaturas SHA-1).

### 3.2. Segredos Privados do Backend (Chaves que NUNCA Podem Ser Embarcadas)
Segredos como chaves de conta de serviço (`service_account.json`), strings de conexão a banco de dados (`DATABASE_URL`), chaves de assinatura HMAC/RSA (`JWT_SECRET`) e client secrets de provedores OAuth **possuem autoridade administrativa plena**:
- Se uma chave de service account for embarcada no APK ou IPA, qualquer usuário pode extraí-la via descompilação (por exemplo, usando `apktool` ou `jadx`) e obter controle administrativo sobre o projeto na nuvem.
- **Invariante Rígida:** O aplicativo móvel comunica-se exclusivamente com a API REST do backend CApp. O backend é o único componente autorizado a possuir e utilizar credenciais privadas de banco de dados, provedores de identidade e APIs administrativas.

---

## 4. Segregação de Credenciais por Ambiente

As credenciais e recursos são divididos em três ambientes completamente estanques:

```
┌─────────────────────────────────┐      ┌─────────────────────────────────┐      ┌─────────────────────────────────┐
│     Desenvolvimento (dev)       │      │       Homologação (staging)     │      │        Produção (prod)          │
├─────────────────────────────────┤      ├─────────────────────────────────┤      ├─────────────────────────────────┤
│ • PostgreSQL local / Compose    │      │ • Banco gerenciado isolado      │      │ • Banco dedicado com réplica    │
│ • Credenciais locais didáticas  │      │ • Credenciais aleatórias dinâm. │      │ • TLS obrigatório (verify-full) │
│ • Firebase dev/emulador         │      │ • Projeto Firebase: capp-stg    │      │ • Projeto Firebase: capp-prod   │
│ • Sem acesso a dados reais      │      │ • Ensaio obrigatório de rotação │      │ • Usuário com menor privilégio  │
└─────────────────────────────────┘      └─────────────────────────────────┘      └─────────────────────────────────┘
```

### 4.1. Desenvolvimento Local (`development`)
- Destinado ao fluxo diário de engenharia na máquina do desenvolvedor.
- Utiliza valores padronizados de desenvolvimento (`postgres://capp:capp@localhost:5432/capp` ou `postgres:5432` no Compose).
- Template documentado em `.env.example` e `backend/.env.example`.
- Dados didáticos e fictícios (respeitando a invariante de e-mails `@aluno.unb.br`).
- Nenhuma conexão permitida a bancos de homologação ou produção.

### 4.2. Homologação (`staging`)
- Espelho fiel da infraestrutura de produção, porém com dados sintéticos e mascarados.
- Instância dedicada de banco de dados e projeto próprio no Firebase (`capp-staging`).
- Credenciais geradas com alta entropia gerenciadas via cofre seguro (ex.: GitHub Secrets / AWS Secrets Manager / GCP Secret Manager).
- **Ambiente oficial de ensaio de rotação:** Toda alteração de procedimento operacional de credenciais deve ser ensaiada previamente em homologação, comprovando a invalidação da credencial anterior.

### 4.3. Produção (`production`)
- Ambiente de atendimento aos estudantes e Centros Acadêmicos da UnB.
- Conexão com o banco de dados obrigatoriamente criptografada com TLS/SSL (`sslmode=verify-full`).
- Credenciais de banco com alta entropia (mínimo de 32 caracteres pseudoaleatórios seguros).
- **Princípio do Menor Privilégio no PostgreSQL:**
  - O usuário utilizado pela aplicação em tempo de execução (`capp_app`) possui permissões restritas a DML (`SELECT`, `INSERT`, `UPDATE`, `DELETE`) nas tabelas necessárias.
  - O usuário não possui privilégios de `SUPERUSER`, `CREATEDB` ou `CREATEROLE`.
  - Migrações são executadas em etapas controladas ou através de credencial administrativa dedicada de migração.
- Projeto Firebase de produção (`capp-prod`) com credencial de conta de serviço restrita às permissões estritas de autenticação e mensagens (FCM).

---

## 5. Diagnóstico Seguro e Mascaramento em Logs

O backend CApp implementa controles estruturais para evitar que falhas ou configurações inválidas exponham segredos:

### 5.1. Inicialização Segura (*Fail-Fast*)
Ao iniciar, o processo lê e valida as variáveis através do módulo `backend::config::AppConfig`:
1. Se uma variável obrigatória (como `DATABASE_URL`) não estiver presente ou for vazia, a aplicação aborta imediatamente com erro claro informando o nome da variável faltante:
   ```
   A variável de ambiente obrigatória 'DATABASE_URL' não foi definida
   ```
2. Se o valor contiver formato inválido (por exemplo, protocolo divergente de `postgres://` ou `postgresql://`), a mensagem de erro detalha a restrição técnica sem jamais imprimir a URL fornecida:
   ```
   A variável de ambiente 'DATABASE_URL' contém formato inválido: deve iniciar com o esquema 'postgres://' ou 'postgresql://'
   ```
3. O valor da variável nunca é ecoado em `panic`, `std::fmt::Display` de erro ou em rastros de pilha (`stacktrace`).

### 5.2. Envoltório `SecretString`
Para prevenir vazamento acidental em instruções de log (`tracing::info!`, `tracing::debug!`):
- Todos os segredos e credenciais em memória são encapsulados no tipo `SecretString`.
- As implementações das traits `std::fmt::Debug` e `std::fmt::Display` de `SecretString` geram invariavelmente `[REDACTED]`.
- O acesso ao conteúdo puro do segredo exige invocação explícita de `SecretString::expose_secret()`, que deve ser restrita a chamadas diretas de conectores criptografados (ex.: pool SQLx).

### 5.3. Higiene em Pipelines de CI/CD
- Nenhum pipeline deve executar `printenv` ou `env` sem filtros.
- Variáveis marcadas como secret no GitHub Actions são automaticamente mascaradas pela plataforma caso apareçam na saída padrão.

---

## 6. Procedimentos Operacionais Padrão (SOP) de Rotação, Revogação e Recuperação

### 6.1. Procedimento 1: Rotação de Credenciais do PostgreSQL (`DATABASE_URL`)

#### A. Rotação Programada (Zero-Downtime via Dual-User)
Para alternar a senha ou o usuário do banco sem indisponibilidade da API:

1. **Criação da Credencial Sucessora no Banco:**
   Conecte-se como administrador do PostgreSQL e crie o usuário sucessor com as mesmas permissões:
   ```sql
   -- Gerar senha segura com 32+ caracteres
   CREATE USER capp_app_v2 WITH PASSWORD '<NOVA_SENHA_SEGURA_ALEATORIA>';
   GRANT capp_app_role TO capp_app_v2;
   GRANT CONNECT ON DATABASE capp TO capp_app_v2;
   ```
2. **Atualização da Configuração do Backend:**
   Atualize o valor de `DATABASE_URL` no cofre de segredos / variáveis do orquestrador apontando para o usuário `capp_app_v2`:
   ```bash
   DATABASE_URL="postgres://capp_app_v2:<NOVA_SENHA_SEGURA_ALEATORIA>@<HOST_DB>:5432/capp?sslmode=verify-full"
   ```
3. **Reinicialização Controlada (Rolling Update):**
   Dispare o redeploy das réplicas do backend. Verifique a prontidão através do endpoint `/ready`:
   ```bash
   curl -f http://<BACKEND_HOST>:3000/ready
   ```
4. **Verificação da Desativação da Credencial Anterior:**
   Monitore as conexões ativas no PostgreSQL para garantir que o usuário anterior (`capp_app_v1`) não possui mais sessões ativas:
   ```sql
   SELECT pid, usename, client_addr, state 
   FROM pg_stat_activity 
   WHERE usename = 'capp_app_v1';
   ```
5. **Revogação e Exclusão da Credencial Antiga:**
   Após a estabilização da nova versão e confirmação de zero conexões antigas:
   ```sql
   REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM capp_app_v1;
   REVOKE CONNECT ON DATABASE capp FROM capp_app_v1;
   DROP USER capp_app_v1;
   ```

#### B. Rotação de Emergência (Em Caso de Comprometimento de Senha)
Caso a senha do banco tenha sido exposta:

1. **Notificação e Contenção:** Declarar incidente de segurança e acionar a equipe de infraestrutura.
2. **Alteração Imediata no Banco:**
   ```sql
   ALTER USER capp_app WITH PASSWORD '<NOVA_SENHA_EMERGENCIAL_ALEATORIA>';
   ```
3. **Derrubada Forçada de Sessões Conectadas:**
   Encerre imediatamente todas as sessões ativas suspeitas:
   ```sql
   SELECT pg_terminate_backend(pid) 
   FROM pg_stat_activity 
   WHERE usename = 'capp_app' AND pid <> pg_backend_pid();
   ```
4. **Atualização no Backend:**
   Atualize `DATABASE_URL` no orquestrador e reinicie imediatamente todos os contêineres do backend.
5. **Auditoria:** Analise os logs do PostgreSQL para mapear comandos executados no período entre a exposição e a revogação.

---

### 6.2. Procedimento 2: Rotação de Conta de Serviço do Firebase (`FIREBASE_SERVICE_ACCOUNT_KEY`)

#### A. Rotação Programada
1. **Emissão de Nova Chave Privada:**
   - Acesse o Console Google Cloud -> **IAM e administração** -> **Contas de serviço**.
   - Selecione a conta de serviço do backend CApp.
   - Na aba **Chaves**, clique em **Adicionar chave** -> **Criar nova chave** -> formato **JSON**.
   - Salve o JSON de forma segura em ambiente restrito.
2. **Atualização do Segredo no Backend:**
   - Atualize o segredo `FIREBASE_SERVICE_ACCOUNT_KEY` no cofre de segredos do ambiente de execução.
3. **Reinicialização e Validação:**
   - Reinicie as instâncias da API backend.
   - Execute um teste de validação de token ou emissão de credencial para comprovar a aceitação da nova chave.
4. **Revogação da Chave Antiga:**
   - No console do Google Cloud, na aba **Chaves**, localize a chave privada anterior (identificada pelo Key ID antigo) e selecione **Excluir**.
   - A revogação no IAM do Google Cloud tem efeito imediato.

#### B. Revogação de Emergência
1. Acesse imediatamente o Google Cloud Console e **exclua** a chave comprometida. Isso encerra instantaneamente a capacidade de gerar ou validar asserções com aquela chave privada.
2. Emita uma nova chave para o backend e atualize a configuração do serviço.
3. Consulte o **Cloud Audit Logs** com filtro para a conta de serviço afetada, identificando se houve chamadas não autorizadas.

---

### 6.3. Procedimento 3: Rotação de JWT e Tokens de Sessão (`JWT_SECRET`)

#### A. Rotação Programada (Com Transição Suave)
1. **Suporte a Múltiplas Chaves Ativas (`kid`):**
   - O validador do backend é configurado com a chave atual e a nova chave.
   - O emissor de tokens passa a assinar novos JWTs utilizando a nova chave.
   - A chave anterior permanece aceita para validação durante o tempo de vida máximo de um token existente (TTL, por exemplo, 24 horas).
2. **Depreciação da Chave Antiga:**
   - Decorrido o período de TTL de todos os tokens anteriores, remova a chave antiga da lista de chaves de validação aceitas.

#### B. Revogação de Emergência (Em Caso de Vazamento da Chave de Assinatura)
1. Substitua imediatamente o valor de `JWT_SECRET` por uma nova sequência aleatória com pelo menos 32 bytes (256 bits) de entropia.
2. Reinicie as instâncias do backend.
3. **Efeito:** Todos os tokens de sessão emitidos anteriormente se tornam instantaneamente inválidos, exigindo que os usuários realizem novo login através do fluxo institucional. Isso bloqueia qualquer atacante que tenha gerado tokens forjados com a chave vazada.

---

### 6.4. Procedimento 4: Rotação de Credenciais OAuth (Microsoft UnB)

1. No portal Microsoft Entra ID (Azure AD), na tela de registro do aplicativo CApp, gere um novo **Client Secret** com validade definida.
2. Atualize a variável `MICROSOFT_CLIENT_SECRET` no backend CApp.
3. Execute o fluxo de autenticação com e-mail didático `@aluno.unb.br` em homologação para validar a integração.
4. Exclua o Client Secret anterior no portal da Microsoft.

---

## 7. Roteiro Prático de Ensaio de Rotação em Homologação

Para cumprir com o critério de conclusão da Issue #255 (*"Uma rotação foi ensaiada em homologação e a credencial anterior deixou de funcionar"*), este é o roteiro prático e auditável:

```
[Etapa 1] Testar conexão com Credencial A (Sucesso)
    ↓
[Etapa 2] Provisionar Credencial B no PostgreSQL
    ↓
[Etapa 3] Atualizar backend para utilizar Credencial B e verificar /ready (Sucesso)
    ↓
[Etapa 4] Revogar Credencial A no PostgreSQL (DROP USER / REVOKE)
    ↓
[Etapa 5] Teste Negativo: Tentar conexão com Credencial A (FALHA CONFIRMADA)
```

### Comandos de Execução do Ensaio

1. **Estado Inicial:** O backend conecta utilizando a `DATABASE_URL` da Credencial A (`capp_app_v1`). O endpoint `/ready` retorna HTTP 200.
2. **Provisionar Nova Credencial:**
   ```sql
   CREATE USER capp_app_v2 WITH PASSWORD 'senha_homologacao_rotacionada_456';
   GRANT capp_app_role TO capp_app_v2;
   ```
3. **Atualizar Backend e Reiniciar:**
   Atualizar `DATABASE_URL` para apontar para `capp_app_v2` e reiniciar o container do backend.
   Confirmar retorno 200 em `/ready`.
4. **Revogar Credencial Anterior:**
   ```sql
   DROP USER capp_app_v1;
   ```
5. **Verificação Negativa Obrigatória:**
   Executar tentativa de conexão utilizando a credencial antiga:
   ```bash
   psql "postgres://capp_app_v1:senha_antiga_123@localhost:5432/capp" -c "SELECT 1;"
   ```
   **Resultado Esperado:** O comando deve falhar com `FATAL: role "capp_app_v1" does not exist` ou `FATAL: password authentication failed`, comprovando de forma conclusiva que a credencial anterior deixou de funcionar.

---

## 8. Checklist de Conformidade

Antes de aprovar qualquer entrega ou alteração de infraestrutura relacionada a segredos:

- [x] O inventário de segredos está atualizado com componente, sensibilidade e finalidade.
- [x] Credenciais privadas não constam no código-fonte, nos artefatos móveis (APK/IPA) ou nos logs.
- [x] Variáveis de ambiente possuem templates seguros (`.env.example`) sem senhas de produção.
- [x] O backend falha com diagnóstico seguro (*fail-fast*) sem ecoar valores de segredos em mensagens de erro.
- [x] Tipos sensíveis em memória utilizam mascaramento (`SecretString`).
- [x] Procedimentos de rotação programada e de emergência estão documentados para Postgres, Firebase e JWT.
- [x] O ensaio de rotação em homologação foi planejado e inclui o teste de invalidação da credencial anterior.
