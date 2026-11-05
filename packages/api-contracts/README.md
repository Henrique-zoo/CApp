# Contratos da API CApp

## Fonte de verdade

O arquivo `openapi.yaml` é a especificação OpenAPI dos endpoints implementados
pelo backend. Alterações em rotas, métodos HTTP, autenticação, DTOs, respostas
ou códigos de erro públicos devem atualizar esse arquivo na mesma alteração
que modifica a implementação.

A especificação atual usa OpenAPI 3.1.0. Os caminhos são servidos sem prefixo
de versão `/v1`. A propriedade `info.version` representa a versão do contrato,
não uma versão de URL nem a versão do pacote Rust do backend.

## Versionamento

A versão do contrato segue Semantic Versioning (`MAJOR.MINOR.PATCH`).

| Tipo | Quando utilizar | Exemplos |
| --- | --- | --- |
| PATCH | Correções que não alteram o comportamento observável do protocolo. | Correção de descrições, exemplos e documentação. |
| MINOR | Adições compatíveis com todos os clientes ainda suportados. | Nova operação ou propriedade opcional cujo recebimento seja seguro para clientes existentes. |
| MAJOR | Mudanças incompatíveis com pelo menos um cliente suportado. | Remover ou renomear rota ou campo, exigir propriedade antes opcional, mudar tipo ou nulabilidade, restringir entrada aceita ou alterar autenticação e semântica de respostas existentes. |

Uma alteração aditiva só é considerada compatível depois de verificar como
os clientes suportados lidam com o novo conteúdo. Novos valores de enumeração
ou novas variantes de uma resposta discriminada, por exemplo, podem quebrar
clientes que não possuem tratamento genérico.

Códigos de erro novos podem ser adicionados em uma versão MINOR quando o
envelope e os status existentes forem preservados e os clientes suportados
tratarem códigos desconhecidos com segurança. Alterar o significado de um
código existente é uma mudança MAJOR.

A versão do contrato deve ser atualizada em `info.version` no mesmo PR da
mudança de protocolo. O OASDiff verifica automaticamente se uma alteração
incompatível exige aumento da versão MAJOR. Uma alteração compatível não exige
aumento MAJOR; a escolha entre MINOR e PATCH deve seguir a tabela acima.

Aumentar a versão MAJOR não torna uma remoção automaticamente compatível com
clientes antigos. Se versões diferentes do aplicativo precisarem coexistir,
o backend deverá preservar as operações antigas durante a migração, por
exemplo, oferecendo uma nova operação ou rota e descontinuando a anterior
gradualmente.

## Relação entre contratos e versões do aplicativo

Cada versão publicada do aplicativo deve ser registrada junto da versão do
contrato contra a qual foi desenvolvida e testada.

| Versão do contrato | Versão Flutter (`pubspec.yaml`) | Situação |
| --- | --- | --- |
| 1.0.0 | 1.0.0+1 | Linha de base atual do repositório; publicação em produção não registrada. |

A tabela deve ser atualizada ao preparar uma nova versão do aplicativo ou
do contrato. Versões não registradas não recebem garantia de compatibilidade
por esta política.

A versão de build do Flutter (`+N`) faz parte da identificação do aplicativo,
mas não altera a versão SemVer do contrato.

## Exemplos compartilhados e semântica dos campos

Os exemplos JSON em `examples/` são a referência compartilhada entre a
documentação OpenAPI e os testes automatizados do backend e do aplicativo.
O `openapi.yaml` os referencia por `externalValue`; os testes Rust leem os
mesmos arquivos em tempo de compilação, e os testes Flutter os carregam
durante a execução.

Os exemplos atuais cobrem:

| Arquivo | Comportamento representado |
| --- | --- |
| `institutional-auth-request.json` | Corpo da requisição de verificação institucional. |
| `institutional-verification-student.json` | Estudante com nome, e-mail e curso informados. |
| `institutional-verification-student-null-fields.json` | Estudante com nome, e-mail e curso nulos. |
| `institutional-verification-non-student.json` | Pessoa sem participação no grupo estudantil. |
| `api-error-unknown-institution.json` | Erro público para instituição não cadastrada. |

Os valores desses arquivos são fictícios e destinados exclusivamente à
documentação e aos testes. Não devem conter tokens reais, credenciais ou
dados pessoais.

### Campos obrigatórios, nulos e ausentes

A presença de uma propriedade e a possibilidade de seu valor ser `null`
são regras distintas do contrato:

- `microsoftAccessToken` é obrigatório e deve ser uma string não vazia.
  O backend não aplica `trim` antes de consultar o provedor.
- `profile.name` e `profile.email` devem estar presentes na resposta, mas
  aceitam string ou `null`.
- Quando `affiliation.type` é `student`, `courseName` deve estar presente,
  mas aceita string ou `null`.
- Quando `affiliation.type` é `nonStudent`, `courseName` não é retornado.
- `expiresAt` é obrigatório e representa a expiração da evidência institucional
  em data e hora RFC 3339.

Portanto, substituir um campo nulo por um campo omitido não é necessariamente
uma alteração compatível. A propriedade `required` e a nulabilidade precisam
ser avaliadas separadamente ao modificar os schemas.

### Envelope e códigos de erro

As respostas de erro da operação institucional utilizam o envelope
`{"error":{"code":"...","message":"...","details":null}}`.

- `error.code` é o identificador estável para tratamento programático.
  Clientes não devem depender do texto de `message` para classificar erros.
- `error.message` é uma mensagem pública legível e não deve expor diagnósticos
  internos, credenciais ou informações sensíveis.
