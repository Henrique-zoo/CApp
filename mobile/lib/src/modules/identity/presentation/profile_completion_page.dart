/// Cadastro em uma única tela, com duas etapas no mesmo formulário.
///
/// A etapa inicial apresenta nome e curso do Graph, além
/// da instituição identificada. Valores externos não nulos são somente leitura;
/// a ausência de nome pode receber um complemento declarado pelo usuário,
/// sem alterar a evidência ou o vínculo. O e-mail do Graph é apenas sugestão
/// transitória para o campo de acesso na segunda etapa, sem outro campo institucional.
/// Curso nunca é declarado: sua ausência mantém um aviso de acesso limitado
/// durante as duas etapas, sem impedir a conclusão do perfil interno.
/// Apelido, período atual e ingresso são os dados acadêmicos obrigatórios.
///
/// Avançar valida os dados e troca o conteúdo do cartão, sem abrir outra rota ou
/// ativar a conta. A segunda etapa configura e-mail/senha no mesmo UID Firebase
/// e envia o perfil ao backend. Voltar preserva os dados e descarta as senhas.
/// Uma falha permite repetir a conclusão; o SessionGate libera a área principal
/// somente após o aceite do backend e descarta o formulário ao trocar de conta.
///
/// O ingresso sugerido conta o período corrente e permanece editável. Uma edição
/// manual não é sobrescrita por mudanças do período. O rascunho vive apenas nesta
/// tela; sair preserva a conta incompleta, sem gravar campos ainda não enviados.
library;

import 'package:flutter/material.dart';

import '../identity.dart';
import 'account_sign_out_dialog.dart';
import 'institution_entry_suggestion.dart';
import 'limited_student_access_notice.dart';
import 'session_failure_message.dart';

enum _ProfileStep { academic, credentials }

/// Apresenta o perfil institucional e configura o acesso sem mudar de rota.
class ProfileCompletionPage extends StatefulWidget {
  /// Apresenta o cadastro da conta descrita por [state] e coordenada por [controller].
  ///
  /// A composição deve fornecer o ramo de perfil pendente, com confirmação
  /// institucional disponível, e substituir a página ao trocar de identidade.
  /// O usuário interno ainda pode estar em recuperação; nesse caso, o formulário
  /// aguarda a fotografia e oferece nova tentativa. [now] só orienta a sugestão
  /// de ingresso, permitindo testes determinísticos sem alterar a validade da sessão.
  const ProfileCompletionPage({
    super.key,
    required this.controller,
    required this.state,
    this.now = DateTime.now,
  });

  /// Coordenação existente; a página não cria nem descarta esse recurso.
  final SessionController controller;

  /// Fotografia atual da mesma conta do controlador, usada para renderizar o fluxo.
  final SessionSignedIn state;

  /// Relógio apenas da sugestão acadêmica, sem participar da validade da sessão.
  final DateTime Function() now;

  /// Cria o rascunho local e os controladores que pertencem somente a esta página.
  @override
  State<ProfileCompletionPage> createState() => _ProfileCompletionPageState();
}

class _ProfileCompletionPageState extends State<ProfileCompletionPage> {
  final _academicForm = GlobalKey<FormState>();
  final _credentialsForm = GlobalKey<FormState>();
  final _scroll = ScrollController();
  final _name = TextEditingController();
  final _nickname = TextEditingController();
  final _semester = TextEditingController();
  final _entry = TextEditingController();
  final _email = TextEditingController();
  final _password = TextEditingController();
  final _confirm = TextEditingController();
  var _step = _ProfileStep.academic;
  bool _entryWasEdited = false;
  bool _credentialsOpened = false;
  String? _message;

  /// Inicializa o rascunho com a conta e a evidência recebidas, sem ler preferências.
  @override
  void initState() {
    super.initState();
    _loadAcademicData();
    _loadGraphData();
  }

  /// Recupera os campos internos para retomada do cadastro da mesma conta.
  ///
  /// Um ingresso já informado é tratado como escolha do usuário e não é
  /// sobrescrito por sugestões. Na ausência dele, tenta sugerir o semestre a
  /// partir do período atual. Apenas controladores locais são modificados.
  void _loadAcademicData() {
    final user = widget.state.user;

    _nickname.text = user?.nickname ?? '';
    _semester.text = user?.currentSemester?.toString() ?? '';
    _entry.text = user?.institutionEntrySemester ?? '';
    _entryWasEdited = _entry.text.isNotEmpty;

    _suggestEntry();
  }

