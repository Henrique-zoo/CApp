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
/// A variante estudantil exige grupo confirmado e curso externo não vazio.
/// A variante não estudantil expressa uma consulta concluída sem confirmação
/// do grupo ou do curso. A apresentação informa o acesso limitado resultante.
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
/// O curso vem exclusivamente da instituição; não pode ser declarado pelo usuário.
final class StudentAffiliation extends InstitutionalAffiliation {
  /// Cria o vínculo com o curso informado pela fonte institucional.
  ///
  /// Rejeita curso vazio ou em branco com [FormatException]. O texto válido é
  /// preservado sem normalização; a autorização das operações cabe ao servidor.
  StudentAffiliation({required this.courseName}) {
    if (courseName.trim().isEmpty) {
      throw const FormatException(
        'Vínculo estudantil exige curso institucional',
      );
    }
  }

  /// Curso não vazio informado pela instituição.
  final String courseName;

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

/// Grupo estudantil ou curso não confirmado na consulta institucional.
///
/// Permite entrada limitada, sem direitos de estudante em qualquer CA, com aviso
/// explícito. Sua presença não exige logout nem impede a conclusão do cadastro.
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
