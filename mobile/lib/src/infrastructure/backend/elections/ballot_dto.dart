/// Representações HTTP do aceite de uma cédula confidencial.
///
/// O cliente assina a cédula pública completa. A submissão HTTP leva as cifras,
/// a chave e o manifesto. A autenticação e a rota fornecem os demais dados
/// usados pelo backend para reconstituir a mensagem que o cliente assinou.
library;

/// Corpo de POST /elections/{id}/ballots do contrato SignedBallotSubmission.
final class BallotSubmissionDto {
  const BallotSubmissionDto({
    required this.manifestHash,
    required this.publicKey,
    required this.ciphertexts,
    required this.signature,
  });

  final String manifestHash;
  final String publicKey;
  final List<Map<String, dynamic>> ciphertexts;
  final String signature;

  Map<String, Object> toJson() => {
    'payload': {
      'manifestHash': manifestHash,
      'publicKey': publicKey,
      'ciphertexts': ciphertexts,
    },
    'signature': signature,
  };
}

/// Envelope persistido do voto e do recibo. A camada criptográfica confere
/// a assinatura e o contexto dos dois artefatos antes de usar a resposta.
final class RecordedBallotDto {
  const RecordedBallotDto._(this.entry, this.receipt, this.recovered);

  factory RecordedBallotDto.fromJson(Map<String, dynamic>? json) {
    if (json
        case {
          'entry': final Map<String, dynamic> entry,
          'receipt': final Map<String, dynamic> receipt,
          'recovered': final bool recovered,
        }
        when json.length == 3) {
      return RecordedBallotDto._(entry, receipt, recovered);
    }
    throw const FormatException('Confirmação de voto inválida');
  }

  final Map<String, dynamic> entry;
  final Map<String, dynamic> receipt;
  final bool recovered;
}