  /// Preenche o nome com prioridade para o Graph e, depois, para o complemento salvo.
  ///
  /// Ausência nas duas fontes deixa o campo vazio para preenchimento opcional.
  /// Não recupera e-mail institucional ou curso declarado do perfil interno; o
  /// curso é somente leitura na evidência e a sugestão de e-mail fica no estado.
  void _loadGraphData() {
    final profile = widget.state.verification?.profile;

    _name.text = profile?.name ?? widget.state.user?.name ?? '';
  }

  /// Incorpora a chegada do perfil e mudanças da evidência sem refazer toda a etapa.
  ///
  /// Recupera os dados acadêmicos quando a conta antes ausente chega. O nome é
  /// recarregado somente se o perfil institucional mudou; mudanças meramente
  /// operacionais, como início/fim de uma tentativa, preservam o rascunho local.
  @override
  void didUpdateWidget(covariant ProfileCompletionPage oldWidget) {
    super.didUpdateWidget(oldWidget);

    if (oldWidget.state.user == null && widget.state.user != null) {
      _loadAcademicData();
    }

    final before = oldWidget.state.verification?.profile;
    final after = widget.state.verification?.profile;

    // Atualizações operacionais não descartam complementos ainda não enviados.
    if (before != after) _loadGraphData();
  }

  /// Descarta os recursos locais, inclusive os controladores com senhas digitadas.
  /// O controlador de sessão continua pertencendo à composição.
  @override
  void dispose() {
    _scroll.dispose();

    for (final controller in [
      _name,
      _nickname,
      _semester,
      _entry,
      _email,
      _password,
      _confirm,
    ]) {
      controller.dispose();
    }

    super.dispose();
  }

  /// Atualiza a sugestão civil de ingresso enquanto ainda não houve escolha manual.
  ///
  /// Usa o período corrente inclusive, pelo relógio injetado no widget. Período
  /// inválido ou sugestão não representável deixa o campo vazio. Um valor editado
  /// ou recuperado da conta é preservado; a sugestão não valida matrícula real.
  void _suggestEntry() {
    if (_entryWasEdited) return;

    final value = int.tryParse(_semester.text);

    _entry.text = value == null
        ? ''
        : suggestInstitutionEntrySemester(
                currentSemester: value,
                at: widget.now(),
              ) ??
              '';
  }

  /// Autoriza ações locais apenas para o cadastro atual, ocioso e do mesmo UID.
  ///
  /// Consulta o estado mais recente do controlador para impedir que uma tela
  /// antiga envie dados depois de troca de conta, saída ou mudança de destino.
  /// A decisão de acesso foi calculada pela coordenação; não é autorização HTTP.
  bool get _canInteract {
    final state = widget.controller.state;

    return state is SessionSignedIn &&
        !state.isBusy &&
        state.firebaseUid == widget.state.firebaseUid &&
        state.access == SessionAccess.profileRequired;
  }

  /// Valida a etapa acadêmica e apresenta as credenciais sem concluir a conta.
  ///
  /// Com senha já vinculada, usa o e-mail interno atual. No primeiro avanço sem
  /// senha, prefere o endereço da conta ou a sugestão transitória recebida; depois,
  /// preserva a edição do usuário ao voltar/avançar. Dados inválidos ou estado
  /// incompatível impedem a mudança. Não chama Firebase ou backend.
  void _continue() {
    if (!_canInteract || !_academicForm.currentState!.validate()) return;

    if (widget.controller.hasInternalPassword) {
      _email.text = widget.controller.internalEmail ?? _email.text;
    } else if (!_credentialsOpened) {
      _email.text =
          widget.state.user?.email ?? widget.state.suggestedEmail ?? '';
    }

    _credentialsOpened = true;

    _changeStep(_ProfileStep.credentials);
  }

  /// Retorna à etapa acadêmica, conservando dados e e-mail e apagando as senhas.
  /// A operação só é admitida enquanto esta página ainda pode interagir.
  void _back() {
    if (!_canInteract) return;

    _password.clear();
    _confirm.clear();
    _changeStep(_ProfileStep.academic);
  }

