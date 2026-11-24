# HTTPS no CApp

O app se conecta diretamente ao servidor Rust/Axum por HTTPS. O Rustls
negocia TLS 1.3 ou 1.2 na porta 3000 do container. O Compose publica essa
porta como 8443 no host e mantém o PostgreSQL na rede interna. O backend
continua exigindo Firebase Auth e App Check nas rotas protegidas.

## Gerar o certificado e iniciar o backend

É necessário ter Python 3, OpenSSL 3, Docker Compose com BuildKit e um
projeto Firebase configurado. Execute a partir da raiz do repositório, em
Bash no Linux ou WSL.

```bash
python deploy/https/generate_certificate.py
[ -f .env ] || cp .env.example .env
export BACKEND_UID="$(id -u)" BACKEND_GID="$(id -g)"
```

Edite `.env` com os dados do projeto Firebase e do banco. Se já usava esse
arquivo, confira se `BACKEND_PORT=8443` e remova as opções do antigo perfil
HTTPS. Os caminhos padrão são `.tls/backend.crt` para o certificado público
e `.tls/backend.key` para a chave privada. O binário exige ambos os caminhos,
e o Compose monta os arquivos em modo somente leitura. O UID e o GID devem
permitir que o processo sem privilégios leia a chave de modo `0600`.

O certificado autoassinado vale aproximadamente dez anos e inclui
`localhost`, `127.0.0.1`, `::1` e `10.0.2.2` no SAN. Para usar o IP da
rede local ou um nome DNS, inclua cada endereço pelo qual o app realmente
vai acessar o backend. Repita `--ip` ou `--dns` para incluir vários.

```bash
python deploy/https/generate_certificate.py \
  --ip 192.168.1.20 --dns capp.lan
```

Esses são comandos alternativos de geração inicial. Se já houver arquivos,
use `--out-dir` para gerar outro par sem sobrescrever a chave anterior.
A chave TLS fica no servidor, fora do Git, da imagem e do app. A demonstração
eleitoral usa sua própria chave em `ELECTION_SIGNING_KEY_FILE`.

```bash
docker compose --profile backend up -d --build --wait
curl --fail --cacert .tls/backend.crt https://localhost:8443/ready
```

O healthcheck usa o certificado público e confere o nome
`BACKEND_TLS_HEALTH_HOST` (padrão `localhost`) ao consultar `/ready` no
container. Consulte `docker compose logs backend` se a inicialização falhar.
Para inspecionar o PostgreSQL, use `docker compose exec postgres psql -U capp -d capp`
com o usuário e o banco do seu ambiente.

## Executar no Android

Configure Firebase Auth, App Check e o registro Android do MSAL conforme
o [README do mobile](../../mobile/README.md). Na raiz do repositório, execute
`flutter devices` para conferir o identificador do aparelho ou emulador.

- **Emulador Android padrão**. Use `https://10.0.2.2:8443`, que aponta para o host.
- **Dispositivo físico via USB**. Com a depuração USB autorizada, execute
  `adb reverse tcp:8443 tcp:8443` e use `https://127.0.0.1:8443`.
- **Dispositivo físico via Wi-Fi**. Inclua o IP ou DNS da máquina do servidor
  no SAN. Defina `CAPP_HTTPS_BIND` em `.env` para o IP dessa interface ou
  `0.0.0.0`, disponibilize TCP 8443 na rede local e recrie o serviço.
  Use o IP ou DNS correspondente em `BACKEND_BASE_URL`.

```bash
cd mobile
flutter pub get
public_cert="$(base64 < ../.tls/backend.crt | tr -d '\r\n')"
flutter run -d '<id-do-android>' \
  --dart-define=BACKEND_BASE_URL='https://10.0.2.2:8443' \
  --dart-define="BACKEND_TLS_TRUST_PEM_BASE64=$public_cert" \
  --dart-define=MSAL_CLIENT_ID='<application-client-id>' \
  --dart-define=MSAL_ANDROID_REDIRECT_URI='<redirect-uri-do-registro-android>'
```

Em PowerShell no diretório `mobile`, obtenha o valor equivalente com
`[Convert]::ToBase64String([IO.File]::ReadAllBytes((Resolve-Path '../.tls/backend.crt').Path))`.
Passe apenas o certificado público pelo `--dart-define`. O app confere a
cadeia de confiança, a validade e o DNS ou IP da URL no handshake. A confiança
adicional vale para o transporte do backend Android.

## Verificar HTTPS e trocar o certificado

Com o backend em execução, instale `cryptography` em um ambiente virtual
Python e execute a partir de `mobile`.

```bash
python tool/test_backend_tls.py \
  --backend-cert ../.tls/backend.crt --backend-url https://localhost:8443
```

Acrescente `--device emulator-5554` (ou o ID do seu emulador) e use
`--backend-url https://10.0.2.2:8443` para rodar a mesma suíte no Android.
A suíte verifica o backend real e usa servidores locais para testar
certificados desconhecidos, expirados, incompatíveis com o endereço e
redirecionamentos para HTTP.

A substituição do par TLS é manual. Gere um novo par, ajuste os caminhos e
recrie o container. Recompile o app com o novo certificado antes de usá-lo
com o servidor. Para um certificado de CA pública, configure a cadeia PEM
do servidor em `BACKEND_TLS_CERT_FILE`, a chave correspondente e um arquivo
de âncoras confiáveis para o healthcheck em `BACKEND_TLS_HEALTH_CA_FILE`.
Aponte `BACKEND_TLS_HEALTH_HOST` e a URL do app para o DNS do certificado.
Nesse caso, omita `BACKEND_TLS_TRUST_PEM_BASE64` para usar as raízes da
plataforma.
