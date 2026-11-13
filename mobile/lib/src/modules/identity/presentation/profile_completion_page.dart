/// Cadastro em uma única tela, com duas etapas no mesmo formulário.
///
/// A etapa inicial apresenta nome, e-mail institucional e curso do Graph, além
/// da instituição identificada. Valores externos não nulos são somente leitura;
/// ausências podem receber complementos opcionais declarados pelo usuário. Esses
/// complementos pertencem ao perfil CApp e não alteram a evidência ou o vínculo.
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
import 'session_failure_message.dart';

enum _ProfileStep { academic, credentials }

/// Apresenta o perfil institucional e configura o acesso sem mudar de rota.
class ProfileCompletionPage extends StatefulWidget {
  const ProfileCompletionPage({
    super.key,
    required this.controller,
    required this.state,
    this.now = DateTime.now,
  });

  final SessionController controller;
  final SessionSignedIn state;

  /// Relógio apenas da sugestão acadêmica, sem participar da validade da sessão.
  final DateTime Function() now;

  @override
  State<ProfileCompletionPage> createState() => _ProfileCompletionPageState();
}

class _ProfileCompletionPageState extends State<ProfileCompletionPage> {
  final _academicForm = GlobalKey<FormState>();
  final _credentialsForm = GlobalKey<FormState>();
  final _scroll = ScrollController();
  final _name = TextEditingController();
  final _institutionalEmail = TextEditingController();
  final _course = TextEditingController();
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

  @override
  void initState() {
    super.initState();
    _loadAcademicData();
    _loadGraphData();
  }

  void _loadAcademicData() {
    final user = widget.state.user;

    _nickname.text = user?.nickname ?? '';
    _semester.text = user?.currentSemester?.toString() ?? '';
    _entry.text = user?.institutionEntrySemester ?? '';
    _entryWasEdited = _entry.text.isNotEmpty;

    _suggestEntry();
  }

  void _loadGraphData() {
    final profile = widget.state.verification?.profile;
    final affiliation = profile?.affiliation;

    _name.text = profile?.name ?? widget.state.user?.name ?? '';
    _institutionalEmail.text =
        profile?.email ?? widget.state.user?.institutionalEmail ?? '';
    _course.text =
        (affiliation is StudentAffiliation ? affiliation.courseName : null) ??
        widget.state.user?.courseName ??
        '';
  }

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

  @override
  void dispose() {
    _scroll.dispose();

    for (final controller in [
      _name,
      _institutionalEmail,
      _course,
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

  bool get _canInteract {
    final state = widget.controller.state;

    return state is SessionSignedIn &&
        !state.isBusy &&
        state.firebaseUid == widget.state.firebaseUid &&
        state.access == SessionAccess.profileRequired;
  }

  void _continue() {
    if (!_canInteract || !_academicForm.currentState!.validate()) return;

    if (widget.controller.hasInternalPassword) {
      _email.text = widget.controller.internalEmail ?? _email.text;
    } else if (!_credentialsOpened) {
      _email.text =
          widget.state.user?.email ??
          _optional(_institutionalEmail.text) ??
          widget.controller.internalEmail ??
          '';
    }

    _credentialsOpened = true;

    _changeStep(_ProfileStep.credentials);
  }

  void _back() {
    if (!_canInteract) return;

    _password.clear();
    _confirm.clear();
    _changeStep(_ProfileStep.academic);
  }

  void _changeStep(_ProfileStep step) {
    FocusScope.of(context).unfocus();
    setState(() {
      _step = step;
      _message = null;
    });

    if (_scroll.hasClients) _scroll.jumpTo(0);
  }

  String? _optional(String value) => value.trim().isEmpty ? null : value.trim();

  Future<void> _submit() async {
    if (!_canInteract || !_credentialsForm.currentState!.validate()) return;

    final profile =
        (widget.controller.state as SessionSignedIn).verification!.profile;
    final affiliation = profile.affiliation;

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
          institutionalEmail: profile.email == null
              ? _optional(_institutionalEmail.text)
              : null,
          courseName:
              affiliation is StudentAffiliation &&
                  affiliation.courseName == null
              ? _optional(_course.text)
              : null,
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
            'Confira seus dados. Os valores fornecidos pela instituição não podem ser alterados; os campos não informados podem ser completados.',
          ),
          const SizedBox(height: 24),
          _graphField(
            label: 'Nome completo',
            controller: _name,
            readOnly: profile.name != null,
            busy: busy,
          ),
          const SizedBox(height: 12),
          _graphField(
            label: 'E-mail institucional',
            controller: _institutionalEmail,
            readOnly: profile.email != null,
            busy: busy,
            email: true,
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
            _graphField(
              label: 'Curso',
              controller: _course,
              readOnly: affiliation.courseName != null,
              busy: busy,
            ),
          ],
          const SizedBox(height: 8),
          Text(
            affiliation is StudentAffiliation
                ? 'Vínculo estudantil confirmado.'
                : 'Vínculo estudantil não confirmado.',
          ),
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

  Widget _graphField({
    required String label,
    required TextEditingController controller,
    required bool readOnly,
    required bool busy,
    bool email = false,
  }) => TextFormField(
    controller: controller,
    readOnly: readOnly,
    enabled: !busy,
    keyboardType: email ? TextInputType.emailAddress : TextInputType.text,
    decoration: InputDecoration(
      labelText: label,
      hintText: readOnly ? null : 'Não informado (opcional)',
      suffixIcon: readOnly ? const Icon(Icons.lock_outline) : null,
    ),
    validator: !email || readOnly
        ? null
        : (value) {
            final text = value?.trim() ?? '';

            return text.isNotEmpty &&
                    (!text.contains('@') || RegExp(r'\s').hasMatch(text))
                ? 'Informe um e-mail válido.'
                : null;
          },
  );

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
