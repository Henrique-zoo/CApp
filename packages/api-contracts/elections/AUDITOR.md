# Auditor independente do quadro eleitoral (#381)

`scripts/audit_election_board.py` lê uma exportação real sem acessar o servidor
nem importar sua implementação Rust. Usa o schema, a canonicalização JCS e os
primitivos públicos do validador Python dos contratos. O corpus `vectors.json`
é usado somente nos testes, nunca para decidir a validade de uma exportação.

## Executar

Requer Python, as dependências de `packages/api-contracts/requirements-ci.txt`
e OpenSSL no `PATH`. Execute da raiz do repositório:

```bash
python packages/api-contracts/scripts/audit_election_board.py \
  --snapshot /dados-exportados/quadro.json \
  --trusted-key /auditoria-independente/chave-servidor.txt \
  --checkpoint /auditoria-independente/eleicao-ancora.json
```

A chave é um arquivo de texto contendo **somente SPKI DER em base64url sem
padding**, previamente confirmado por canal independente. Copiar a chave do
próprio snapshot não estabelece confiança. A cópia `serverSigningKey` no
manifesto deve coincidir com a chave fixada. Não use chave privada no auditor.

Guarde `eleicao-ancora.json` em armazenamento sob controle do observador, fora
do banco, do diretório de exportação e do volume do servidor. Use um arquivo
por eleição. Preserve também cópias
anteriores para inspeção humana. A ferramenta não protege um disco local que
o adversário possa reescrever nem torna confiável uma primeira observação
adversarial.

Na primeira execução válida, o arquivo é criado com o manifesto, calendário,
último recibo e fechamento observado. Nas seguintes, o mesmo caminho fornece
a referência anterior e só é atualizado após validar a exportação inteira e
comparar a continuidade. Um lock exclusivo `<arquivo>.lock` serializa a
comparação e a substituição, recusando um escritor concorrente ou uma versão
anterior que ficou desatualizada. Gravação temporária e substituição atômica
evitam um arquivo parcial; falhas de verificação não alteram a âncora. Se o
processo for interrompido abruptamente, o lock pode permanecer: remova-o
manualmente somente após confirmar que não há auditor usando esse arquivo.
Um arquivo de
outra eleição não é sobrescrito. Não apague a referência para fazer uma
exportação recusada passar.

Um recibo guardado anteriormente pelo eleitor pode ser acrescentado:

```bash
python packages/api-contracts/scripts/audit_election_board.py \
  --snapshot /dados-exportados/quadro.json \
  --trusted-key /auditoria-independente/chave-servidor.txt \
  --checkpoint /auditoria-independente/eleicao-ancora.json \
  --receipt /auditoria-independente/meu-recibo.json
```

O arquivo `--receipt` contém o envelope assinado original do recibo. Sua
assinatura é verificada antes de comparar eleição, manifesto, índice, hash
da entrada e hash da cédula. O recibo enviado junto ao snapshot é verificado,
mas **não conta como evidência prévia independente** nessa execução. Após uma
execução aceita, o último recibo verificado fica retido na âncora local.

`--receipt` também aceita o manifesto assinado que `/board/checkpoint`
devolve quando ainda não há recibo. Nesse caso a evidência ancora apenas a
configuração congelada: `anchoredThroughIndex` vale zero. A assinatura do
manifesto não declara que o quadro estava vazio quando foi consultado.

## Resultado e limites

O resultado é JSON no stdout. Códigos de saída:

| Código | Significado |
| --- | --- |
| `0` | Exportação válida e continuidade com pelo menos uma evidência prévia relevante confirmada. |
| `2` | Arquivo, assinatura, relação, cadeia ou comparação inválida. A âncora não é alterada. |
| `3` | Exportação válida, mas continuidade inconclusiva: não há evidência prévia relevante. Na primeira observação sem evidência de outra eleição, a âncora é criada para comparações futuras. |

