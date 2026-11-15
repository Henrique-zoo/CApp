//! Protocolo eleitoral compartilhado entre aplicativo, backend, autoridades e auditor.
//!
//! A cifragem no dispositivo e a apuração por participantes independentes exigem
//! uma representação comum: a assinatura de um JSON não pode depender da ordem
//! incidental dos campos ou da precisão numérica de uma plataforma. O contrato v1
//! em `packages/api-contracts/elections/` fixa formatos, bytes, parâmetros e vetores
//! antes de as quatro frentes implementarem seus próprios adaptadores.
//!
//! Este módulo conserva os espaços de [`api`], [`application`], [`domain`] e
//! [`infrastructure`]. Os contratos da issue #377 não implementam endpoints,
//! entidades, persistência ou telas: essas entregas pertencem às issues consumidoras.
//! A CLI que confere fixtures não é um servidor de votação nem o auditor do produto.
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
//! matemática dessa propriedade. HTTPS foi adiado por decisão do projeto; riscos
//! de interceptação, alteração e reutilização de credenciais em trânsito são
//! reconhecidos. Não se promete resistência a um backend malicioso, anonimato
//! da participação, ausência de inferência em unanimidade ou resistência à coerção.
//!
//! `valid` significa contagem consistente sob essas premissas. Vencedor, desempate,
//! quórum e posse pertencem às respectivas funcionalidades. Sem todas as autoridades
//! a contagem fica incompleta; não há recuperação por limiar nesta versão.
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
//! [RFC 8017](https://www.rfc-editor.org/rfc/rfc8017.html) e
//! [RFC 8235](https://www.rfc-editor.org/rfc/rfc8235.html).

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
