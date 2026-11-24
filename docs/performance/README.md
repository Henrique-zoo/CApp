# Plano de desempenho e capacidade do CApp

**Status:** proposta inicial, pendente de validação antes das medições.

## 1. Objetivo

Definir cenários representativos, conjuntos de dados sintéticos, perfis
de carga, metas de desempenho e condições reproduzíveis para avaliar
a capacidade do CApp à medida que aumentam o número de Centros Acadêmicos
(CAs), o volume de dados e a concorrência de uso.

O plano cobre o backend Rust, o PostgreSQL e o aplicativo Flutter.
Os resultados de cada plataforma devem ser registrados separadamente,
mantendo a mesma referência de cenário, perfil de dados e ambiente
sempre que a comparação exigir.

Este documento e `perfis.json` representam uma proposta de medição.
Não são resultados de benchmark e não demonstram que o sistema já atende
às metas definidas.

## 2. Escopo e dependências

- Issue #250: planejamento geral de desempenho e capacidade.
- Issue #253: métricas e observabilidade do backend.
- Issue #263: testes de carga, análise do backend e PostgreSQL.
- Issue #264: medição e otimização do aplicativo mobile.
- Issue #258: definição dos cenários, perfis, metas e condições de medição.

Os cenários compartilhados estão em
`features/desempenho_e_capacidade.feature`. Os parâmetros versionados
estão em `docs/performance/perfis.json`.

Não serão criados documentos separados para backend e mobile nesta issue.

## 3. Estado atual e limites conhecidos

A inspeção inicial do repositório encontrou as rotas globais
`GET /health` e `GET /ready` em `backend/src/api/router.rs`.
A primeira verifica a saúde do servidor; a segunda verifica a comunicação
com o PostgreSQL. Elas não representam, isoladamente, os fluxos funcionais
de consulta e escrita previstos para o produto.

O diretório `backend/migrations/` ainda contém somente `.keep`, e os
módulos de funcionalidades do aplicativo apresentam áreas de estruturação
inicial. Por isso, não se deve presumir que endpoints, operações de
persistência ou telas de negócio já estejam implementados.

Antes de executar um fluxo de carga, devem ser mapeados os endpoints
e as jornadas de usuário realmente disponíveis. Fluxos inexistentes
permanecem pendentes; não devem ser substituídos por endpoints ou telas
inventados para produzir um resultado aparentemente completo.

## 4. Fluxos representativos

O plano considera os seguintes comportamentos, quando disponíveis:

1. Consulta de listagens.
2. Consulta de detalhes.
3. Busca de conteúdo.
4. Escrita válida.
5. Uso concorrente por diferentes CAs.
6. Troca do CA ativo e atualização do conteúdo exibido.
7. Inicialização fria e inicialização subsequente do aplicativo.
8. Navegação e consulta em diferentes condições de rede.
9. Uso prolongado, observando fluidez, memória e falhas.

Os fluxos de API e os de interface são medidos separadamente. O tempo
de resposta da API não deve ser confundido com o tempo entre uma ação
do usuário e a apresentação do conteúdo no aplicativo.

Os cenários Gherkin devem manter a cobertura pretendida mesmo enquanto
a implementação ou automação estiver pendente. A remoção de
`@pending_backend` ou `@pending_mobile` só deve acontecer quando o
cenário correspondente estiver pronto para execução naquela plataforma.

## 5. Conjuntos de dados sintéticos

`perfis.json` define três tamanhos iniciais:

| Perfil | CAs | Usuários | Registros de conteúdo | Registros operacionais |
|---|---:|---:|---:|---:|
| Pequeno | 10 | 1.000 | 5.000 | 2.000 |
| Médio | 50 | 10.000 | 50.000 | 20.000 |
| Grande | 200 | 50.000 | 200.000 | 100.000 |

A geração de dados deve utilizar a semente fixa registrada na configuração,
permitindo repetir uma medição com a mesma massa de dados.

