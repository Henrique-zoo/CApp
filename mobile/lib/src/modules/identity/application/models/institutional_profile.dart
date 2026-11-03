/// Representação institucional do usuário consumida pela aplicação.
///
/// Este modelo reúne o identificador de objeto Microsoft, os dados pessoais
/// opcionais e o vínculo estudantil. Mantê-lo separado do DTO permite que
/// alterações do protocolo ou do cache sejam adaptadas nas fronteiras sem
/// obrigar a apresentação a conhecer JSON.
///
/// ## Lugar no fluxo
///
/// O gateway constrói o perfil após uma resposta válida do backend, e o codec
/// pode reconstruí-lo a partir de um registro local compatível.
/// InstitutionalVerification agrega esse perfil ao prazo da evidência.
/// A interface consome os dados para apresentação, preservando ausências reais.
///
/// O identificador Microsoft não é o UID Firebase. A associação à conta que
/// solicitou a operação pertence ao resultado do caso de uso e ao envelope
/// do armazenamento.
///
/// ## Responsabilidade do modelo
///
/// O objeto é imutável e compara seus valores. Não consulta serviços, serializa
/// conteúdo ou provisiona um usuário interno. Campos nulos não devem ser
/// substituídos por dados inferidos, e o vínculo não define permissões gerais.
library;

import 'institutional_affiliation.dart';

/// Dados institucionais conhecidos de uma pessoa.
///
/// Não representa um usuário interno provisionado no CApp.
/// Também não contém credenciais, papéis gerais ou permissões.
///
/// A associação deste perfil a uma conta autenticada é realizada pelo
/// resultado da operação e pela coordenação da sessão.
final class InstitutionalProfile {
  /// Cria uma fotografia dos dados institucionais.
  ///
  /// [microsoftObjectId] identifica a pessoa no diretório Microsoft.
  /// É diferente do UID Firebase.
  ///
  /// [name] e [email] preservam a ausência de dados com valores nulos.
  /// [affiliation] representa o resultado da consulta ao vínculo estudantil.
  ///
  /// A construção não realiza validação de UUID, normalização de textos,
  /// consultas externas ou associação com a conta Firebase atual.
  const InstitutionalProfile({
    required this.microsoftObjectId,
    required this.name,
    required this.email,
    required this.affiliation,
  });

  /// Identificador da pessoa no diretório Microsoft.
  final String microsoftObjectId;

  /// Nome fornecido pela instituição, ou nulo quando ausente.
  final String? name;

  /// E-mail fornecido pela instituição, ou nulo quando ausente.
  final String? email;

  /// Resultado institucional do vínculo estudantil.
  final InstitutionalAffiliation affiliation;

  /// Compara todos os dados do perfil por conteúdo.
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        other is InstitutionalProfile &&
            other.microsoftObjectId == microsoftObjectId &&
            other.name == name &&
            other.email == email &&
            other.affiliation == affiliation;
  }

  /// Hash dos atributos utilizados pela igualdade.
  @override
  int get hashCode => Object.hash(microsoftObjectId, name, email, affiliation);
}
