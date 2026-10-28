/// API pública dos componentes visuais compartilhados pelo aplicativo.
///
/// Disponibiliza componentes genéricos para a composição global e para
/// a apresentação dos módulos, sem expor seus caminhos internos.
///
/// WarningBox apresenta um aviso incorporado ao layout.
/// TemporaryWarning apresenta ocorrências por meio de um SnackBar temporário.
///
/// Os componentes recebem dados de apresentação. A interpretação de falhas
/// e a produção das mensagens continuam pertencendo aos seus módulos.
///
/// Importar esta biblioteca apenas disponibiliza os tipos; não apresenta
/// avisos, registra observadores ou executa operações externas.
library;

export 'widgets/warning_box.dart' show WarningBox;
export 'widgets/temporary_warning.dart' show TemporaryWarning;
