/// Fachada das operações técnicas de monitoramento do backend.
///
/// Esta biblioteca expõe MonitoringApi para quem precisar consultar saúde
/// do processo HTTP ou prontidão de sua conexão com PostgreSQL. Os endpoints
/// são públicos e distintos das operações de identidade ou de outros módulos.
///
/// ## Relação com a composição
///
/// BackendApi cria o cliente sobre seu transporte compartilhado quando a
/// fachada é acessada. A construção é inerte: somente a chamada explícita
/// de uma sondagem produz uma requisição.
///
/// A API técnica não é pré-condição do fluxo de sessão. Falhas de uma operação
/// do domínio são tratadas nessa própria operação, sem depender de uma sondagem
/// anterior.
/// AppDependencies solicita prontidão na abertura para apresentar diagnóstico,
/// em paralelo à sessão e sem condicionar sua restauração ou o login.
///
/// As exportações não instalam monitoramento periódico, verificam configuração
/// de provedores ou concedem acesso ao usuário.
library;

export 'monitoring_api.dart' show MonitoringApi;
