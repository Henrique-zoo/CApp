# Demonstração da eleição segura no CApp

O roteiro usa duas chapas, três eleitores e quatro autoridades criptográficas.
O backend representa uma autoridade. A comissão e cada chapa têm um
representante designado para esta eleição. As designações de demonstração são
cadastradas no banco com contas CApp reais de teste antes da cerimônia.

As assinaturas e referências do manifesto usam os domínios do
[perfil eleitoral](../../packages/api-contracts/elections/profile.json).
Se a demonstração anterior usou o prefixo genérico, crie outra eleição e
publique um novo manifesto antes de distribuir o hash aos participantes.

## Ambiente

Use um banco novo para testar esta branch. O esquema eleitoral foi consolidado
e o SQLx rejeita bancos que já tenham aplicado as versões eleitorais anteriores.

1. Configure Firebase Auth e App Check no projeto de teste e gere a configuração
   Android com FlutterFire. Provisionar as contas no app é parte da preparação.
2. Prepare três contas CApp com login próprio. Configure o CA de teste,
   sua instituição e os cursos em `academic_center_course`. Cada eleitor
   precisa de vínculo institucional vigente em um desses cursos. Guarde os
   e-mails e as senhas do ensaio em um roteiro local.
3. Suba o backend com o [guia HTTPS](../https/README.md), o PostgreSQL e a
   chave RSA privada do servidor em `ELECTION_SIGNING_KEY_FILE`. O cadastro, a
   eleição e as requisições do app usam a mesma instalação. Mantenha a hora dos
   dispositivos e do servidor sincronizadas.

   Execute da raiz, depois de configurar o ambiente e o certificado TLS.
   Gere a chave de assinatura uma vez e use o mesmo arquivo nas próximas
   execuções da instalação.

   ```bash
   mkdir -p .tls
   export ELECTION_SIGNING_KEY_FILE="$PWD/.tls/election-signing.key"
   if [ ! -e "$ELECTION_SIGNING_KEY_FILE" ]; then
     (umask 077; openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "$ELECTION_SIGNING_KEY_FILE")
   fi
   docker compose -f compose.yaml -f compose.elections.yaml --profile backend up -d --build --wait
   ```

4. Reserve um período no futuro, com segundos inteiros e fuso explícito.
   Use UUIDs das contas do CApp e do CA provisionado. Defina os horários
   contando o tempo para cadastrar as três chaves e publicar o manifesto antes
   da abertura.

   ```bash
   docker compose -f compose.yaml -f compose.elections.yaml exec -T postgres sh -c \
     'exec psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" "$@"' sh \
     -v election_id="$ELECTION_ID" \
     -v center_id="$CENTER_ID" \
     -v commission_id="$COMMISSION_ACCOUNT_ID" \
     -v slate_a_account_id="$AURORA_ACCOUNT_ID" \
     -v slate_b_account_id="$HORIZONTE_ACCOUNT_ID" \
     -v starts_at="$STARTS_AT" -v ends_at="$ENDS_AT" \
     < backend/seeds/election_demo.sql
   ```

5. Para um Android físico via USB, autorize a depuração no aparelho e use
   o UUID da eleição em `ELECTION_ID`. O arquivo local `mobile/.env` fornece
   a configuração MSAL e o token de depuração do App Check. Ajuste o caminho
   de leitura do certificado público se o gerou em outro local.

   ```bash
   adb devices -l
   adb reverse tcp:8443 tcp:8443
   public_cert="$(base64 < .tls/backend.crt | tr -d '\r\n')"
   cd mobile
   flutter run --debug -d "$(adb get-serialno)" \
     --dart-define-from-file=.env \
     --dart-define="ELECTION_ID=$ELECTION_ID" \
     --dart-define=BACKEND_BASE_URL=https://127.0.0.1:8443 \
     --dart-define="BACKEND_TLS_TRUST_PEM_BASE64=$public_cert"
   ```

## Percurso

1. Cada representante usa **Entrar com e-mail e senha**, abre **Início →
   Autoridades da eleição** e seleciona **Cadastrar chave deste aparelho**.
   A troca de conta pode ser feita por **Conta → Sair da conta**. Mantenha o
   dispositivo e a instalação de cada autoridade até enviar a contribuição.
2. A comissão seleciona **Publicar manifesto** e copia o hash. Entregue essa
   referência aos demais por um canal independente do download do quadro.
   Cada participante usa **Informar hash do manifesto**. A publicação fixa
   a ordem das chapas, Nulo e as chaves públicas.
3. Durante a votação, três contas elegíveis votam **Aurora**, **Aurora** e
   **Horizonte**. Cada eleitor guarda e confere seu próprio recibo. As
   autoridades podem guardar o checkpoint do quadro enquanto a votação avança.
4. Quando o prazo termina, o backend encerra o quadro e publica sua própria
   contribuição. Em **Auditar apuração**, baixe as evidências e confira o estado
   incompleto antes de uma das outras autoridades contribuir.
5. Após o encerramento, cada representante usa **Enviar contribuição após o
   encerramento**. Quando todas são aceitas, a apuração mostra Aurora 2,
   Horizonte 1 e Nulo 0. Em **Auditar apuração**, baixe a versão final, interrompa
   o acesso ao backend e use **Verificar cópia local**.

Uma segunda cédula da mesma conta é recusada pelo backend. A suíte de
integração exercita essa recusa e a recuperação do recibo após reenvio. O teste
do auditor no app também confere a recusa de uma cópia com cédula adulterada.
Use cópias de evidência para demonstrar adulterações e preserve o pleito
original como referência.

## Verificação e pacote

```bash
cd backend
cargo test --lib --test integration --test bdd_step_registry --features test-support --locked
cd ../mobile
flutter test --no-pub test/src/modules/elections
```

O teste de auditoria do Flutter usa OpenSSL no host para verificar os vetores
públicos com RSA-PSS. O Android usa o verificador do dispositivo. Recibos
independentes comprovam o prefixo que cobrem. A avaliação de completude do
quadro exige uma referência externa ao encerramento.

Da raiz do checkout, com o relatório de até dez páginas revisado pela equipe,
monte o arquivo de entrega:

```bash
python3 deploy/elections/package_demo.py \
  --report /caminho/relatorio.pdf \
  --output /caminho/capp-eleicao-segura.zip
```

O pacote inclui o código e dependências fixadas, os roteiros e os vetores.
Regenere `mobile/lib/firebase_options.dart` e `google-services.json` no
projeto Firebase de teste antes de compilar a cópia extraída. Use as
credenciais e chaves privadas da instalação à parte.
