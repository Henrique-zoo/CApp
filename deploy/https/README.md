# HTTPS para a demonstração no Android

O emulador acessa o Caddy em `https://10.0.2.2:8443`. O Caddy termina a
conexão TLS e encaminha a requisição para `backend:3000` pela rede do Compose.
O PostgreSQL também fica restrito aos contêineres. O único serviço publicado
no host é o Caddy, por padrão na interface de loopback.

## Subir e conferir a API

Na raiz do repositório, use Docker com Compose v2.24.4 ou posterior. Os
comandos abaixo são para PowerShell. No primeiro build, o Docker baixa as
imagens e compila o backend Rust.

```powershell
$demo = @('-p', 'capp-https', '-f', 'compose.yaml', '-f', 'compose.https.yaml')
docker compose @demo --env-file .env.example up -d --build --wait
docker compose @demo ps
$ca = Join-Path ([IO.Path]::GetTempPath()) 'capp-demo-root.crt'
docker compose @demo cp caddy:/data/caddy/pki/authorities/local/root.crt $ca
```

Confira que os três serviços aparecem como `healthy` e que somente o Caddy
publica uma porta. O teste Android abaixo usa a CA exportada para consultar
`/ready` e espera HTTP **200**. O arquivo `.env.example` permite testar essa
rota pública. Para entrar com
uma conta institucional, faça a configuração Firebase e MSAL do projeto e
suba o Compose com seu `--env-file .env` no lugar do exemplo.

## Conectar o CApp

A CA (autoridade certificadora) local é criada pelo Caddy na primeira
execução. Copie seu **certificado público** diretamente do contêiner como
acima. Para esta compilação de debug, passe o certificado ao cliente do
backend. Ele verifica a cadeia, o prazo de validade e o nome do servidor.
As chaves privadas da CA e do servidor ficam no volume `caddy_data`.

```powershell
$caBase64 = [Convert]::ToBase64String([IO.File]::ReadAllBytes($ca))
Push-Location mobile
flutter run --debug --dart-define=BACKEND_BASE_URL=https://10.0.2.2:8443 "--dart-define=CAPP_DEMO_CA_PEM_BASE64=$caBase64" "--dart-define=MSAL_CLIENT_ID=$env:MSAL_CLIENT_ID" "--dart-define=MSAL_ANDROID_REDIRECT_URI=$env:MSAL_ANDROID_REDIRECT_URI"
Pop-Location
```

Preencha as duas variáveis MSAL com os valores públicos do seu registro
Android antes de executar. Para usar um celular físico na mesma rede, defina
`CAPP_TLS_HOST` e `CAPP_HTTPS_BIND` com o IP de rede local do computador antes
do `up` e use esse mesmo IP na URL do aplicativo. O passo a passo do login
está no [README do mobile](../../mobile/README.md#configuração-e-execução-android).

## Repetir os testes de certificado

Com o emulador Android ligado e o Compose acima pronto, rode a partir de
`mobile`. Use o identificador exibido por `flutter devices`.

```powershell
python -m pip install cryptography
flutter pub get --enforce-lockfile
python tool/test_backend_tls.py --device emulator-5554 --caddy-ca "$ca" --caddy-url https://10.0.2.2:8443
```

O teste usa certificados temporários para verificar que o cliente aceita o
certificado correto e rejeita CA desconhecida, certificado expirado e nome
incorreto. A última consulta verifica o Caddy e o backend reais. A CI também
confere que a API e o banco não têm portas publicadas e que a porta 3000
não responde ao host. Ao terminar a demonstração, volte à raiz e execute
`docker compose @demo down`. Os volumes preservam o banco e a CA para a
próxima execução.

## Domínio público

Com um domínio que a equipe controle, configure o DNS do nome para o servidor
e permita conexões externas na porta TCP 443. No ambiente do Compose, defina
`CAPP_TLS_MODE=public`, `CAPP_TLS_HOST=<domínio>`, `CAPP_HTTPS_BIND=0.0.0.0` e
`CAPP_HTTPS_PORT=443`. Use o arquivo de ambiente da implantação e mantenha
o volume de dados do Caddy entre reinícios.

O Caddy usa ACME com o desafio TLS-ALPN-01 na porta 443 para emitir e
renovar o certificado do domínio. No aplicativo, configure
`BACKEND_BASE_URL=https://<domínio>` e use as autoridades de confiança do
dispositivo. [Documentação de HTTPS automático do Caddy](https://caddyserver.com/docs/automatic-https)