- `error.details` deve estar presente no envelope. Nos erros atualmente
  documentados para a verificação institucional, seu valor é `null`.

Códigos conhecidos devem receber o tratamento correspondente à sua semântica.
Quando um cliente receber um código ainda desconhecido, deve evitar falhar
na desserialização do envelope e aplicar um tratamento seguro baseado no
status HTTP ou em uma falha genérica apropriada. A inclusão de um novo código
deve ser acompanhada de testes e atualização da documentação.

## Compatibilidade e suporte

Enquanto uma versão do aplicativo estiver registrada como suportada, o
backend deve preservar o protocolo que essa versão utiliza.

O encerramento do suporte de uma versão publicada do aplicativo deve ser
anunciado com antecedência mínima de 180 dias corridos. A data de fim do
suporte deve ser registrada na matriz de versões quando for definida.

Recursos estáveis da API que precisem ser removidos devem passar por
descontinuação:

1. Marcar a operação ou propriedade como `deprecated: true`.
2. Informar uma data `x-sunset` no formato `YYYY-MM-DD`, respeitando pelo
   menos 180 dias corridos de antecedência.
3. Manter o comportamento anterior até a data de encerramento anunciada.
4. Remover o recurso somente na data anunciada ou depois dela, verificando
   que nenhum cliente ainda suportado depende dele.

O CI aplica o prazo de 180 dias às descontinuações estáveis suportadas pelo
OASDiff. Recursos não cobertos pela verificação automatizada continuam
sujeitos à revisão de compatibilidade.

Se uma versão suportada do aplicativo ainda utilizar um recurso, seu suporte
deve ser estendido ou o aplicativo deve ser migrado antes da remoção.

## Processo de alteração

Todo PR que alterar o contrato deve:

1. Atualizar `openapi.yaml` junto da implementação ou do cliente afetado.
2. Atualizar `info.version` de acordo com a política SemVer.
3. Atualizar os arquivos em `examples/` quando os exemplos ou formatos
   compartilhados forem afetados, mantendo-os coerentes com os schemas.
4. Considerar todas as versões do aplicativo registradas como suportadas.
5. Incluir ou atualizar testes do backend e do aplicativo quando o protocolo
   ou os DTOs forem modificados. Os testes devem consumir os exemplos
   compartilhados, em vez de manter cópias independentes dos mesmos JSONs.
6. Descrever no PR o impacto sobre os consumidores e a estratégia de migração,
   quando aplicável.

O workflow valida os arquivos JSON referenciados por `externalValue`,
valida a especificação com Redocly e compara a versão proposta com a
especificação da branch de destino usando OASDiff. Alterações em qualquer
arquivo de `packages/api-contracts/` também acionam os workflows de testes
do backend e do aplicativo.

A comparação de especificações identifica mudanças incompatíveis declaradas
no OpenAPI; ela não substitui testes da implementação real. Os testes do
backend e do aplicativo continuam necessários para verificar os exemplos
e os comportamentos HTTP implementados.

## Exceções

Uma alteração incompatível intencional exige:

- justificativa e referência à issue ou decisão que aprovou a mudança;
- identificação das versões suportadas afetadas;
- estratégia de migração e data de encerramento da compatibilidade anterior;
- atualização da versão MAJOR do contrato;
- aprovação explícita dos mantenedores responsáveis.

A alternativa preferencial é introduzir o novo comportamento sem remover
imediatamente o anterior e aplicar o processo de descontinuação.

Quando uma exceção exigir ignorar uma detecção específica do OASDiff, a
exceção deve ser restrita à operação e à mudança aprovadas, registrada em
arquivo de exceções versionado e acompanhada da justificativa e referência
à aprovação. Não se deve desabilitar globalmente a detecção de mudanças
incompatíveis.

Exceções emergenciais, como correções necessárias por segurança, também
devem registrar o impacto sobre os clientes e a decisão dos mantenedores.

### Registro técnico de exceções

As exceções a findings de nível ERR aprovadas pelos mantenedores devem ser
registradas em `.oasdiff-err-ignore.txt`.

Cada linha identifica especificamente o endpoint e a descrição da alteração
que pode ser ignorada. O formato segue as regras de correspondência do OASDiff.

Não são permitidos padrões genéricos que ignorem uma rota inteira ou uma
categoria ampla de mudanças. A justificativa, a issue ou decisão de aprovação,
as versões afetadas e a estratégia de migração devem constar no PR.

O arquivo vazio significa que não há exceções ativas. Uma exceção deve ser
removida assim que deixar de ser necessária.

## Exceções da validação Redocly

O arquivo `.redocly.lint-ignore.yaml` registra exceções pontuais para regras
que não podem ser atendidas corretamente com as informações e os
comportamentos atualmente definidos pelo projeto:

- **`no-empty-servers`**: ainda não existe uma URL canônica de implantação
  definida para a API. A especificação não utiliza endereços fictícios ou
  referências a `localhost`.
- **`info-license`**: a licença do projeto ainda não foi formalmente definida.
- **`operation-4xx-response` em `/health` e `/ready`**: esses endpoints não
  recebem dados de entrada que justifiquem respostas 4XX. `/health` informa
  a saúde do processo, enquanto `/ready` informa a disponibilidade do
  PostgreSQL por meio de respostas 200 ou 503.

As exceções são limitadas aos respectivos nós da especificação. As regras
Redocly continuam sendo aplicadas às demais operações. Quando o projeto
definir uma licença ou uma URL oficial de implantação, as exceções
correspondentes deverão ser removidas.
