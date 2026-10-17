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

**Atenção:** o `compose.yaml` ainda não repassa `FIREBASE_PROJECT_ID` ao
contêiner do backend. Essa integração precisa ser coordenada com as alterações
de configuração do PR #336 antes de considerar o fluxo operacional via Compose.

## Restrição ao tenant institucional

A claim `firebase.sign_in_provider == "microsoft.com"` identifica o provedor
de login, mas não comprova, isoladamente, que a conta pertence ao tenant
institucional da UnB.

O aplicativo envia o parâmetro `tenant` ao iniciar o login Microsoft. Isso não
substitui a configuração e a validação da política de contas permitidas no
Microsoft Entra ID e no Firebase Authentication.

Essa restrição precisa ser confirmada por alguém com acesso administrativo à
configuração do provedor. Até lá, não se deve considerar comprovado que contas
externas estão impedidas de entrar.

## Verificações e testes

Os testes do adaptador cobrem assinatura válida, assinatura adulterada,
expiração, emissor, audiência, provedor, identificador de usuário e configuração
do ID do projeto.

O adaptador não registra o token recebido nos logs.
