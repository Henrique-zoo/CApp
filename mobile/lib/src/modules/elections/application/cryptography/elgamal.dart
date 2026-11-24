/// Gera chaves de autoridade e cifra cédulas da modalidade secreta no dispositivo.
///
/// A aplicação usa o ElGamal exponencial para que o backend possa agregar as
/// cifras por escolha. A decifração dos totais depende das contribuições das
/// autoridades. Cada cédula tem uma cifra por escolha, incluindo Nulo.
///
/// Na preparação da eleição, [generateAuthorityKeyPair] gera o par ElGamal da
/// autoridade no dispositivo do responsável. O caso de uso guarda o segredo
/// localmente até a apuração e publica o valor público com a prova de conhecimento.
///
/// O caso de uso fornece a chave pública conjunta e a ordem das escolhas do
/// mesmo manifesto validado. [encryptBallot] transforma a escolha em cifras
/// locais. A posição no resultado permite associar cada cifra ao ID da escolha
/// antes de montar e assinar o envelope que referencia o hash do manifesto.
/// Os nonces ficam restritos à cifragem. Em caso de falha de envio, o caso de
/// uso reutiliza o pacote já assinado para conservar a mesma submissão.
///
/// A biblioteca confere o grupo e a chave pública e verifica que a escolha
/// consta na lista recebida. A autenticidade do manifesto, a elegibilidade do
/// eleitor e o aceite da cédula são verificados fora da cifragem.
library;

import 'dart:math';

import 'package:uuid/uuid_value.dart';

part 'elgamal_tally.dart';

/// Identificador do grupo aceito no manifesto eleitoral v1.
const _groupId = 'rfc3526-modp14';

/// Primo de 2048 bits do grupo MODP 14, fixado no perfil eleitoral v1.
final _p = BigInt.parse(
  'ffffffffffffffffc90fdaa22168c234c4c6628b80dc1cd129024e088a67cc74'
  '020bbea63b139b22514a08798e3404ddef9519b3cd3a431b302b0a6df25f1437'
  '4fe1356d6d51c245e485b576625e7ec6f44c42e9a637ed6b0bff5cb6f406b7ed'
  'ee386bfb5a899fa5ae9f24117c4b1fe649286651ece45b3dc2007cb8a163bf05'
  '98da48361c55d39a69163fa8fd24cf5f83655d23dca3ad961c62f356208552bb'
  '9ed529077096966d670c354e4abc9804f1746c08ca18217c32905e462e36ce3b'
  'e39e772c180e86039b2783a2ec07a28fb5c55df06f4c52c9de2bcbf695581718'
  '3995497cea956ae515d2261898fa051015728e5a8aacaa68ffffffffffffffff',
  radix: 16,
);

/// Ordem do subgrupo, dada por `(p - 1) / 2` para o primo seguro do perfil.
final _q = (_p - BigInt.one) >> 1;

/// Gerador do subgrupo de ordem [_q] usado para a chave e para os bits cifrados.
final _g = BigInt.from(2);

/// Par ElGamal de uma única autoridade eleitoral.
///
/// [publicValue] pode ser publicado no manifesto. [secret] fica sob custódia da
/// autoridade para produzir a prova de conhecimento e a contribuição de apuração.
final class ElGamalAuthorityKeyPair {
  ElGamalAuthorityKeyPair._({required this.secret, required this.publicValue});

  /// Restaura o par do segredo conservado no aparelho.
  factory ElGamalAuthorityKeyPair.fromSecret(BigInt secret) {
    if (secret <= BigInt.zero || secret >= _q) {
      throw const FormatException('Escalar da autoridade fora do intervalo');
    }
    return ElGamalAuthorityKeyPair._(
      secret: secret,
      publicValue: _g.modPow(secret, _p),
    );
  }

