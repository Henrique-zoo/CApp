part of 'elgamal.dart';

/// Lê um elemento público do grupo. O agregado vazio pode conter a identidade.
BigInt readElectionElement(String value, {bool allowIdentity = true}) {
  final number = _readDecimal(value);
  if (number < BigInt.one ||
      number >= _p ||
      (!allowIdentity && number == BigInt.one) ||
      number.modPow(_q, _p) != BigInt.one) {
    throw const FormatException('Elemento fora do grupo eleitoral');
  }
  return number;
}

BigInt _readDecimal(String value) {
  if (value.length > 617 || !RegExp(r'^(0|[1-9][0-9]*)$').hasMatch(value)) {
    throw const FormatException('Inteiro eleitoral fora do formato');
  }
  return BigInt.parse(value);
}

BigInt _response(String value) {
  final scalar = _readDecimal(value);
  if (scalar >= _q) throw const FormatException('Resposta fora do subgrupo');
  return scalar;
}

/// Multiplica elementos públicos ou cifras na ordem fornecida pelo manifesto.
BigInt multiplyElectionElements(BigInt left, BigInt right) =>
    (left * right) % _p;

/// Confere a prova Schnorr de posse do segredo de uma chave eleitoral.
bool verifyAuthorityKeyProof({
  required BigInt publicValue,
  required BigInt commitment,
  required BigInt challenge,
  required String response,
}) {
  return _g.modPow(_response(response), _p) ==
      multiplyElectionElements(
        commitment,
        publicValue.modPow(challenge % _q, _p),
      );
}

/// Confere que a contribuição usa o mesmo expoente da chave pública.
bool verifyElectionDecryption({
  required BigInt publicValue,
  required BigInt base,
  required BigInt value,
  required BigInt commitmentG,
  required BigInt commitmentA,
  required BigInt challenge,
  required String response,
}) {
  final scalar = _response(response);
  final c = challenge % _q;
  return (base == BigInt.one) == (value == BigInt.one) &&
      _g.modPow(scalar, _p) ==
          multiplyElectionElements(commitmentG, publicValue.modPow(c, _p)) &&
      base.modPow(scalar, _p) ==
          multiplyElectionElements(commitmentA, value.modPow(c, _p));
}

/// Recupera totais no intervalo de zero à contagem de cédulas.
/// Retorna nulo se alguma posição produzir um expoente fora desse intervalo.
List<int>? recoverElectionTotals(
  List<ElGamalCiphertext> aggregate,
  List<List<BigInt>> shares,
  int voterCount,
) {
  if (voterCount < 0 ||
      voterCount > 100000 ||
      shares.any((share) => share.length != aggregate.length)) {
    throw const FormatException('Dimensão de apuração inválida');
  }
  final width = sqrt(voterCount + 1).ceil();
  final offsets = <BigInt, int>{};
  var power = BigInt.one;
  for (var offset = 0; offset < width; offset++) {
    offsets[power] = offset;
    power = multiplyElectionElements(power, _g);
  }
  final blockStep = power.modInverse(_p);

  final totals = <int>[];
  for (var position = 0; position < aggregate.length; position++) {
    final factor = shares.fold<BigInt>(
      BigInt.one,
      (product, share) => multiplyElectionElements(product, share[position]),
    );
    var encoded = multiplyElectionElements(
      aggregate[position].b,
      factor.modInverse(_p),
    );
    int? total;
    for (var block = 0; block < width; block++) {
      final offset = offsets[encoded];
      if (offset != null) {
        total = block * width + offset;
        break;
      }
      encoded = multiplyElectionElements(encoded, blockStep);
    }
    if (total == null || total > voterCount) return null;
    totals.add(total);
  }
  return totals;
}
