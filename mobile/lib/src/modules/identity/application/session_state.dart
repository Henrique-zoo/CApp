/// Contrato observável entre a coordenação da sessão e sua apresentação.
///
/// Esta biblioteca representa a sessão sem associar seus estados a telas, SDKs
/// ou códigos HTTP. SessionController publica novas instâncias, e os consumidores
/// observam essas fotografias para renderizar a interface e indicar atividades.
///
/// ## Dimensões do estado
///
/// Os tipos distinguem restauração, ausência de autenticação, identidade conhecida
/// e falha de acompanhamento. SessionSignedIn significa que existe uma conta
/// autenticada; sua verificação institucional ainda pode estar ausente ou expirada.
///
/// SessionAccess publica o destino calculado pelo controlador. SessionOperation
/// indica a atividade em andamento e permite bloquear interações repetidas.
/// Essas dimensões não devem ser inferidas apenas pelo tipo SessionSignedIn.
///
/// ## Falhas e validade
///
/// SessionFailure oferece uma classificação para apresentação.
/// SessionFailureDetails conserva a exceção e a pilha correspondentes; o resumo
/// não substitui o diagnóstico original nem o erro propagado pela operação.
///
/// Os modelos não consultam relógio e não executam efeitos colaterais. O prazo é
/// avaliado pelo controlador, enquanto autenticação e autorização efetivas de
/// requisições continuam pertencendo ao backend. Durante o logout, a operação
/// permanece ocupada até terminar a tentativa de limpeza local, mesmo depois
/// de o provedor informar ausência da conta.
library;

import 'models/institutional_verification.dart';
import 'session_cache_result.dart';

/// Destino local da interface, calculado pelo controlador da sessão.
///
/// É uma fotografia da política de entrada. Não substitui a autorização dos
/// endpoints e não consulta o relógio quando lida pela apresentação.
enum SessionAccess {
  /// Autenticação ou cache inicial ainda estão sendo restaurados.
  restoring,

  /// É necessário concluir a operação de entrada Microsoft e backend.
  entryRequired,

  /// Existe autenticação e verificação institucional dentro do prazo local.
  ready,

  /// Não foi possível determinar ou acompanhar a autenticação.
  restorationFailed,
}

/// Diagnóstico original de uma operação que falhou na sessão atual.
///
/// A apresentação usa a classificação pública da causa quando disponível.
/// A causa e a pilha são preservadas para diagnóstico, sem registro automático.
final class SessionFailureDetails {
  /// Preserva [cause] e sua [stackTrace] sem convertê-las em texto público.
  const SessionFailureDetails({required this.cause, required this.stackTrace});

  /// Exceção original; pode conter credenciais ou dados pessoais em sua cadeia.
  final Exception cause;

  /// Pilha original capturada durante a operação.
  final StackTrace stackTrace;
}

/// Operação atualmente coordenada pelo controlador da sessão.
///
/// A atividade é independente do destino: uma conta já deslogada pode continuar
/// em signingOut enquanto a coordenação tenta remover seu cache local.
enum SessionOperation {
  /// Nenhuma operação em andamento.
  idle,

  /// Aguardando a restauração inicial da autenticação.
  restoringAuthentication,

  /// Autenticação com o provedor Microsoft.
  signingIn,

  /// Criando ou renovando a evidência institucional.
  verifyingInstitution,

  /// Encerrando a sessão.
  signingOut,

  /// Recuperando o cache do usuário autenticado.
  restoringCache,
}

/// Classificação das falhas apresentáveis pelo fluxo de sessão.
///
/// A apresentação transforma essas categorias em mensagens.
/// Exceções técnicas não devem ser exibidas diretamente ao usuário.
/// O controlador classifica as falhas normalizadas dos adaptadores e preserva
/// sua causa em SessionFailureDetails. Falha de limpeza após logout não implica
/// que a autenticação permaneça ativa.
enum SessionFailure {
  /// Uma operação externa foi cancelada, sem implicar logout.
  operationCancelled,

  /// Não foi possível concluir a comunicação com uma dependência.
  communicationFailed,

  /// O usuário cancelou a interação com o provedor.
  authenticationCancelled,

  /// Não foi possível concluir a autenticação.
  authenticationFailed,

  /// Não foi possível concluir a verificação institucional.
  institutionalVerificationFailed,

  /// Uma dependência está temporariamente indisponível.
  serviceUnavailable,

  /// Não foi possível concluir o logout.
  signOutFailed,

  /// Ocorreu uma falha sem classificação específica.
  unexpected,

  /// Não foi possível acompanhar ou restaurar a autenticação.
  authenticationObservationFailed,

  /// O registro local existe, mas seu conteúdo foi rejeitado.
  cacheRejected,

  /// Não foi possível acessar o armazenamento local.
  cacheReadFailed,

  /// A verificação foi concluída, mas não pôde ser persistida.
  cacheWriteFailed,

  /// A autenticação foi encerrada, mas a remoção do cache local falhou.
  cacheRemovalFailed,
}

/// Estado observável do fluxo de sessão.
///
/// Cada transição produz uma nova instância. O estado não executa
/// autenticação, persistência ou navegação.
/// Os construtores não validam todas as combinações de operação e variante;
/// a coerência das transições pertence ao produtor dos estados.
sealed class SessionState {
  /// Define o resumo operacional, inicialmente ocioso e sem falha.
  const SessionState({
    this.access = SessionAccess.entryRequired,
    this.operation = SessionOperation.idle,
    this.failure,
    this.failureDetails,
  });

