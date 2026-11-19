//! Protocolo eleitoral compartilhado entre aplicativo, backend, autoridades e auditor.
//!
//! A cifragem no dispositivo e a apuração por participantes independentes exigem
//! uma representação comum: a assinatura de um JSON não pode depender da ordem
//! incidental dos campos ou da precisão numérica de uma plataforma. O contrato v1
//! em `packages/api-contracts/elections/` fixa formatos, bytes, parâmetros e vetores
//! antes de as quatro frentes implementarem seus próprios adaptadores.
//!
//! Este módulo separa [`api`], [`application`], [`domain`] e [`infrastructure`].
//! A #130 implementa entidades e invariantes; os contratos da #377 continuam
//! sendo a referência pública. Endpoints, persistência e telas são entregas das
//! respectivas issues consumidoras, ainda reservadas neste módulo.
//! A CLI que confere fixtures não é um servidor de votação nem o auditor do produto.
//!
//! # Modelo implementado na #130
//!
//! A comissão escolhe [`domain::value_objects::modality::ElectionModality`] ao
//! criar [`domain::entities::election::Election`]. O calendário existe antes da
//! configuração. A identificada recebe opções validadas e associa a conta à
//! escolha; a confidencial recebe manifesto verificado e conserva somente cifras.
//! Nulo é opção própria; sua posição final é obrigatória no manifesto confidencial.
//!
//! Na preparação, [`domain::entities::election::Election::invalidate_configuration`]
//! retira a prontidão quando opções/autoridades/chaves precisam de revisão pela
//! aplicação da #131. O histórico mantém cada configuração junto da revisão de
//! calendário vigente, inclusive manifestos posteriormente invalidados. Uma nova
//! publicação não modifica a origem ou as revisões assinadas que referenciam o
//! manifesto antigo. Depois da abertura, publicação e invalidação são recusadas.
//! Esse histórico pertence à persistência interna; não acrescenta mensagens aos
//! schemas públicos nem implementa assinatura, auditoria ou a configuração HTTP.
//!
//! A abertura congela configuração e revisão vigente. Reconstituição reproduz as
//! mesmas transições, incluindo revisões anteriores ao manifesto, sem flags que
//! desativem invariantes. Hashes de encerramento e agregado são referências
//! independentes do resultado. A apuração confidencial exige o conjunto exato de
//! contribuições verificadas; a identificada não exige esses artefatos.
//!
//! [`domain::entities::participation::Participation`] representa o primeiro aceite
//! por conta interna/eleição e o vínculo à chave pública, sem cadastro antecipado.
//! A repetição usa operação e digest do payload, recuperando o mesmo aceite até
//! após o fim; não precisa da chave privada ou de nova elegibilidade. O repositório
//! de #381 continua responsável por unicidade concorrente e commit conjunto de
//! participação, cédula, quadro e recibo, conforme [`application::ballot_ports`].
//! [`domain::entities::board_entry::BoardEntry`] conserva a cédula original e sua
//! posição encadeada. O aceite confidencial exige registro e recibo com a mesma
//! cédula, índice e digest do registro; a identificada dispensa ambos. O modelo
//! confere referências, enquanto o codec confere digests/assinaturas e o store
//! confirma o conjunto atomicamente. Repetições devolvem as evidências originais.
//!
//! `VerifiedManifest`, `VerifiedBallot` e `VerifiedTally` representam verificações
//! efetuadas nas portas de #378–#380/#139. Seus construtores de evidência
//! criptográfica são restritos ao crate. Não são algoritmos nem atalhos públicos de
//! construção. Os adaptadores verificam também os objetos persistidos antes de
//! reconstituí-los. O domínio não implementa ElGamal, RSA-PSS, JCS ou auditoria.
//! A contagem identificada usa um construtor próprio que rejeita resultados
//! confidenciais e cancelamentos administrativos.
//!
//! As entidades não implementam `Deserialize`: DTOs dos adaptadores devem chamar
//! construtores validados. Nenhum tipo persistido contém segredos de autoridades,
//! chave RSA privada do eleitor ou aleatoriedade de cifragem. A integridade de um
//! histórico completo depende das assinaturas e do backend confiável da #377;
//! reconstituição semântica não promete detectar uma reescrita integral maliciosa.
//!
//! Os testes unitários cobrem ambas as modalidades, participação, calendário,
//! estados e snapshots adulterados, sem banco ou cloud. Leem os casos semânticos
//! de `vectors.json`, incluindo a irregularidade compensada. Isso não substitui
//! verificação criptográfica dos vetores ou automação funcional dos BDDs.
//!
//! # Participação no fluxo
//!
//! A comissão indicada pela gestão organiza a eleição e aprova as chapas. Chapa
//! única é permitida. O manifesto congela opções ordenadas, parâmetros e chaves
//! de uma autoridade do servidor, uma da comissão e uma por chapa. Cada autoridade
//! guarda seu segredo ElGamal; o servidor possui somente seu próprio segredo e
//! a chave RSA de assinatura dos seus registros. Nunca recebe os demais segredos.
//!
//! O eleitor entra com a conta Firebase do CApp. O adaptador de identidade fornece
//! um principal confiável, e a porta de elegibilidade confere o vínculo institucional
//! vigente no instante do novo voto. Curso vem do Graph, não de declaração manual.
//! O identificador eleitoral público é opaco por conta/eleição; não é o UID Firebase.
//! Um contexto de tela pode antecipar a decisão, mas o aceite precisa reavaliá-la.
//!
//! Não há inscrição ou chave do eleitor obrigatória antes da abertura. Ao preparar
//! a cédula, o dispositivo gera uma chave RSA, cifra cada opção com aleatoriedade
//! nova e assina o payload completo. A primeira cédula aceita associa a chave à
//! conta autenticada. O pacote assinado permite repetição após falha de transporte;
//! consulta de participação/recibo em outro dispositivo dispensa a chave privada.
//!
//! O armazenamento deve confirmar voto, participação, índice, cadeia e recibo de
//! forma consistente, com unicidade por conta/eleição. O mesmo envio recupera o
//! aceite mesmo depois do encerramento; outro envio não cria nova participação.
//! Trocar dispositivo ou chave não altera a identidade usada nessa restrição.
//!
//! # Objetos e bytes do protocolo
//!
//! `schemas.json` especifica calendário inicial, manifesto, cédula, registro, recibo,
//! cancelamento, encerramento, agregado, contribuição de autoridade e resultado.
//! Tipos auxiliares compõem esses objetos; não exigem um endpoint ou entidade por
//! tipo. DTOs de telas e operações HTTP serão definidos nas integrações respectivas.
//! A camada de domínio continuará recebendo valores construídos/reconstituídos
//! por caminhos validados, sem depender de mapas JSON ou credenciais Firebase.
//!
//! Todo envelope público possui `payload` e `signature`. A entrada da assinatura
//! é UTF-8 de `CAPP-ELECTIONS/1` seguido de LF, concatenado com UTF-8 JCS do payload
//! conforme RFC 8785. O digest é SHA-256 dessa mesma entrada, em hexadecimal
//! minúsculo, excluindo a assinatura externa. Uma assinatura aninhada da cédula
//! participa do payload do registro e, portanto, do encadeamento desse registro.
//!
//! JCS preserva arrays e conteúdo Unicode, ordenando nomes de propriedades conforme
//! UTF-16. Rejeitar duplicatas antes de formar mapas. Inteiros criptográficos são
//! strings decimais sem sinal ou zeros à esquerda; índices e contagens são inteiros
//! pequenos com limites explícitos. Nunca converter os valores criptográficos para
//! ponto flutuante. Chaves/assinaturas usam Base64url canônico sem padding; a chave
//! pública é DER SubjectPublicKeyInfo RSA. O perfil exige RSA-PSS com SHA-256,
//! MGF1-SHA-256, salt de 32 bytes e expoente público 65537.
//!
//! O grupo é MODP 14 da RFC 3526: primo de 2048 bits, ordem q=(p-1)/2 e g=2.
//! Conferir faixa/subgrupo, rejeitar chave individual ou conjunta igual a 1 e
//! verificar as provas das autoridades. Segredos e nonces reais vêm de CSPRNG;
//! nonces não se repetem entre posições ou provas. As fixtures são públicas e
//! sintéticas: seus parâmetros de produção são reais, mas seu material de teste
//! jamais pode ser reutilizado em eleições reais.
//!
//! # Calendário e estados
//!
//! O intervalo é [startsAt, endsAt), pelo relógio do backend. Em preparação, a
//! comissão pode adiar a abertura e escolher outro fim, sem obrigação de manter a
//! duração; todo novo fim é estritamente posterior ao anterior. Depois de aberta,
//! somente o fim pode ser prorrogado, antes de vencer o prazo vigente. Não há
//! encerramento antecipado válido nem reabertura de uma eleição encerrada.
//!
//! O manifesto já usado nas cédulas não muda. O calendário inicial é um objeto
//! assinado independente, disponível antes de as chaves estarem prontas. O manifesto
//! referencia sua origem; revisões numeradas e assinadas encadeiam os adiamentos.
//! Uma revisão anterior ao manifesto pode ter manifestHash nulo.
//! O encerramento referencia o calendário efetivo e congela cabeça do quadro e
//! contagem. Voto, prorrogação, encerramento e cancelamento concorrentes devem
//! observar uma ordem transacional consistente no adaptador de persistência.
//!
//! A comissão pode cancelar uma eleição não terminal, inclusive por criação
//! equivocada em preparação. O backend registra ator, motivo e instante, conserva
//! o histórico e não apura antecipadamente. Cancelamento administrativo produz
//! `cancelled`; falha do guardrail produz `annulled`; falta de contribuição válida
//! produz `incomplete`. Sanção por descumprimento de autoridade não recupera uma
//! chave ou contribuição e será detalhada na governança, fora deste contrato.
//!
//! # Apuração e limites deliberados
//!
//! As opções de chapa seguem a ordem publicada no manifesto e Nulo fica por último,
//! com UUID zero reservado. Toda cédula produzida corretamente contém exatamente um 1: uma chapa
//! ou Nulo. Não existe opção de branco. O servidor agrega as cifras da fotografia
//! fechada; todas as autoridades contribuem para o mesmo agregado, sem revelar x.
//! As provas verificam conhecimento do segredo e correção da contribuição de
//! decifração. Não são provas de que o conteúdo da cédula é 0/1 e tem soma 1.
//!
//! Para cada opção, combinar as contribuições para obter g^T e procurar T em `[0,N]`,
//! onde N é o número de eleitores distintos com cédula aceita. Ausência de total
//! nesse intervalo ou soma dos totais diferente de N anula a eleição. Nulo participa
//! da soma. A anulação impede resultado válido ou posse, mas preserva evidências.
//! A igualdade é condição necessária: irregularidades compensadas podem passar.
//! O corpus inclui cifras e assinaturas válidas demonstrando exatamente esse limite.
//!
//! O modelo assume backend confiável e cliente suficientemente protegido por
//! App Check para geração correta da cédula. Não atribui ao App Check uma prova
//! matemática dessa propriedade. Não se promete resistência a um backend malicioso,
//! anonimato da participação, ausência de inferência em unanimidade ou resistência
//! à coerção.
//!
//! `valid` significa contagem consistente sob essas premissas. Vencedor, desempate,
//! quórum e posse pertencem às respectivas funcionalidades. Sem todas as autoridades
//! a contagem fica incompleta; não há recuperação por limiar nesta versão.
//!
//! # Transporte HTTPS
//!
//! HTTPS é obrigatório entre aplicativo e backend, tanto na votação secreta quanto
//! na identificada. A integração do protocolo na issue #385 deve terminar TLS no
//! Caddy e encaminhar HTTP ao backend:3000 pela rede privada do Compose, no mesmo host.
//! A porta HTTP do backend não fica exposta externamente. O aplicativo inicia as
//! requisições diretamente em HTTPS; redirecionar uma chamada HTTP depois de enviar
//! credenciais não protege os dados da primeira requisição.
//!
//! Em produção, usar certificado de uma CA pública, com emissão e renovação
//! automáticas pelo Caddy. Em desenvolvimento, uma CA local ou certificado
//! autoassinado exige confiança explícita no cliente. Instalar a CA no computador
//! do servidor não a torna automaticamente confiável no aplicativo do celular.
//! O app valida cadeia/confiança, validade e nome do servidor; não aceita qualquer
//! certificado nem desabilita essa validação. Chaves privadas TLS ficam no proxy,
//! fora do repositório, e são distintas dos segredos eleitorais.
//!
//! TLS protege o canal; Firebase Auth identifica a conta e App Check verifica a
//! atestação do aplicativo. Nenhum deles substitui os demais ou comprova o conteúdo
//! 0/1 da cédula cifrada. A integração deve testar uma chamada HTTPS válida, a recusa
//! de certificados não confiáveis, expirados ou de outro nome e a ausência de acesso
//! externo direto por HTTP. Os testes de fixtures desta issue não exercitam rede
//! nem comprovam a configuração TLS; o proxy será implementado na integração #385
//! e reaproveitado na integração de produto #386.
//!
//! # Fronteiras de implementação e validação
//!
//! `interfaces.json` fixa entradas, saídas e atomicidade esperadas das portas de
//! elegibilidade, codec, criptografia, persistência, relógio, aleatoriedade e leitura
//! pública. Não exige que o domínio execute HTTP, banco ou geração de segredo.
//! `security-matrix.json` associa garantias, premissas, limitações e testes reais.
//!
//! A frente A implementa criptografia/autoridades; B, domínio e coordenação; C,
//! assinatura, registro e cliente; D, auditor e testes adversariais. As quatro usam
//! os mesmos schemas/vetores e podem começar sem o servidor inteiro pronto.
//! O auditor de produto deverá produzir seu próprio veredito; o validador de
//! fixtures não substitui sua implementação ou a validação funcional dos BDDs.
//!
//! A CI confere schemas, JCS, hashes, RSA-PSS e relações dos vetores públicos por
//! `scripts/test_election_contracts.py`. Consumidores Rust/Dart devem ler os mesmos
//! arquivos e reproduzir bytes/hashes e verificar assinaturas. Assinaturas PSS novas
//! não são comparadas byte a byte, pois usam salt aleatório. Nenhuma chave privada
//! faz parte do corpus. Os cenários de produto mantêm pendência até automação real.
//!
//! Referências: [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785.html),
//! [RFC 3526](https://www.rfc-editor.org/rfc/rfc3526.html),
//! [RFC 8017](https://www.rfc-editor.org/rfc/rfc8017.html),
//! [RFC 8235](https://www.rfc-editor.org/rfc/rfc8235.html),
//! [RFC 8446](https://www.rfc-editor.org/rfc/rfc8446.html) e
//! [HTTPS automático no Caddy](https://caddyserver.com/docs/automatic-https).

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