  /// Troca para [step] no mesmo formulário e limpa somente o diagnóstico local.
  ///
  /// Remove o foco e retorna a rolagem ao início quando ela está anexada. Não
  /// abre outra rota, salva rascunho ou modifica o estado da conta no backend.
  void _changeStep(_ProfileStep step) {
    FocusScope.of(context).unfocus();
    setState(() {
      _step = step;
      _message = null;
    });

    if (_scroll.hasClients) _scroll.jumpTo(0);
  }

  /// Converte [value] em complemento aparado ou nulo para omissão no transporte.
  /// A admissibilidade de preencher o nome continua sendo conferida pelo backend.
  String? _optional(String value) => value.trim().isEmpty ? null : value.trim();

  /// Encaminha um formulário de credenciais válido à conclusão coordenada da conta.
  ///
  /// Exige a segunda etapa montada e a mesma identidade ainda em cadastro. O
  /// controlador trata a credencial Firebase e, depois, envia os dados acadêmicos;
  /// o nome só acompanha o perfil quando a evidência não o informou. Curso e
  /// contato institucional não são enviados como complementos.
  ///
  /// Falhas esperadas viram mensagem pública nesta página; o estado continua na
  /// coordenação para repetição. Depois de uma tentativa validada, os campos de
  /// senha são sempre limpos no `finally`, inclusive em erro. A página não anuncia
  /// sucesso antes do backend nem tenta usar seu contexto após ser descartada.
  Future<void> _submit() async {
    if (!_canInteract || !_credentialsForm.currentState!.validate()) return;

    final profile =
        (widget.controller.state as SessionSignedIn).verification!.profile;

    setState(() => _message = null);

    try {
      await widget.controller.completeProfile(
        email: _email.text.trim(),
        password: _password.text,
        profile: ProfileCompletion(
          nickname: _nickname.text.trim(),
          currentSemester: int.parse(_semester.text),
          institutionEntrySemester: _entry.text,
          name: profile.name == null ? _optional(_name.text) : null,
        ),
      );
    } on Exception catch (error) {
      if (mounted) {
        setState(
          () => _message =
              error is IdentityException && error.publicMessage != null
              ? error.publicMessage
              : 'Não foi possível concluir o perfil. Tente novamente.',
        );
      }
    } finally {
      if (mounted) {
        _password.clear();
        _confirm.clear();
      }
    }
  }

  /// Solicita a saída preservando a distinção entre continuar depois e escolher modo.
  ///
  /// [chooseMode] abre a confirmação de saída, oferecendo exigência de senha apenas
  /// se ela já existe. Sem seleção de modo, usa entrada rápida para retomar a conta
  /// incompleta. Cancelamento, descarte do widget ou outra operação não inicia saída.
  /// O rascunho não enviado é local e não é persistido por esta ação.
  ///
  /// Aguarda o controlador encerrar o fluxo. Se uma falha ocorrer com a página
  /// ainda montada, apresenta uma mensagem, conservando no controlador o resultado
  /// efetivo da tentativa de autenticação/limpeza.
  Future<void> _leave({bool chooseMode = false}) async {
    if (widget.controller.state.isBusy) return;

    final mode = chooseMode
        ? await confirmAccountSignOut(
            context,
            canRequirePassword: widget.controller.hasInternalPassword,
          )
        : AccountSignOutMode.keepSession;

    if (mode == null || !mounted) return;

    try {
      await widget.controller.signOut(mode: mode);
    } on Exception {
      if (mounted) {
        setState(
          () =>
              _message = 'Não foi possível concluir a saída. Tente novamente.',
        );
      }
    }
  }

