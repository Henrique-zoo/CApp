/// União do protocolo HTTP para o resultado do vínculo estudantil.
///
/// O backend usa o discriminador type para distinguir student e nonStudent.
/// Esta biblioteca representa esse formato e mantém suas variantes como partes
/// da mesma biblioteca Dart, permitindo conversão e seleção exaustiva.
///
/// ## Leitura e tradução
///
/// InstitutionalIdentityResponse desserializa affiliation por meio desta união.
/// A variante estudantil exige courseName presente, admitindo nulo; a não
/// estudantil não admite essa propriedade, inclusive nula.
/// Um discriminador desconhecido produz FormatException.
///
/// O mapper do gateway converte a variante para o modelo homônimo da aplicação.
/// O cache utiliza seu próprio codec e não depende destes DTOs.
///
/// ## Significado e fronteiras
///
/// nonStudent é resultado válido de uma consulta concluída, não uma forma de
/// representar indisponibilidade do provedor ou recusa de autenticação.
/// Sua apresentação pode orientar uma mensagem, mas não define papéis gerais
/// ou capacidades do usuário.
///
/// As partes não devem ser importadas isoladamente. Os DTOs das variantes não
/// substituem os modelos de aplicação e mantêm a igualdade padrão de Object.
library;

part 'student_affiliation.dart';
part 'non_student_affiliation.dart';

/// Resultado da verificação de participação no grupo institucional de estudantes.
/// A união não conhece validade temporal; ela pertence à resposta que a contém.
sealed class InstitutionalAffiliation {
  /// Permite que as variantes da biblioteca construam vínculos imutáveis.
  const InstitutionalAffiliation();

  /// Seleciona a variante a partir do discriminador JSON `type`.
  ///
  /// Aceita `student` e `nonStudent`. Lança [FormatException] para tipos
  /// desconhecidos ou para dados incompatíveis com a variante selecionada.
  /// A seleção não consulta o Graph ou valida autorização. As variantes aceitam
  /// chaves adicionais, com a restrição específica de `courseName` em não estudante.
  factory InstitutionalAffiliation.fromJson(Map<String, dynamic> json) {
    return switch (json['type']) {
      'student' => StudentAffiliation.fromJson(json),
      'nonStudent' => NonStudentAffiliation.fromJson(json),
      _ => throw FormatException('Tipo de vínculo institucional desconhecido'),
    };
  }

  /// Serializa o vínculo, incluindo seu discriminador `type`.
  /// A implementação deve seguir o contrato da variante, sem inserir tokens.
  Map<String, dynamic> toJson();
}
