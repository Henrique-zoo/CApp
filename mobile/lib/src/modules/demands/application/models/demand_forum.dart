/// Fóruns de visibilidade e escopo das demandas de um Centro Acadêmico.
library;

/// Fórum ao qual uma demanda pertence.
enum DemandForum {
  /// Fórum institucional do curso, restrito a discentes do curso e gestão do CA.
  institutional('institutional', 'Fórum Institucional', 'Institucional'),

  /// Fórum do espaço físico do CA, aberto a todos os usuários autenticados.
  physicalSpace('physical_space', 'Espaço Físico do CA', 'Espaço Físico');

  const DemandForum(this.wireValue, this.label, this.shortLabel);

  /// Valor canônico trafegado no contrato HTTP.
  final String wireValue;

  /// Rótulo completo do fórum para exibição ao usuário.
  final String label;

  /// Rótulo compacto usado em abas e badges.
  final String shortLabel;

  /// Converte o valor textual da API para [DemandForum].
  static DemandForum fromWire(String value) {
    for (final forum in DemandForum.values) {
      if (forum.wireValue == value) return forum;
    }
    throw FormatException('Fórum de demanda desconhecido: $value');
  }
}
