/// Regressões das primitivas de votação secreta usadas no dispositivo.
///
/// Lê os parâmetros públicos do contrato e confere as cifras com uma
/// decifração de referência. Os segredos fixos são exclusivos destes testes.
library;

import 'dart:convert';
import 'dart:io';

import 'package:capp/src/modules/elections/application/cryptography/elgamal.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid_value.dart';

void main() {
  late BigInt prime;
  late BigInt order;
  late BigInt generator;
  late String groupId;

  final aurora = UuidValue.withFormatValidation(
    '00000100-1111-4111-8111-111111111111',
  );
  final horizonte = UuidValue.withFormatValidation(
    '00000200-1111-4111-8111-111111111111',
  );
  final nulo = UuidValue.withFormatValidation(
    '00000000-0000-0000-0000-000000000000',
  );

  setUpAll(() {
    final profile = jsonDecode(
      File('../packages/api-contracts/elections/profile.json')
          .readAsStringSync(),
    ) as Map<String, Object?>;
    final group = profile['group']! as Map<String, Object?>;
    groupId = group['id']! as String;
    prime = BigInt.parse(group['primeHex']! as String, radix: 16);
    order = (prime - BigInt.one) >> 1;
    generator = BigInt.parse(group['generator']! as String);
  });

  test('recusa grupos e chaves públicas incompatíveis com o perfil', () {
    expect(
      ElGamalPublicKey.fromValue(generator, groupId: groupId).value,
      generator,
    );
    expect(
      () => ElGamalPublicKey.fromValue(generator, groupId: 'rfc3526-modp15'),
      throwsArgumentError,
    );

    for (final value in [
      -BigInt.one,
      BigInt.zero,
      BigInt.one,
      prime - BigInt.one,
      prime,
      prime + generator,
    ]) {
      expect(
        () => ElGamalPublicKey.fromValue(value, groupId: groupId),
        throwsArgumentError,
      );
    }
  });

  test('gera uma chave de autoridade válida no grupo do contrato', () {
    final key = generateAuthorityKeyPair();

    expect(key.secret, greaterThanOrEqualTo(BigInt.one));
    expect(key.secret, lessThan(order));
    expect(key.publicValue, generator.modPow(key.secret, prime));
    expect(
      ElGamalPublicKey.fromValue(key.publicValue, groupId: groupId).value,
      key.publicValue,
    );
  });

  test('recusa uma escolha ausente do manifesto', () {
    final publicKey = ElGamalPublicKey.fromValue(generator, groupId: groupId);

    expect(
      () => encryptBallot(
        publicKey: publicKey,
        orderedChoiceIds: [aurora, nulo],
        selectedChoiceId: horizonte,
      ),
      throwsArgumentError,
    );
  });

  test('cifra chapa e Nulo nas posições corretas com nonces independentes', () {
    // Três autoridades fictícias com segredos 1, 2 e 3. O segredo conjunto é
    // conhecido apenas para conferir as cifras produzidas neste teste.
    final secret = BigInt.from(6);
    final publicKey = ElGamalPublicKey.fromValue(
      generator.modPow(secret, prime),
      groupId: groupId,
    );
    final choices = [aurora, horizonte, nulo];
    final selections = [aurora, nulo, nulo];
    final firstComponents = <BigInt>{};
    final totals = List.filled(choices.length, 0);

    for (final selected in selections) {
      final ballot = encryptBallot(
        publicKey: publicKey,
        orderedChoiceIds: choices,
        selectedChoiceId: selected,
      );

      expect(ballot, hasLength(choices.length));

      for (final (index, cipher) in ballot.indexed) {
        expect(cipher.a, greaterThan(BigInt.one));
        expect(cipher.a, lessThan(prime));
        expect(cipher.a.modPow(order, prime), BigInt.one);
        expect(cipher.b, greaterThan(BigInt.one));
        expect(cipher.b, isNot(generator));
        expect(cipher.b, lessThan(prime));
        expect(cipher.b.modPow(order, prime), BigInt.one);
        expect(firstComponents.add(cipher.a), isTrue, reason: 'nonce repetido');

        final mask = cipher.a.modPow(secret, prime);
        final decoded = (cipher.b * mask.modInverse(prime)) % prime;
        final isSelected = choices[index] == selected;

        expect(decoded, isSelected ? generator : BigInt.one);

        totals[index] += decoded == generator ? 1 : 0;
      }
    }

    expect(totals, [1, 0, 2]);
  });
}