`scope: live-prefix` identifica um quadro ainda aberto. O auditor aceita uma
extensão que preserva o prefixo ancorado, inclusive a primeira cédula depois
de um manifesto ancorado vazio. Não exige que duas exportações legítimas
tenham o mesmo tamanho. Um recibo de índice N permite verificar que o prefixo
até N não mudou; não prova a inexistência de votos posteriores omitidos pelo
servidor. Acrescentar votos depois desse prefixo também não é, por si só,
evidência de fraude.

`anchoredThroughIndex` informa N (zero para manifesto e `null` sem referência).

`matchesPublishedClosure: true` significa que as entradas fornecidas
cobrem exatamente a contagem e o hash final declarados no fechamento
assinado, com agregado homomórfico consistente. Não prova frescor global ou
ausência de outra história assinada. `previousClosureVerified: true` indica
adicionalmente que o mesmo fechamento foi comparado com um fechamento
conservado na âncora local anterior. Se um fechamento já foi observado e
guardado, uma exportação aberta ou com outro fechamento é recusada. Sem
âncora relevante, até uma história inteiramente reescrita e re-assinada pode
ser internamente válida: o resultado de continuidade permanece inconclusivo.

São verificados JSON estrito sem nomes duplicados, schemas de envelopes,
JCS/prefixo/hash, RSA-PSS do servidor e das cédulas, manifesto e provas de
chaves das autoridades, opções e elementos do grupo, referências cruzadas,
índices consecutivos, cadeia completa, uma participação por eleitor,
recibo final, calendário, fechamento e produto dos ciphertexts do agregado.
Não há uma chamada remota ou confiança implícita no banco para esses passos.

O escopo não inclui apuração final, shares de decriptação, identidade
Firebase, prova de matrícula ou prova de que cada cédula codifica exatamente
uma opção. Permanecem as premissas explícitas de `profile.json`, inclusive
servidor confiável para o vínculo com o cadastro e ausência de prova pública
individual 0/1. A assinatura da cédula demonstra posse da chave nela declarada;
um export sem cadastro público não demonstra sozinho o vínculo institucional.

`submissionId` identifica a operação no escopo de eleição e eleitor; dois
eleitores distintos podem usar o mesmo valor. A unicidade de participação do
eleitor impede que a repetição de sua operação crie uma segunda entrada.

## Formato de exportação

Todos os objetos abaixo, exceto o wrapper, são envelopes completos
`{"payload": {...}, "signature": "..."}` do schema existente:

```json
{
  "manifest": "envelope manifest",
  "calendarOrigin": "envelope calendar-origin",
  "entries": ["envelopes board-entry em ordem crescente"],
  "checkpoint": "último envelope receipt; null se vazio",
  "closure": "envelope closure; null se aberto",
  "aggregate": "envelope aggregate; null se aberto"
}
```

As strings acima são descrições, não valores wire. `closure` e `aggregate`
devem estar ambos presentes ou ambos nulos. O transporte pode incluir
`scheduleChanges`, uma lista ordenada de envelopes `schedule-change`; sem
esse campo, o calendário vigente é o originário. Prorrogações genuínas
acrescentam ao histórico. O fechamento deve referenciar o último calendário.
O store da #381 atualmente exporta apenas o calendário originário.

A âncora local é um wrapper de evidências existentes, identificado por
`format: capp-board-anchor-v1`; não introduz um novo `kind` de protocolo nem
uma assinatura fictícia de snapshot.

## Testes

```bash
python -m unittest discover -s packages/api-contracts/scripts \
  -p 'test_audit_election_board.py' -v
```

Os testes usam os envelopes RSA-PSS e provas matemáticas dos vetores
independentes do contrato. Cobrem remoção, reordenação, inserção, alteração,
truncamento coerente, recibo inválido, comparação entre eleições, primeira
observação, crescimento de prefixo, fechamento e preservação do arquivo após
falha. Um cenário adicional gera chaves RSA efêmeras e re-assina uma história
reescrita coerente: ela é inconclusiva sem referência e recusada com o recibo
prévio. As chaves efêmeras são removidas ao concluir o teste.