  /// Destino de apresentação publicado pelo controlador.
  ///
  /// A validade é reavaliada pelo produtor dos estados, incluindo expiração
  /// e retomada. A leitura desta propriedade não executa efeitos colaterais.
  final SessionAccess access;

  /// Operação em andamento.
  final SessionOperation operation;

  /// Falha associada à última operação, quando houver.
  ///
  /// Sua presença não implica que a autenticação foi encerrada.
  /// É uma categoria de apresentação, não um repositório da causa técnica.
  final SessionFailure? failure;

  /// Diagnóstico da operação associada a [failure], quando existente.
  ///
  /// Falhas de leitura do cache também preservam seus detalhes no resultado
  /// específico da leitura. A ausência deste campo não apaga esse diagnóstico.
  final SessionFailureDetails? failureDetails;

  /// Indica se existe uma operação em andamento.
  /// Deriva exclusivamente de [operation], sem consultar tarefas ou streams.
  bool get isBusy => operation != SessionOperation.idle;
}

/// Aguardando o primeiro estado restaurado da autenticação.
///
/// Enquanto este estado estiver ativo, o aplicativo ainda não sabe
/// se existe um usuário autenticado.
final class SessionRestoring extends SessionState {
  /// Inicia a fotografia local antes do primeiro evento de autenticação.
  const SessionRestoring()
    : super(
        access: SessionAccess.restoring,
        operation: SessionOperation.restoringAuthentication,
      );
}

/// Não existe um usuário autenticado.
///
/// Durante uma tentativa de login, continua representando a ausência
/// da autenticação, com a operação [SessionOperation.signingIn].
final class SessionSignedOut extends SessionState {
  /// Representa ausência de identidade, com eventual operação ou resumo de falha.
  const SessionSignedOut({
    super.operation,
    super.failure,
    super.failureDetails,
  });
}

/// Existe um usuário autenticado no Firebase.
///
/// A evidência institucional é opcional e independente da autenticação.
/// Ela pode estar ausente, expirada ou representar um não estudante.
/// `NonStudent` informa o resultado da consulta institucional, sem definir um
/// papel geral de autorização ou exigir encerramento da autenticação.
final class SessionSignedIn extends SessionState {
  /// Agrupa a identidade e os dados locais conhecidos.
  ///
  /// Não confere o proprietário do perfil, não avalia prazo e não consulta o
  /// servidor. O controlador e o armazenamento fazem essas associações.
  /// access é calculado pelo controlador; SessionAccess.ready exige uma
  /// verificação presente e válida no instante da publicação.
  const SessionSignedIn({
    required this.firebaseUid,
    this.verification,
    this.cacheLoadResult,
    super.access,
    super.operation,
    super.failure,
    super.failureDetails,
  });

  /// UID ao qual pertencem os dados deste estado.
  final String firebaseUid;

  /// Última verificação institucional conhecida para esta conta.
  ///
  /// Pode ter sido recebida do backend ou recuperada do armazenamento.
  /// Ausência e expiração não significam, isoladamente, logout do Firebase.
  ///
  /// O modelo não contém informações do protocolo HTTP.
  final InstitutionalVerification? verification;

  /// Resultado da última leitura do cache para este usuário.
  ///
  /// Preserva motivos de rejeição e causas de falhas de acesso.
  /// Nulo significa que ainda não há resultado de leitura neste estado.
  final SessionCacheLoadResult? cacheLoadResult;

  /// Verifica se a evidência conhecida ainda está dentro do prazo local.
  ///
  /// No instante exato de expiração, retorna false.
  ///
  /// O instante é recebido como argumento para permitir testes
  /// determinísticos e evitar dependência interna do relógio.
  ///
  /// Este resultado auxilia a interface. O backend continua validando
  /// a evidência e autorizando cada operação protegida.
  ///
  /// ## Retorno
  ///
  /// `false` se não há evidência ou se `expiresAt` é anterior ou igual a [instant].
  /// A comparação de [DateTime] considera o instante, inclusive entre fusos.
  /// O resultado não muda sozinho com o relógio: o chamador deve consultar de novo
  /// quando necessário. Não exige vínculo `student`.
  ///
  /// ## Exemplo
  ///
  /// ```dart
  /// final session = SessionSignedIn(firebaseUid: 'uid-de-exemplo');
  /// assert(!session.hasUnexpiredVerificationAt(DateTime.utc(2026)));
  /// ```
  bool hasUnexpiredVerificationAt(DateTime instant) {
    final knownVerification = verification;

    return knownVerification != null && knownVerification.isValidAt(instant);
  }
}

/// Não foi possível determinar ou acompanhar a autenticação.
///
/// Não significa que o usuário realizou logout.
/// A autenticação do SDK pode continuar existindo; este estado informa falha em
/// observá-la e preserva o diagnóstico para recuperação e investigação.
final class SessionRestorationFailed extends SessionState {
  /// Cria a falha de observação sem fabricar uma identidade ou executar logout.
  const SessionRestorationFailed({
    required this.cause,
    required this.stackTrace,
  }) : super(
         access: SessionAccess.restorationFailed,
         failure: SessionFailure.authenticationObservationFailed,
       );

  /// Exceção original recebida do acompanhamento da autenticação.
  final Exception cause;

  /// Pilha original da falha.
  final StackTrace stackTrace;
}