  /// Prova que a autoridade conhece o segredo usado no valor público.
  ({BigInt commitment, BigInt response}) proveKey(
    BigInt Function(BigInt commitment) challenge,
  ) {
    final nonce = _sampleNonZeroScalar(Random.secure());
    final commitment = _g.modPow(nonce, _p);
    final response = (nonce + challenge(commitment) * secret) % _q;
    return (commitment: commitment, response: response);
  }

  /// Decifra parcialmente uma posição de um agregado previamente validado.
  ({BigInt value, BigInt commitmentG, BigInt commitmentA, BigInt response})
  contribute(
    BigInt base,
    BigInt Function(BigInt value, BigInt commitmentG, BigInt commitmentA)
    challenge,
  ) {
    final nonce = _sampleNonZeroScalar(Random.secure());
    final value = base.modPow(secret, _p);
    final commitmentG = _g.modPow(nonce, _p);
    final commitmentA = base.modPow(nonce, _p);
    final response =
        (nonce + challenge(value, commitmentG, commitmentA) * secret) % _q;
    return (
      value: value,
      commitmentG: commitmentG,
      commitmentA: commitmentA,
      response: response,
    );
  }

  /// Escalar privado mantido sob custódia local até a apuração.
  final BigInt secret;

  /// Elemento público calculado a partir do segredo.
  final BigInt publicValue;
}

/// Chave pública conjunta da eleição no grupo fixado pelo protocolo.
///
/// A validação é feita uma vez ao obter o valor do manifesto. As cifras de uma
/// mesma eleição podem então usar a mesma chave sem repetir esse cálculo.
final class ElGamalPublicKey {
  ElGamalPublicKey._(this.value);

  /// Constrói a chave a partir de [value] e do [groupId] do manifesto.
  ///
  /// Lança [ArgumentError] se o grupo divergir do perfil, se `value` estiver
  /// fora de `1 < value < p` ou se `value^q mod p` for diferente de `1`. A
  /// autenticidade do valor é conferida pela validação do manifesto.
  factory ElGamalPublicKey.fromValue(BigInt value, {required String groupId}) {
    if (groupId != _groupId) {
      throw ArgumentError('Grupo ElGamal não suportado.', 'groupId');
    }

    if (value <= BigInt.one ||
        value >= _p ||
        value.modPow(_q, _p) != BigInt.one) {
      throw ArgumentError('Chave ElGamal fora do subgrupo aceito.', 'value');
    }

    return ElGamalPublicKey._(value);
  }

  /// Elemento público validado, representado como inteiro de precisão arbitrária.
  final BigInt value;
}

/// Par `(a, b)` que cifra o bit de uma escolha.
///
/// A posição da cifra na cédula corresponde à posição da escolha no manifesto.
typedef ElGamalCiphertext = ({BigInt a, BigInt b});

/// Gera um novo par ElGamal para uma autoridade na preparação da eleição.
///
/// Cada chamada usa [Random.secure] para sortear um segredo uniforme em `[1, q)`
/// e deriva `publicValue = g^secret mod p` no grupo fixado pelo perfil eleitoral.
/// O segredo diferente de zero garante que o valor público difira da identidade.
///
/// O chamador deve manter o segredo sob custódia local e reutilizar o mesmo par
/// depois de registrar o valor público. A função retorna os valores em memória.
/// Propaga [UnsupportedError] se a plataforma não oferecer aleatoriedade segura.
ElGamalAuthorityKeyPair generateAuthorityKeyPair() {
  final random = Random.secure();

  final secret = _sampleNonZeroScalar(random);
  final publicValue = _g.modPow(secret, _p);

  return ElGamalAuthorityKeyPair._(secret: secret, publicValue: publicValue);
}

