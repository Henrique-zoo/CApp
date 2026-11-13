/// Porta da aplicação para observar e conferir a identidade autenticada.
///
/// A coordenação da sessão precisa conhecer a conta restaurada e suas mudanças,
/// mas não precisa de User ou FirebaseAuth. Esta biblioteca define somente as
/// operações de autenticação necessárias a essa coordenação.
///
/// ## Fluxo de observação
///
/// watchUserId deve informar o estado restaurado na primeira emissão e depois
/// publicar login, logout e troca de conta. O controlador aguarda essa primeira
/// emissão antes de decidir se consulta o cache. Um valor nulo significa ausência
/// de autenticação; uma falha deve continuar sendo um erro do stream.
///
/// currentUserId permite conferir a conta durante operações assíncronas,
/// inclusive quando a notificação de uma mudança ainda não chegou.
/// O adaptador deve manter coerência entre a consulta e seus eventos.
///
/// ## Fronteira e propriedade
///
/// FirebaseMicrosoftAuth implementa a porta na composição atual. O consumidor
/// possui suas inscrições, enquanto o SDK possui sua autenticação.
/// signOut desativa a seleção atual. O adaptador pode conservar a sessão SDK para
/// entrada rápida posterior. AccountAuthentication distingue exigir senha e
/// esquecer; nenhuma operação revoga automaticamente sessões de outros dispositivos.
///
/// O UID identifica a conta autenticada pelo provedor. Não comprova vínculo
/// institucional, perfil interno ou capacidade para uma operação protegida.
library;

/// Permite acompanhar o usuário autenticado e encerrar sua sessão.
///
/// A implementação concreta adapta os eventos do Firebase sem expor
/// tipos do SDK à camada de aplicação.
/// A implementação deve manter coerência entre os eventos e [currentUserId].
/// O consumidor cancela suas inscrições; esta porta não possui o cache local.
abstract interface class AuthSession {
  /// Observa o UID do usuário autenticado.
  ///
  /// A primeira emissão deve informar o estado restaurado da autenticação.
  /// Emissões posteriores representam login, logout ou troca de usuário.
  ///
  /// Um valor nulo indica ausência de usuário autenticado.
  ///
  /// ## Retorno e falhas
  ///
  /// Stream da identidade restaurada e das mudanças posteriores. A frequência,
  /// o compartilhamento entre ouvintes e as falhas são definidos pelo adaptador.
  /// Uma falha deve ser entregue como erro do stream, sem fabricar um evento nulo.
  /// A implementação Firebase atual não usa este stream para renovar evidência.
  Stream<String?> watchUserId();

  /// Desativa a sessão selecionada no aplicativo.
  ///
  /// Após o sucesso, [watchUserId] deve emitir null.
  ///
  /// O modo de saída e a limpeza do cache pertencem à coordenação do controlador.
  /// Falhas de encerramento são propagadas ao chamador.
  ///
  /// Concluir o Future representa a desativação local informada pelo adaptador.
  /// Não implica revogar tokens já emitidos em todos os dispositivos, remover
  /// evidências no backend ou apagar o cache institucional do aplicativo.
  Future<void> signOut();

  /// UID atualmente autenticado, ou null quando não há autenticação.
  ///
  /// Usado para conferir a identidade durante operações assíncronas.
  /// A restauração inicial continua sendo acompanhada por [watchUserId].
  /// A consulta é síncrona e não realiza uma verificação institucional. Nulo é
  /// ausência de identidade conhecida, não uma conversão de falha em sucesso.
  String? get currentUserId;
}
