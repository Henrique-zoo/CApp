/// Fachada dos DTOs de erro compartilhados pelo protocolo HTTP.
///
/// Esta biblioteca oferece ApiError e ApiErrorDetails aos adaptadores que
/// interpretam respostas do backend. Ela centraliza a representação do envelope
/// sem transformar códigos específicos de um domínio em regras globais de Dio.
///
/// ## Divisão de responsabilidades
///
/// O transporte conserva status, corpo e diagnóstico da requisição.
/// Os DTOs interpretam a estrutura pública desse corpo.
/// O gateway de cada módulo traduz os códigos relevantes para falhas da sua
/// aplicação, preservando causa e pilha para diagnóstico.
///
/// No módulo de identidade, essa tradução pertence a
/// BackendInstitutionalIdentityGateway. Não existe um interceptor global de
/// erros de negócio instalado por esta fachada.
///
/// As exportações apenas disponibilizam tipos; não executam I/O, registram
/// tratamento automático ou apresentam mensagens.
library;

export 'api_error.dart' show ApiError;
export 'api_error_details.dart' show ApiErrorDetails;
