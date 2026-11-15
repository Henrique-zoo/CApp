/// Perfil interno que permite retomar o provisionamento sem depender do Graph.
///
/// E-mail de login e dados acadêmicos escolhidos pelo usuário são independentes
/// do perfil institucional. O backend decide o estado; a apresentação apenas
/// escolhe entre concluir o cadastro e entrar no aplicativo.
/// O complemento de nome é uma declaração para uma ausência do Graph. A
/// apresentação dá prioridade ao dado externo e identifica o complemento;
/// o nome declarado não comprova um vínculo. O e-mail do Graph pode acompanhar
/// o resultado do cadastro como sugestão transitória, sem integrar este perfil.
///
/// ## Participação no fluxo
///
/// O gateway transforma o DTO HTTP em [InternalUser], e o armazenamento local
/// reconstitui a mesma classe ao ler o cache. A fábrica aplica as invariantes
/// comuns às duas entradas para evitar que uma fotografia incompleta seja tratada
/// como conta ativa. [AccountSession] agrega a evidência e a sugestão transitória
/// recebidas na operação; somente a conta e a evidência têm representação no cache.
///
/// [ProfileCompletion] leva o formulário ao caso de uso depois de configurar a
/// credencial Firebase. Ele não escolhe UID ou estado de conta: o backend confere
/// origem, validade e conteúdo e devolve a nova fotografia. Nenhum destes tipos
/// executa autenticação, persistência, navegação ou consulta ao Graph.
library;

import 'package:uuid/uuid.dart';

import 'institutional_verification.dart';

/// Estado da conta CApp, distinto da validade institucional e da sessão Firebase.
///
/// `incomplete` permite retomar o cadastro; `active` representa perfil concluído;
/// `inactive` representa a desativação que o login não pode desfazer. A validade
/// da evidência e a existência da credencial interna são verificadas separadamente.
enum InternalUserStatus { incomplete, active, inactive }

/// Fotografia da conta provisionada. Não contém senha nem tokens.
final class InternalUser {
  /// Rejeita valores inconsistentes antes de expor a fotografia à aplicação.
  ///
  /// [id] e [status] identificam a conta retornada pelo backend. Os demais dados
  /// podem estar ausentes enquanto ela estiver incompleta. Um período informado
  /// deve ser maior que zero e um ingresso informado deve satisfazer
  /// [isValidInstitutionEntrySemester], independentemente do estado.
  ///
  /// Para [InternalUserStatus.active], exige [nickname] não vazio após `trim`,
  /// [email] não vazio, [currentSemester] e [institutionEntrySemester]. [name] é
  /// um complemento opcional quando o Graph não forneceu nome. O UUID já deve ter
  /// sido validado na fronteira; o texto dos demais campos é preservado, sem
  /// normalização, consulta ao Firebase ou verificação sintática do e-mail.
  ///
  /// Lança [FormatException] quando a fotografia viola essas invariantes. A mesma
  /// validação é usada pelo gateway e pelo cache; ela não comprova que o perfil
  /// veio do backend nem substitui a autorização das operações no servidor.
  ///
  /// Exemplo de uma conta provisionada que ainda deve retomar o formulário:
  ///
  /// ```dart
  /// final user = InternalUser(
  ///   id: UuidValue.withFormatValidation('00000000-0000-0000-0000-000000000010'),
  ///   status: InternalUserStatus.incomplete,
  /// );
  /// assert(user.nickname == null);
  /// ```
  factory InternalUser({
    required UuidValue id,
    required InternalUserStatus status,
    String? nickname,
    String? email,
    int? currentSemester,
    String? institutionEntrySemester,
    String? name,
  }) {
    if ((currentSemester != null && currentSemester < 1) ||
        (institutionEntrySemester != null &&
            !isValidInstitutionEntrySemester(institutionEntrySemester)) ||
        (status == InternalUserStatus.active &&
            (nickname == null ||
                nickname.trim().isEmpty ||
                email == null ||
                email.isEmpty ||
                currentSemester == null ||
                institutionEntrySemester == null))) {
      throw const FormatException('Perfil interno inconsistente');
    }

    return InternalUser._(
      id: id,
      status: status,
      nickname: nickname,
      email: email,
      currentSemester: currentSemester,
      institutionEntrySemester: institutionEntrySemester,
      name: name,
    );
  }

