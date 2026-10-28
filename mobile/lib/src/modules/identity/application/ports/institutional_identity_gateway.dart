/// Porta de saída para a verificação institucional solicitada pela aplicação.
///
/// Esta biblioteca expressa o que o caso de uso precisa do backend sem incorporar
/// detalhes de endpoint, cabeçalhos, DTOs ou status HTTP ao seu contrato.
/// O adaptador pode mudar sem alterar a coordenação ou a apresentação.
///
/// ## Participação no caso de uso
///
/// InstitutionalAuthService obtém um access token Microsoft e solicita verify.
/// A implementação de infraestrutura providencia a requisição autenticada,
/// converte a resposta e devolve InstitutionalVerification.
///
/// Na composição atual, o ID token Firebase é obtido pelo transporte a partir
/// da mesma autenticação. Ele não é argumento desta porta nem se confunde com
/// o access token Microsoft recebido.
///
/// ## Significado do resultado
///
/// Uma resposta não estudantil é verificação concluída, não uma exceção.
/// Falhas externas e respostas incompatíveis propagam explicitamente, em vez de
/// serem representadas por perfil ausente. Os adaptadores classificam as falhas
/// reconhecidas no vocabulário da aplicação.
///
/// O gateway não decide prontidão, persiste cache ou associa a resposta ao UID.
/// Essas responsabilidades pertencem ao caso de uso e ao controlador.
library;

import '../errors/identity_exception.dart';
import '../models/institutional_verification.dart';

/// Obtém uma verificação institucional para a identidade autenticada.
///
/// A implementação concreta deve utilizar a mesma autenticação observada
/// pela porta de sessão da aplicação.
///
/// A coordenação que chama esta porta confere a identidade antes e depois
/// da operação. Esta porta não armazena o perfil e não publica estados de UI.
abstract interface class InstitutionalIdentityGateway {
  /// Verifica a identidade institucional usando [microsoftAccessToken].
  ///
  /// A credencial deve ser não vazia e pertencer à mesma identidade utilizada
  /// na autenticação da requisição ao backend.
  ///
  /// Retorna um modelo próprio da aplicação. A implementação deve converter
  /// a resposta externa antes de concluir, sem expor DTOs de transporte.
  ///
  /// O resultado de vínculo não estudantil é um sucesso da operação.
  /// Não deve ser convertido em exceção ou ausência de perfil.
  ///
  /// Lança [IdentityException] para falhas externas reconhecidas de
  /// comunicação, rejeição ou conversão da resposta.
  /// Violações de programação continuam propagando como erros.
  ///
  /// Não persiste dados, executa navegação ou garante que a verificação ainda
  /// estará dentro do prazo quando o chamador utilizar o resultado.
  ///
  /// A credencial é sensível e não deve ser persistida no cache institucional
  /// ou incluída em diagnósticos públicos.
  Future<InstitutionalVerification> verify({
    required String microsoftAccessToken,
  });
}