  /// Compõe a etapa atual, o aviso de acesso limitado e os diagnósticos da sessão.
  ///
  /// [context] fornece tema e navegação locais. A construção só apresenta a
  /// fotografia recebida: operações externas são iniciadas pelos callbacks.
  /// A ausência do usuário interno apresenta recuperação em vez de um formulário
  /// pronto. A mudança de etapa preserva a rota e os controladores desta página.
  @override
  Widget build(BuildContext context) {
    final busy = widget.state.isBusy;
    final academic = _step == _ProfileStep.academic;
    final message = _message ?? sessionFailureMessage(widget.state);
    final base = Theme.of(context);
    final purple = ColorScheme.fromSeed(
      seedColor: const Color(0xFF8B5CF6),
      brightness: Brightness.dark,
    );

    return PopScope(
      canPop: academic && !busy,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop && !academic) _back();
      },
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Criar conta'),
          leading: academic ? null : BackButton(onPressed: _back),
          actions: [
            TextButton(
              onPressed: busy ? null : () => _leave(chooseMode: true),
              child: const Text('Sair'),
            ),
          ],
        ),
        body: SafeArea(
          child: Center(
            child: SingleChildScrollView(
              controller: _scroll,
              padding: const EdgeInsets.all(24),
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 560),
                child: Theme(
                  data: base.copyWith(
                    colorScheme: purple,
                    textTheme: base.textTheme.apply(
                      bodyColor: purple.onSurface,
                      displayColor: purple.onSurface,
                    ),
                    inputDecorationTheme: InputDecorationTheme(
                      filled: true,
                      fillColor: const Color(0xFF34214F),
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                    ),
                  ),
                  child: Card(
                    color: const Color(0xFF241633),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(24),
                      side: const BorderSide(color: Color(0xFF7550A0)),
                    ),
                    child: Padding(
                      padding: const EdgeInsets.all(24),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          Text(academic ? 'Etapa 1 de 2' : 'Etapa 2 de 2'),
                          const SizedBox(height: 8),
                          LinearProgressIndicator(value: academic ? 0.5 : 1),
                          const SizedBox(height: 24),
                          if (widget.state.verification?.profile.affiliation
                              is NonStudentAffiliation) ...[
                            const LimitedStudentAccessNotice(),
                            const SizedBox(height: 16),
                          ],
                          if (widget.state.user == null) ...[
                            const Text(
                              'Precisamos recuperar o perfil da sua conta antes de continuar.',
                            ),
                            TextButton(
                              onPressed: busy
                                  ? null
                                  : widget.controller.retryAccountSession,
                              child: const Text('Tentar novamente'),
                            ),
                          ] else if (academic)
                            _buildAcademicForm(busy)
                          else
                            _buildCredentialsForm(busy),
                          TextButton(
                            onPressed: busy ? null : _leave,
                            child: const Text('Continuar depois'),
                          ),
                          if (message != null)
                            Semantics(liveRegion: true, child: Text(message)),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  /// Mantém a diferença entre dados da instituição e complementos declarados.
  ///
  /// [busy] desativa entradas e avanço. Nome externo é somente leitura e sua
  /// ausência pode ser completada; curso confirmado é sempre inalterável. Os
  /// validadores locais exigem apelido, período e ingresso antes da segunda etapa,
  /// sem substituir a confirmação de vínculo ou as regras autoritativas do backend.
  Widget _buildAcademicForm(bool busy) {
    final verification = widget.state.verification!;
    final profile = verification.profile;
    final affiliation = profile.affiliation;

    return Form(
      key: _academicForm,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text(
            'Dados institucionais',
            style: TextStyle(fontSize: 22, fontWeight: FontWeight.w600),
          ),
          const SizedBox(height: 8),
          const Text(
            'Confira seus dados. Os valores fornecidos pela instituição não podem ser alterados. Nome ausente pode ser completado; o curso é confirmado somente pela instituição.',
          ),
          const SizedBox(height: 24),
          _graphField(
            label: 'Nome completo',
            controller: _name,
            readOnly: profile.name != null,
            busy: busy,
          ),
          const SizedBox(height: 12),
          TextFormField(
            key: ValueKey(verification.institution.name),
            initialValue: verification.institution.name,
            readOnly: true,
            enabled: !busy,
            decoration: const InputDecoration(
              labelText: 'Instituição',
              suffixIcon: Icon(Icons.lock_outline),
            ),
          ),
          if (affiliation is StudentAffiliation) ...[
            const SizedBox(height: 12),
            TextFormField(
              initialValue: affiliation.courseName,
              readOnly: true,
              enabled: !busy,
              decoration: const InputDecoration(
                labelText: 'Curso',
                suffixIcon: Icon(Icons.lock_outline),
              ),
            ),
          ],
          const SizedBox(height: 8),
          if (affiliation is StudentAffiliation)
            const Text('Vínculo estudantil confirmado.'),
          const SizedBox(height: 24),
          TextFormField(
            controller: _nickname,
            enabled: !busy,
            decoration: const InputDecoration(labelText: 'Apelido'),
            validator: (value) => value == null || value.trim().isEmpty
                ? 'Informe um apelido.'
                : null,
          ),
          const SizedBox(height: 12),
          TextFormField(
            controller: _semester,
            enabled: !busy,
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
              labelText: 'Período atual do curso',
            ),
            onChanged: (_) => _suggestEntry(),
            validator: (value) => (int.tryParse(value ?? '') ?? 0) < 1
                ? 'Informe um período a partir de 1.'
                : null,
          ),
          const SizedBox(height: 12),
          TextFormField(
            controller: _entry,
            enabled: !busy,
            decoration: const InputDecoration(
              labelText: 'Semestre de ingresso na instituição',
              hintText: '2024.1',
              helperText: 'Sugestão editável. Ajuste se houve pausas ou mudança de curso.',
              helperMaxLines: 2,
            ),
            onChanged: (_) => _entryWasEdited = true,
            validator: (value) => !isValidInstitutionEntrySemester(value ?? '')
                ? 'Use o formato AAAA.1 ou AAAA.2.'
                : null,
          ),
          const SizedBox(height: 24),
          FilledButton(
            onPressed: busy ? null : _continue,
            child: const Text('Continuar'),
          ),
        ],
      ),
    );
  }

  /// Monta um campo de dado externo com distinção entre leitura e edição permitida.
  ///
  /// [controller] pertence ao rascunho da página; [readOnly] preserva o valor Graph
  /// e indica bloqueio, enquanto [busy] desativa a interação durante uma operação.
  /// O campo não decide sua origem nem valida o vínculo; o consumidor fornece
  /// [label] e escolhe a política com base na evidência recebida.
  Widget _graphField({
    required String label,
    required TextEditingController controller,
    required bool readOnly,
    required bool busy,
  }) => TextFormField(
    controller: controller,
    readOnly: readOnly,
    enabled: !busy,
    decoration: InputDecoration(
      labelText: label,
      hintText: readOnly ? null : 'Não informado (opcional)',
      suffixIcon: readOnly ? const Icon(Icons.lock_outline) : null,
    ),
  );

  /// Apresenta a credencial interna sem acrescentar outro contato institucional.
  ///
  /// [busy] bloqueia alterações/envio. Se a senha já existe, o e-mail é somente
  /// leitura e a página pede sua confirmação por senha. Na criação, o endereço
  /// sugerido permanece editável. Os validadores verificam e-mail, senha não vazia
  /// e confirmação correspondente; requisitos do provedor são tratados no Firebase.
  /// O botão apenas encaminha [_submit], que aguarda a conclusão da operação.
  Widget _buildCredentialsForm(bool busy) {
    final hasPassword = widget.controller.hasInternalPassword;

    return AutofillGroup(
      child: Form(
        key: _credentialsForm,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text(
              'Dados de acesso',
              style: TextStyle(fontSize: 22, fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 8),
            Text(
              hasPassword
                  ? 'Confirme sua senha interna para concluir o cadastro.'
                  : 'Escolha seu e-mail e sua senha do CApp. O e-mail institucional é apenas uma sugestão.',
            ),
            const SizedBox(height: 24),
            TextFormField(
              controller: _email,
              enabled: !busy,
              readOnly: hasPassword,
              keyboardType: TextInputType.emailAddress,
              autofillHints: const [AutofillHints.newUsername],
              decoration: const InputDecoration(labelText: 'E-mail de entrada'),
              validator: (value) => value == null || !value.contains('@')
                  ? 'Informe um e-mail válido.'
                  : null,
            ),
            const SizedBox(height: 12),
            TextFormField(
              controller: _password,
              enabled: !busy,
              obscureText: true,
              autofillHints: [
                hasPassword
                    ? AutofillHints.password
                    : AutofillHints.newPassword,
              ],
              decoration: InputDecoration(
                labelText: hasPassword ? 'Sua senha interna' : 'Criar senha',
              ),
              validator: (value) =>
                  value == null || value.isEmpty ? 'Informe sua senha.' : null,
            ),
            const SizedBox(height: 12),
            TextFormField(
              controller: _confirm,
              enabled: !busy,
              obscureText: true,
              decoration: const InputDecoration(labelText: 'Confirmar senha'),
              validator: (value) => value != _password.text
                  ? 'As senhas devem ser iguais.'
                  : null,
            ),
            const SizedBox(height: 24),
            FilledButton(
              onPressed: busy ? null : _submit,
              child: Text(busy ? 'Salvando...' : 'Concluir cadastro'),
            ),
          ],
        ),
      ),
    );
  }
}