  /// Guarda os valores que já passaram pela fábrica, sem repetir sua validação.
  const InternalUser._({
    required this.id,
    required this.status,
    this.nickname,
    this.email,
    this.currentSemester,
    this.institutionEntrySemester,
    this.name,
  });

  /// Identificador interno estável, distinto do UID Firebase e do objeto Microsoft.
  final UuidValue id;

  /// Situação do perfil determinada pelo backend, não pelo cache ou formulário.
  final InternalUserStatus status;

  /// Apelido público escolhido; ausente até a conclusão do perfil.
  final String? nickname;

  /// Endereço escolhido para a credencial interna e confirmado pelo backend.
  /// A sugestão do Graph não é armazenada neste campo antes dessa escolha.
  final String? email;

  /// Nome declarado para uma ausência do Graph; não é uma comprovação.
  final String? name;

  /// Período atual do curso, a partir de 1.
  final int? currentSemester;

  /// Semestre de ingresso na instituição, no formato AAAA.1 ou AAAA.2.
  final String? institutionEntrySemester;
}

/// Resposta autenticada que dispensa Microsoft enquanto a evidência estiver válida.
///
/// A associação ao UID da autenticação atual é conferida pelo caso de uso e pelo
/// controlador. O envelope não cria uma sessão no backend nem decide se a conta
/// está pronta: perfil, credencial interna e validade continuam sendo dimensões
/// distintas. A sugestão transitória não participa da evidência reutilizável.
final class AccountSession {
  /// Agrupa a fotografia da conta e o resultado institucional recebido.
  ///
  /// [verification] nula permite devolver a conta sem inventar uma confirmação;
  /// a coordenação solicitará sincronização. [suggestedEmail] só deve acompanhar
  /// uma verificação de cadastro incompleto, e não uma restauração de sessão.
  /// O construtor não confere origem, propriedade ou expiração e não grava dados.
  const AccountSession({
    required this.user,
    required this.verification,
    this.suggestedEmail,
  });

  /// Conta conhecida no backend, inclusive quando ainda está incompleta.
  final InternalUser user;

  /// Nulo quando a evidência precisa de sincronização institucional.
  final InstitutionalVerification? verification;

  /// Sugestão transitória do Graph para concluir o cadastro; nunca vai ao cache.
  /// A restauração de sessão não fornece esse campo e contas ativas o ignoram.
  final String? suggestedEmail;
}

/// Dados necessários para concluir o perfil, sem UID ou estado enviados pelo cliente.
///
/// É o formulário da aplicação, não um DTO HTTP ou uma entidade persistida. O
/// gateway escolhe a representação de transporte. A senha é tratada pelo Firebase
/// antes da chamada ao backend, que obtém o e-mail escolhido do token validado.
final class ProfileCompletion {
  /// Captura os dados informados sem validar, normalizar ou efetuar I/O.
  ///
  /// [nickname], [currentSemester] e [institutionEntrySemester] são obrigatórios
  /// no cadastro. [name] só deve ser enviado para completar uma ausência do Graph;
  /// o backend recusa sua substituição quando o diretório já informou um nome.
  /// A apresentação valida o formulário, e a validação autoritativa é do backend.
  const ProfileCompletion({
    required this.nickname,
    required this.currentSemester,
    required this.institutionEntrySemester,
    this.name,
  });

  /// Apelido escolhido, cujo conteúdo não vazio é exigido na conclusão.
  final String nickname;

  /// Período atual do curso; não é a quantidade de semestres desde o ingresso.
  final int currentSemester;

  /// Primeiro ingresso na instituição em AAAA.1/AAAA.2, inclusive se houve pausas.
  final String institutionEntrySemester;

  /// Nome enviado somente quando ausente no Graph; não altera a evidência.
  final String? name;
}

/// Valida o semestre institucional no formato AAAA.1 ou AAAA.2.
///
/// [value] deve conter quatro algarismos, ponto e `1` ou `2`, sem espaços extras.
/// Retorna `false` para outro formato; não aplica `trim`, verifica se o ano já
/// ocorreu ou confronta o calendário acadêmico. Assim, é compartilhada pela
/// reconstituição do perfil e pelo formulário sem depender de relógio ou serviço.
///
/// ```dart
/// assert(isValidInstitutionEntrySemester('2024.1'));
/// assert(!isValidInstitutionEntrySemester('2024.3'));
/// ```
bool isValidInstitutionEntrySemester(String value) =>
    RegExp(r'^\d{4}\.[12]$').hasMatch(value);
