# Firebase Authentication

## Responsabilidades

O adaptador valida Firebase ID tokens utilizados no fluxo de identidade
institucional. Ele verifica a assinatura, o algoritmo e as claims esperadas para
o projeto, exigindo que o provedor declarado no token seja `microsoft.com`.

O adaptador não provisiona perfis, não associa usuários a Centros Acadêmicos e
não decide permissões de negócio.

## Configuração local

O backend utiliza a variável `FIREBASE_PROJECT_ID`, correspondente ao ID do
projeto Firebase configurado para a aplicação. O identificador confirmado em
`mobile/lib/firebase_options.dart` é `capp-a331b`.

Para iniciar o PostgreSQL local, execute na raiz do monorepo:
`docker compose up -d --wait postgres`

Para executar o backend com Cargo, forneça as variáveis de ambiente
`DATABASE_URL`, `FIREBASE_PROJECT_ID` e `RUST_LOG`. O backend não carrega
automaticamente arquivos `.env` quando iniciado pelo Cargo.

Exemplo de configuração local:
`DATABASE_URL='postgres://capp:capp@localhost:5432/capp' FIREBASE_PROJECT_ID='capp-a331b' RUST_LOG=info cargo run --locked`

O `compose.yaml` repassa `FIREBASE_PROJECT_ID` ao contêiner do backend.
O valor padrão local é `capp-a331b`, correspondente ao projeto configurado no
aplicativo. Para usar outro projeto, defina `FIREBASE_PROJECT_ID` no ambiente
ou no arquivo `.env` da raiz. O ID precisa corresponder ao projeto que emitiu
o Firebase ID token; caso contrário, a validação de audiência e emissor falhará.

Os templates `.env.example` são mantidos separadamente na configuração do
projeto. Se você copiar um template com um ID de desenvolvimento sintético para
`.env`, substitua-o pelo ID do projeto usado pelo aplicativo durante os testes
de integração.

## Restrição ao tenant institucional

O handler `GET /identity/me` rejeita identidades cujo e-mail não corresponda
ao domínio discente `@aluno.unb.br`, incluindo e-mail ausente, matrícula fora do
formato de nove dígitos e a matrícula explicitamente proibida pela política do
projeto.

A claim `firebase.sign_in_provider == "microsoft.com"` identifica o provedor
de login, mas nem essa claim nem o domínio do e-mail comprovam, isoladamente,
que a identidade pertence ao tenant institucional da UnB.

O aplicativo envia o parâmetro `tenant` ao iniciar o login Microsoft. Isso não
substitui a configuração e a validação da política de contas permitidas no
Microsoft Entra ID e no Firebase Authentication.

A restrição efetiva ao tenant ainda precisa ser confirmada por alguém com
acesso administrativo à configuração do provedor. Até lá, não se deve considerar
comprovado que contas de outros tenants estão impedidas de entrar.

## Verificações e testes

Os testes do adaptador cobrem assinatura válida, assinatura adulterada,
expiração, emissor, audiência, provedor, identificador de usuário e configuração
do ID do projeto.

O adaptador não registra o token recebido nos logs.
