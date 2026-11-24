/// Representação canônica da prova de posse da chave eleitoral.
library;

import 'package:uuid/uuid.dart';

/// Este prefixo separa assinaturas de cadastro das assinaturas de votação.
const voterRegistrationSignatureDomain = 'CAPP-ELECTIONS/REGISTRATION/1\u0000';

/// Mensagem assinada (ordenada lexicograficamente para equivaler a JCS).
/// Os UUIDs são obtidos da conta ativa e da eleição selecionada, não da DTO.
Map<String, Object> voterRegistrationSigningPayload(
  UuidValue accountId,
  UuidValue electionId,
  String publicKey,
) => {
  'accountId': accountId.uuid.toLowerCase(),
  'electionId': electionId.uuid.toLowerCase(),
  'publicKey': publicKey,
};

/// Dados enviados pelo HTTP: somente o que não pode ser obtido do contexto.
Map<String, Object> voterRegistrationRequestPayload(String publicKey) => {
  'publicKey': publicKey,
};
