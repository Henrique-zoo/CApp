/// API de apresentação do módulo de identidade.
///
/// Esta fachada permite que a camada app componha a experiência de entrada sem
/// importar o caminho interno de cada tela. A separação da API de aplicação evita
/// misturar contratos de sessão e componentes Flutter em uma única exportação.
///
/// ## Participação na composição visual
///
/// A composição entrega um SessionController iniciado e o estado atual à
/// InstitutionalLoginPage. A tela apresenta o estado e solicita a operação de
/// entrada; o controlador continua responsável por autenticação, backend e cache.
///
/// O destino global é selecionado pelo gate da camada app. Esta fachada não
/// decide qual área montar e não cria controladores ou recursos externos.
///
/// Somente componentes de apresentação são exportados. Adaptadores Firebase,
/// clientes HTTP e armazenamento permanecem fora da API visual do módulo.
library;

export 'presentation/institutional_login_page.dart' show InstitutionalLoginPage;
