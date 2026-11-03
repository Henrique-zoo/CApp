/// Tradução pura entre os DTOs HTTP e os modelos institucionais da aplicação.
///
/// Protocolo e aplicação possuem representações distintas, mesmo quando os campos
/// coincidem. Esta biblioteca concentra a correspondência entre elas para evitar
/// que telas e controladores dependam do contrato JSON do backend.
///
/// ## Transformação
///
/// Depois de desserializar uma resposta, o gateway entrega seu DTO ao mapper.
/// A função converte a variante de vínculo, cria InstitutionalProfile e envolve
/// o perfil em InstitutionalVerification com o expiresAt recebido.
///
/// Nome, e-mail e curso ausentes são preservados. O vínculo não estudantil mantém
/// seu significado de resultado válido. A normalização do instante para UTC
/// ocorre no construtor do modelo de aplicação.
///
/// ## Lugar na arquitetura
///
/// O mapper pertence à infraestrutura do módulo por conhecer os dois lados da
/// fronteira. Não executa I/O, não compartilha objetos de perfil do transporte
/// com a aplicação e não reaproveita DTOs como formato de cache.
///
/// A decisão de aceitar o prazo e associar o resultado à conta pertence à
/// coordenação. Mapear uma resposta não autentica o usuário ou autoriza operações.
library;

import 'package:capp/src/infrastructure/backend/identity/dto/institutional_affiliation.dart'
    as wire_affiliation;
import 'package:capp/src/infrastructure/backend/identity/dto/institutional_verification_response.dart'
    as wire_response;

import 'package:capp/src/modules/identity/application/models/institutional_affiliation.dart';
import 'package:capp/src/modules/identity/application/models/institutional_profile.dart';
import 'package:capp/src/modules/identity/application/models/institutional_verification.dart';

/// Converte [response] em uma verificação própria da aplicação.
///
/// Preserva nome, e-mail e curso ausentes. O resultado não estudantil permanece
/// uma variante válida do vínculo, sem ser convertido em falha.
///
/// A normalização do instante para UTC pertence ao construtor do modelo.
/// A expiração não é avaliada aqui: a política de prontidão pertence
/// à coordenação da sessão.
///
/// Não compartilha objetos de perfil ou vínculo do transporte com a aplicação.
InstitutionalVerification mapInstitutionalVerification(
  wire_response.InstitutionalVerificationResponse response,
) {
  final profile = response.profile;

  final affiliation = switch (profile.affiliation) {
    wire_affiliation.StudentAffiliation(:final courseName) =>
      StudentAffiliation(courseName: courseName),
    wire_affiliation.NonStudentAffiliation() => const NonStudentAffiliation(),
  };

  return InstitutionalVerification(
    profile: InstitutionalProfile(
      microsoftObjectId: profile.microsoftObjectId,
      name: profile.name,
      email: profile.email,
      affiliation: affiliation,
    ),
    expiresAt: response.expiresAt,
  );
}
