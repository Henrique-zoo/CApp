/// Sinal de que selecionar uma referência salva exige uma senha interna.
///
/// Ausência de sessão SDK ou uma saída protegida levam o controlador a apresentar
/// o formulário da conta selecionada. Este sinal não representa senha incorreta,
/// falha de rede ou necessidade de autorização Microsoft e não contém credenciais.
library;

/// A conta escolhida precisa de autenticação interna explícita.
final class AccountPasswordRequiredException implements Exception {
  const AccountPasswordRequiredException();

  @override
  String toString() => 'AccountPasswordRequiredException';
}
