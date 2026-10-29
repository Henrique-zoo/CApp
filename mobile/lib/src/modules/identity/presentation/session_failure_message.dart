/// Tradução das falhas de sessão para mensagens destinadas ao usuário.
///
/// Os diagnósticos precisam conservar exceção e pilha, mas não são textos de
/// interface. Esta biblioteca centraliza a tradução a partir dos tipos da
/// aplicação, evitando que widgets interpretem SDKs, status HTTP ou JSON.
///
/// ## Seleção da mensagem
///
/// Para IdentityException, uma publicMessage não vazia tem prioridade porque foi
/// explicitamente marcada como pública na fronteira. Na sua ausência, a mensagem
/// vem de IdentityFailureKind. A causa técnica não é convertida em texto público.
///
/// A operação de saída possui mensagens próprias para não orientar um novo
/// login quando o problema foi encerrar a conta. Nas demais operações, a causa
/// classificada tem prioridade sobre o resumo SessionFailure. Uma evidência
/// presente com entrada exigida e sem operação pode indicar expiração.
///
/// ## Limites
///
/// A biblioteca apresenta uma decisão já publicada. Não consulta relógio,
/// reclassifica respostas do backend ou executa recuperação.
/// Retornar nulo significa apenas que não há mensagem a mostrar, sem apagar
/// o diagnóstico ou transformar uma falha operacional em sucesso.
///
/// As mensagens são compartilhadas pela apresentação do módulo; a biblioteca
/// não define o destino global da aplicação.
library;

import '../application/errors/identity_exception.dart';
import '../application/errors/session_cache_write_exception.dart';
import '../application/session_state.dart';

/// Converte uma falha classificada em texto público.
///
/// Prefere a mensagem explicitamente pública do backend. Sem ela, utiliza
/// a classificação da aplicação, nunca a representação da causa técnica.
String identityFailureMessage(IdentityException failure) {
  final message = failure.publicMessage;
  if (message != null && message.trim().isNotEmpty) return message;
  return switch (failure.kind) {
    IdentityFailureKind.authenticationCancelled =>
      'A autenticação foi cancelada. Você pode tentar novamente.',
    IdentityFailureKind.operationCancelled =>
      'A operação foi cancelada. Você pode tentar novamente.',
    IdentityFailureKind.credentialsRejected =>
      'Não foi possível validar suas credenciais. Entre novamente.',
    IdentityFailureKind.institutionNotSupported =>
      'Sua instituição ainda não está cadastrada no CApp.',
    IdentityFailureKind.verificationConflict =>
      'Não foi possível concluir esta verificação. Tente novamente.',
    IdentityFailureKind.communicationFailed =>
      'Não foi possível comunicar com o serviço. Tente novamente.',
    IdentityFailureKind.serviceUnavailable =>
      'O serviço está temporariamente indisponível. Tente mais tarde.',
    IdentityFailureKind.invalidResponse =>
      'Não foi possível validar a resposta do serviço. Tente novamente.',
    IdentityFailureKind.unexpectedExternalFailure =>
      'Não foi possível concluir a entrada. Tente novamente.',
  };
}

/// Obtém a mensagem da falha ou da necessidade de renovar a entrada.
///
/// A falha de saída e a limpeza posterior possuem mensagens específicas.
/// Nas demais operações, a causa classificada tem prioridade sobre o resumo.
/// Retorna nulo apenas quando não há mensagem, sem apagar o diagnóstico.
/// A expiração usa o destino publicado pelo controlador, sem outro relógio.
String? sessionFailureMessage(SessionState state) {
  if (state.failure == SessionFailure.signOutFailed) {
    return 'Não foi possível sair da conta. Tente novamente.';
  }
  if (state.failure == SessionFailure.cacheRemovalFailed) {
    return 'Você saiu da conta, mas não foi possível remover os dados '
        'salvos neste dispositivo.';
  }
  final cause = state.failureDetails?.cause;
  if (cause is IdentityException) return identityFailureMessage(cause);
  if (cause is SessionCacheWriteException) {
    return 'Sua identidade foi verificada, mas não foi possível salvar '
        'os dados neste dispositivo.';
  }
  if (state.failure != null) {
    return switch (state.failure!) {
      SessionFailure.authenticationCancelled =>
        'A autenticação foi cancelada. Você pode tentar novamente.',
      SessionFailure.operationCancelled =>
        'A operação foi cancelada. Você pode tentar novamente.',
      SessionFailure.communicationFailed =>
        'Não foi possível comunicar com o serviço. Tente novamente.',
      SessionFailure.cacheRejected =>
        'Os dados salvos não puderam ser reutilizados. Entre novamente.',
      SessionFailure.cacheReadFailed =>
        'Não foi possível recuperar os dados salvos. Entre novamente.',
      SessionFailure.cacheWriteFailed =>
        'Não foi possível salvar os dados neste dispositivo.',
      SessionFailure.cacheRemovalFailed =>
        'Você saiu da conta, mas não foi possível remover os dados '
            'salvos neste dispositivo.',
      SessionFailure.serviceUnavailable =>
        'O serviço está temporariamente indisponível. Tente mais tarde.',
      SessionFailure.authenticationObservationFailed =>
        'Não foi possível recuperar sua sessão.',
      SessionFailure.signOutFailed => 'Não foi possível encerrar a sessão.',
      _ => 'Não foi possível concluir a entrada. Tente novamente.',
    };
  }
  if (state is SessionSignedIn &&
      state.verification != null &&
      state.access == SessionAccess.entryRequired &&
      !state.isBusy) {
    return 'Sua verificação institucional expirou. '
        'Entre novamente com a Microsoft.';
  }
  return null;
}
