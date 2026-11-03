/// Modelo de aplicação do vínculo estudantil informado pela instituição.
///
/// Esta união preserva o resultado factual da verificação sem carregar o formato
/// HTTP, o diretório Microsoft ou a representação persistida. Ela pode ser usada
/// pela coordenação e pela apresentação com o mesmo significado.
///
/// ## Origem e percurso
///
/// O gateway converte o vínculo do DTO para StudentAffiliation ou
/// NonStudentAffiliation; o codec reconstrói as mesmas variantes ao ler o cache.
/// InstitutionalProfile agrega esse resultado, acompanhado dos dados do perfil.
///
/// A variante estudantil admite o texto externo do curso, inclusive nulo.
/// A variante não estudantil expressa uma consulta concluída sem confirmação
/// de participação no grupo, e pode orientar uma mensagem informativa.
///
/// ## Limites semânticos
///
/// As duas variantes são resultados válidos. Erro de comunicação, cancelamento
/// e falha de autenticação não são convertidos em NonStudentAffiliation.
/// O tipo não define papéis gerais ou capacidades do usuário; a validade temporal
/// pertence à verificação, e a autorização de operações pertence ao servidor.
library;

/// Vínculo estudantil conhecido na última verificação institucional.
///
/// A validade temporal pertence à verificação que contém este vínculo.
/// A construção de uma variante não consulta a instituição.
///
/// As variantes possuem igualdade por conteúdo.
sealed class InstitutionalAffiliation {
  /// Permite construir as variantes pertencentes a esta biblioteca.
  const InstitutionalAffiliation();
}

/// Participação estudantil reconhecida pela instituição.
///
/// O curso é uma informação opcional e não determina permissões do aplicativo.
final class StudentAffiliation extends InstitutionalAffiliation {
  /// Cria o vínculo com o curso informado pela fonte institucional.
  ///
  /// [courseName] pode ser nulo quando a instituição não fornece esse dado.
  /// O texto é preservado sem normalização.
  const StudentAffiliation({required this.courseName});

  /// Curso informado pela instituição, ou nulo quando ausente.
  final String? courseName;

  /// Compara a variante e o curso informado
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        other is StudentAffiliation && other.courseName == courseName;
  }

  /// Hash compatível com a igualdade por conteúdo.
  @override
  int get hashCode => Object.hash(StudentAffiliation, courseName);
}

/// Participação estudantil não reconhecida na consulta institucional.
///
/// Deve orientar uma mensagem informativa ao usuário.
/// Sua presença não exige logout e não impede, por si só, a entrada no app.
final class NonStudentAffiliation extends InstitutionalAffiliation {
  /// Cria o resultado de vínculo estudantil não reconhecido.
  const NonStudentAffiliation();

  /// Considera equivalentes todas as instâncias desta variante.
  @override
  bool operator ==(Object other) {
    return other is NonStudentAffiliation;
  }

  /// Hash compartilhado pelas instâncias equivalentes de variante.
  @override
  int get hashCode => Object.hash(NonStudentAffiliation, 0);
}
