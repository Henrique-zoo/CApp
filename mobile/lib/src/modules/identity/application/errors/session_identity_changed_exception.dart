/// Sinalização de uma operação que perdeu seu contexto de identidade.
///
/// Login, consulta remota e persistência são assíncronos. Durante suas esperas,
/// a conta pode mudar ou o controlador pode ser encerrado. Esta biblioteca
/// representa a impossibilidade de aplicar uma conclusão à sessão originalmente
/// esperada, sem classificá-la como recusa de credenciais.
///
/// ## Uso na coordenação
///
/// InstitutionalAuthService compara o UID antes e depois das integrações.
/// SessionController também confere o contexto, a revisão e o UID antes de
/// aceitar resultados ou iniciar uma gravação. A revisão detecta mudanças que
/// retornem ao mesmo UID, como uma sequência A/B/A.
///
/// Quando o contexto deixa de ser aplicável, a sinalização impede que aquela
/// operação atualize a sessão atual. Ela não desfaz efeitos já concluídos no
/// provedor, no backend ou no armazenamento.
///
/// A apresentação trata essa conclusão como operação obsoleta. A restauração
/// e os eventos da conta vigente continuam sendo a fonte do estado mostrado.
library;

/// A identidade necessária à operação foi encerrada ou alterada.
///
/// Não representa recusa de credencial pelo backend e não executa logout.
/// Indica que a conclusão não deve ser aplicada à identidade originalmente
/// esperada.
///
/// A conferência de UID deve ser combinada com o contexto e a revisão
/// da operação para distinguir sequências de contas como A/B/A.
final class SessionIdentityChangedException implements Exception {
  /// Cria o sinal de identidade alterada.
  const SessionIdentityChangedException();

  /// Descreve a situação sem incluir identificadores ou credenciais.
  @override
  String toString() {
    return 'A sessão autenticada mudou durante a operação';
  }
}
