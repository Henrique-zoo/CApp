# Cadastro da chave eleitoral — contrato de requisição v1 revisado

`PUT /elections/{id}/registration/me` exige Firebase ID token e App Check.
A identidade da conta é extraída da autenticação; o identificador da eleição vem da rota.

```json
{
  "payload": { "publicKey": "<Base64url SPKI DER>" },
  "signature": "<Base64url RSA-PSS>"
}
```

A DTO contém exclusivamente chave e assinatura. `publicKey` é RSA SPKI DER,
Base64url sem padding, com perfil RSA 2048/3072/4096 e expoente 65537.
`signature` também é Base64url sem padding. JSON com campos desconhecidos ou
repetidos é rejeitado na desserialização.

O cliente assina os bytes:

`ASCII("CAPP-ELECTIONS/REGISTRATION/1\0") || UTF8(JCS({"accountId":<UUID da sessão>,"electionId":<UUID da rota>,"publicKey":<Base64url>}))`

A assinatura utiliza RSA-PSS/SHA-256, MGF1(SHA-256) e salt length 32.
Os identificadores autenticados não são enviados no corpo, mas são vinculados
pela assinatura. O servidor reconstrói a mesma mensagem e verifica a prova
antes de consultar ou inserir o cadastro. Cadastro dispensado, chave
já cadastrada, prazo e elegibilidade continuam sendo decisões da aplicação,
com base nos dados estabilizados pela transação PostgreSQL.

**Incompatibilidade:** o formato anterior (`kind`, `version`, `accountId`,
`electionId` no JSON de cadastro, e prefixo genérico) não é aceito nesta versão.
Cliente, servidor, vetores e auditor devem atualizar juntos.
