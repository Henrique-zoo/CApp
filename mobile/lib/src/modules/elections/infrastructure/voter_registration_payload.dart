/// Representação do payload assinado e enviado para o cadastro eleitoral v1.
library;

import 'package:uuid/uuid.dart';

/// Os campos ASCII mantêm a ordem lexicográfica usada por JCS. Os valores
/// contêm somente UUIDs, Base64url, um literal e um inteiro.
Map<String, Object> voterRegistrationPayload(
  UuidValue accountId,
  UuidValue electionId,
  String publicKey,
) => {
  'accountId': accountId.uuid.toLowerCase(),
  'electionId': electionId.uuid.toLowerCase(),
  'kind': 'voter-registration',
  'publicKey': publicKey,
  'version': 1,
};
