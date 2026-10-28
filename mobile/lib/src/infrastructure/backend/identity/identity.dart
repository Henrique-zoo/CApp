/// Fachada do protocolo HTTP de identidade institucional.
///
/// Esta biblioteca reúne IdentityApi e os DTOs usados para trocar dados com o
/// backend. Seu lugar é a infraestrutura compartilhada, separado da fachada
/// de aplicação do módulo de identidade, que possui tipos com nomes semelhantes.
///
/// ## Uso pelos adaptadores
///
/// BackendApi disponibiliza o cliente com transporte configurado.
/// O gateway do módulo consome esse cliente, monta o DTO de requisição e converte
/// a resposta para modelos próprios antes de entregá-la ao caso de uso.
///
/// Os tipos exportados representam o protocolo: perfil, vínculo e validade
/// recebidos pela rede. Eles não são catálogo de capacidades, estado de sessão
/// ou entidades de um usuário interno provisionado.
///
/// ## Superfície exportada
///
/// As diretivas show limitam a API aos tipos listados. Extensões geradas de
/// copyWith não são todas reexportadas; seu uso pode exigir importação direta
/// da biblioteca do DTO correspondente.
///
/// Importar a fachada não cria Dio, obtém tokens ou inicia chamadas HTTP.
library;

export 'identity_api.dart' show IdentityApi;
export 'dto/institutional_auth_request.dart' show InstitutionalAuthRequest;
export 'dto/institutional_verification_response.dart'
    show InstitutionalVerificationResponse;
export 'dto/institutional_identity_response.dart'
    show InstitutionalIdentityResponse;
export 'dto/institutional_affiliation.dart'
    show InstitutionalAffiliation, StudentAffiliation, NonStudentAffiliation;