/// Cifra a escolha [selectedChoiceId] com a chave conjunta [publicKey].
///
/// [orderedChoiceIds] deve vir do mesmo manifesto validado que fornece a chave,
/// conter IDs distintos e incluir Nulo na última posição. Retorna uma lista de
/// tamanho fixo na ordem recebida. A posição escolhida cifra `1` e as demais
/// cifram `0`, cada uma com um nonce sorteado de forma independente.
/// Se um nonce produzir `b = 1` ou `b = g`, ele é descartado localmente e a
/// cifragem daquela posição é refeita para não expor o bit.
///
/// Lança [ArgumentError] se a escolha não constar na lista, antes de obter a
/// fonte de aleatoriedade ou cifrar qualquer escolha. Propaga [UnsupportedError]
/// se [Random.secure] não conseguir fornecer aleatoriedade segura na plataforma.
///
/// Exemplo com uma chapa e Nulo. Com essa escolha, a primeira cifra contém `0`
/// e a segunda contém `1`. A chave `4` é sintética e serve apenas ao exemplo.
///
/// ```dart
/// final slateId = UuidValue.withFormatValidation(
///   '11111111-1111-4111-8111-111111111111',
/// );
/// final nullChoiceId = UuidValue.withFormatValidation(
///   '00000000-0000-0000-0000-000000000000',
/// );
/// final ciphertexts = encryptBallot(
///   publicKey: ElGamalPublicKey.fromValue(
///     BigInt.from(4),
///     groupId: 'rfc3526-modp14',
///   ),
///   orderedChoiceIds: [slateId, nullChoiceId],
///   selectedChoiceId: nullChoiceId,
/// );
/// assert(ciphertexts.length == 2);
/// ```
List<ElGamalCiphertext> encryptBallot({
  required ElGamalPublicKey publicKey,
  required List<UuidValue> orderedChoiceIds,
  required UuidValue selectedChoiceId,
}) {
  final selectedIndex = orderedChoiceIds.indexOf(selectedChoiceId);

  if (selectedIndex == -1) {
    throw ArgumentError(
      'A escolha não consta no manifesto.',
      'selectedChoiceId',
    );
  }

  final random = Random.secure();

  return List<ElGamalCiphertext>.generate(
    orderedChoiceIds.length,
    (index) =>
        _encryptChoice(h: publicKey, m: index == selectedIndex, random: random),
    growable: false,
  );
}

/// Cifra o bit [m] com a chave [h] e um nonce sorteado com [random].
///
/// Calcula `a = g^r mod p` e `b = (g^m * h^r) mod p`. O fator `g^m` é `g`
/// para a posição selecionada e `1` para as demais. Como `h^r != 1`, a saída
/// `b = 1` revela `m = 1` e `b = g` revela `m = 0`. Ambas exigem novo sorteio.
ElGamalCiphertext _encryptChoice({
  required ElGamalPublicKey h,
  required bool m,
  required Random random,
}) {
  final gm = m ? _g : BigInt.one;

  while (true) {
    final r = _sampleNonZeroScalar(random);
    final b = (gm * h.value.modPow(r, _p)) % _p;

    if (b != BigInt.one && b != _g) {
      return (a: _g.modPow(r, _p), b: b);
    }
  }
}

/// Sorteia uniformemente um escalar no intervalo `[1, q)` por rejeição.
///
/// A amostragem é compartilhada entre segredos de autoridade e nonces de cifra.
/// [random] vem de [Random.secure]. Os bytes são combinados em um [BigInt] e a
/// máscara limita o sorteio ao número de bits de `q`. Descartar zero e valores a
/// partir de `q` preserva a mesma probabilidade para cada escalar aceito. Elevar
/// `g` a esse escalar produz um elemento diferente da identidade.
BigInt _sampleNonZeroScalar(Random random) {
  final bits = _q.bitLength;
  final byteCount = (bits + 7) ~/ 8;
  final mask = (BigInt.one << bits) - BigInt.one;

  while (true) {
    var candidate = BigInt.zero;

    for (var i = 0; i < byteCount; i++) {
      candidate = (candidate << 8) | BigInt.from(random.nextInt(256));
    }

    candidate &= mask;

    if (candidate > BigInt.zero && candidate < _q) {
      return candidate;
    }
  }
}
