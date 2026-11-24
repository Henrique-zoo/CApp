/// Porta usada pela aplicação para consultar e confirmar o cadastro eleitoral.
library;

import 'package:uuid/uuid.dart';

import '../voter_registration.dart';

abstract interface class VoterRegistrationGateway {
  /// Consulta a própria conta. [ensureCurrentAccount] deve ser executado
  /// antes do envio com a credencial resolvida e antes de entregar a resposta.
  Future<VoterRegistration> read(
    UuidValue electionId,
    void Function() ensureCurrentAccount,
  );

  /// Registra a prova no backend sob a mesma sessão que iniciou o fluxo.
  Future<VoterRegistration> register(
    VoterRegistrationProof proof,
    void Function() ensureCurrentAccount,
  );
}