Os perfis incluem uma distribuição desigual: os 10% de CAs mais utilizados
concentram inicialmente 60% dos registros e 60% das requisições. Essa
distribuição é uma hipótese de teste, e não uma afirmação sobre a utilização
real do CApp.

Os volumes representam pontos de comparação propostos. Devem ser adequados
à estrutura real dos dados antes da implementação da ferramenta de geração.

Somente dados sintéticos podem ser usados. Não utilizar dados pessoais
ou cargas pesadas contra o ambiente de produção.

## 6. Perfis de carga do backend

Os perfis iniciais estão em `perfis.json`:

| Perfil | Carga proposta | Objetivo |
|---|---|---|
| Smoke | 1 requisição/s por 30 s | Verificar a preparação e a execução básica |
| Baseline | 5 requisições/s por 300 s | Obter uma referência inicial |
| Normal | 20 requisições/s por 600 s | Avaliar o uso contínuo esperado no modelo |
| Peak | Subida de 20 para 50 requisições/s em 60 s, mantendo o pico por 300 s | Avaliar capacidade e utilização de recursos |
| Spike | Subida de 20 para 80 requisições/s em 10 s, mantendo por 60 s | Avaliar reação a uma rajada |
| Soak | 10 requisições/s por 1.800 s | Identificar instabilidade ou crescimento de recursos ao longo do tempo |

A mistura inicial de operações está registrada em `perfis.json`.
Ela é uma proposta para o conjunto completo de fluxos e não deve ser tratada
como tráfego já observado no produto.

Antes de executar uma medição, mapear os fluxos realmente disponíveis.
Se um tipo de operação não existir, registrar a limitação, ajustar e
versionar a mistura de carga antes da comparação. Não simular uma operação
inexistente como se fosse uma requisição funcional do CApp.

## 7. Metas propostas para backend e PostgreSQL

Os valores iniciais estão em `perfis.json`:

- Leituras: P95 de até 500 ms e P99 de até 1.000 ms.
- Escritas válidas: P95 de até 800 ms e P99 de até 1.500 ms, quando o fluxo
  correspondente estiver implementado e mapeado.
- Taxa máxima de erros inesperados: 1%.
- CPU média da API e do banco: até 70% no perfil normal e até 85% no pico.
- Crescimento de memória durante o teste prolongado: até 10%.
- Recuperação após a rajada: retorno ao comportamento de referência em até
  60 segundos.

Esses números são metas iniciais propostas para orientar a avaliação,
não limites extraídos do relatório de funcionalidades nem resultados
medidos no ambiente atual.

A latência proposta procura estabelecer um objetivo verificável para
operações interativas. O limite de erros define um critério inicial
de qualidade. Os limites de CPU preservam margem de capacidade, enquanto
o teste prolongado e a recuperação após rajada ajudam a identificar
instabilidade e degradação persistente.

As metas devem ser revisadas considerando as características do produto,
o ambiente de execução e uma primeira medição de referência. Devem ser
aprovadas e fixadas antes de comparar otimizações. Alterações posteriores
precisam ser documentadas, sem substituir silenciosamente os valores
anteriores.

## 8. Perfis e metas do aplicativo mobile

`perfis.json` contempla dispositivos Android de duas categorias iniciais:
um dispositivo intermediário com pelo menos 4 GB de RAM e outro com
recursos mais limitados, com pelo menos 3 GB.

Quando possível, usar dispositivos físicos e registrar modelo, versão
do sistema operacional, memória, resolução, arquitetura e modo de build.
O modo `Profile` é a referência inicial para os benchmarks. Resultados
de `Debug` não devem ser misturados com resultados de `Profile` ou
`Release`.

As condições de rede propostas são:

- Favorável: RTT de 30 ms, download de 20 Mbps e upload de 10 Mbps.
- Móvel comum: RTT de 100 ms, download de 10 Mbps e upload de 2 Mbps.
- Limitada: RTT de 250 ms, download de 1,5 Mbps, upload de 0,5 Mbps
  e perda de pacotes de 1%.

As jornadas incluem inicialização fria e subsequente, navegação entre
lista e detalhe, busca, troca de CA ativo e escrita válida quando
existir um fluxo implementado.

As metas iniciais são:

- Inicialização fria: P95 de até 3.000 ms.
- Inicialização subsequente: P95 de até 1.500 ms.
- Primeiro conteúdo útil: P95 de até 2.000 ms na rede móvel comum
  e até 4.000 ms na rede limitada.
- Em 60 Hz: P95 do tempo de quadro de até 16,7 ms e P99 de até 33,3 ms.
- Memória PSS do processo: P95 de até 250 MB no dispositivo intermediário
  e até 200 MB no dispositivo restrito.
- Crescimento de memória durante uso prolongado: até 10%.
- Falhas do aplicativo durante a execução controlada: zero.

Esses valores são hipóteses de engenharia a validar. A referência de
16,7 ms corresponde aproximadamente ao orçamento de um quadro em uma
tela de 60 Hz; os demais limites são objetivos iniciais para comparar
execuções, não garantias de desempenho do aplicativo.

## 9. Métricas e registros

Cada execução deve registrar, conforme a plataforma e as ferramentas
disponíveis:

- commit testado e versões das ferramentas;
- ambiente, sistema operacional e características de CPU e memória;
- perfil e volume de dados;
- fluxos incluídos e operações excluídas por ainda não existirem;
- duração, carga solicitada e carga efetivamente alcançada;
- latência P50, P95 e P99;
- taxa de requisições, erros inesperados e timeouts;
- uso de CPU e memória pelo backend, PostgreSQL e processo mobile;
- latência de consultas e espera por conexão, quando instrumentadas;
- inicialização, apresentação de conteúdo, fluidez, crashes e OOM no mobile;
- cumprimento de cada meta, limitações e observações.

A ausência de instrumentação deve ser registrada como limitação, e não
como um valor zero ou como evidência de que não há problema.

## 10. Ambiente e metodologia de comparação

1. Registrar o commit, o ambiente, as ferramentas e o perfil escolhido.
2. Preparar ambiente isolado e dados sintéticos reproduzíveis.
3. Mapear os fluxos disponíveis e validar as condições de execução.
4. Executar o teste smoke antes dos perfis mais longos.
5. Realizar aquecimento quando aplicável e registrar essa etapa.
6. Repetir cada medição comparável pelo menos três vezes.
7. Registrar métricas, erros e limitações em cada repetição.
8. Comparar resultados apenas sob condições equivalentes.
9. Preservar os resultados de referência e o histórico de alterações nas metas.

Quando possível, manter o gerador de carga separado dos recursos medidos.
Para comparar versões, utilizar o mesmo perfil de dados, a mesma mistura
de operações, as mesmas condições de rede e o mesmo tipo de dispositivo.

Os resultados do backend e do mobile devem ser interpretados em conjunto
quando representarem a mesma jornada, mas seus tempos e métricas não devem
ser confundidos.

## 11. Critérios de conclusão da issue

- [ ] Fluxos representativos estão documentados, com pendências identificadas
      quando não houver implementação ou automação.
- [ ] Cenários Gherkin e perfis reproduzíveis estão versionados.
- [ ] Backend e mobile possuem cenários identificáveis por suas tags.
- [ ] Os conjuntos de dados cobrem crescimento no número de CAs e utilização
      desigual entre eles.
- [ ] As metas propostas possuem justificativa e estão identificadas como
      pendentes de aprovação antes da comparação de otimizações.
- [ ] Ambiente, dispositivos, rede, ferramentas e método de comparação
      estão documentados.
- [ ] Não há afirmação de que os resultados foram medidos quando isso ainda
      não ocorreu.
