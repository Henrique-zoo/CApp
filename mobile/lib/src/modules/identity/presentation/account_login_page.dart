/// Seletor de contas lembradas e escolha explícita da forma de entrada.
///
/// A primeira visualização centraliza as contas salvas em uma caixa de borda
/// roxa, acima das opções de entrar com e-mail/senha ou Microsoft.
/// Não há cabeçalho ou título de seção;
/// sem contas lembradas, a área da lista permanece vazia.
/// O formulário interno só aparece após
/// escolher essa opção ou quando uma conta selecionada exigir senha. Voltar ao
/// seletor limpa os campos e a seleção; não altera as contas lembradas.
///
/// Selecionar uma conta solicita ao controlador uma sessão SDK utilizável. Se
/// a política de saída ou a ausência dessa sessão exigir senha, o formulário
/// permanece vinculado ao UID escolhido. Uma sessão utilizável prossegue sem
/// abrir esse formulário. Microsoft é identificada pelo logo e pela indicação
/// de que o primeiro acesso cria uma conta; a sincronização que precisar de
/// interação continua sendo apresentada pela tela institucional.
///
/// O estado local escolhe apenas qual interface apresentar. Autenticação,
/// validade institucional e persistência pertencem aos contratos da aplicação;
/// esta biblioteca não consulta preferências, Firebase ou MSAL diretamente.
/// A lista é recarregada quando a saída termina: o evento de ausência de usuário
/// pode montar esta tela antes da remoção assíncrona no índice de contas.
library;

import 'package:flutter/material.dart';
import 'package:flutter_svg/svg.dart';

import '../identity.dart';
import 'account_sign_out_dialog.dart';
import 'institutional_login_page.dart';
import 'session_failure_message.dart';

/// Prioriza contas salvas e oferece as formas de entrada sem pedir credenciais.
class AccountLoginPage extends StatefulWidget {
  const AccountLoginPage({
    super.key,
    required this.controller,
    required this.state,
  });

  final SessionController controller;
  final SessionState state;

  @override
  State<AccountLoginPage> createState() => _AccountLoginPageState();
}

class _AccountLoginPageState extends State<AccountLoginPage> {
  final _email = TextEditingController();
  final _password = TextEditingController();
  late Future<List<SavedAccount>> _accounts;
  SavedAccount? _selected;
  String? _message;
  bool _working = false;
  bool _showPasswordForm = false;

  @override
  void initState() {
    super.initState();

    _accounts = widget.controller.savedAccounts();
  }

  @override
  void didUpdateWidget(covariant AccountLoginPage oldWidget) {
    super.didUpdateWidget(oldWidget);

    final signOutFinished =
        oldWidget.state.operation == SessionOperation.signingOut;

    if (widget.state is SessionSignedOut &&
        !widget.state.isBusy &&
        (signOutFinished || oldWidget.state is SessionSignedIn)) {
      _accounts = widget.controller.savedAccounts();
      _selected = null;
      _showPasswordForm = false;
      _message = null;

      _email.clear();
      _password.clear();
    }
  }

  @override
  void dispose() {
    _email.dispose();
    _password.dispose();
    super.dispose();
  }

  Future<void> _run(Future<void> Function() action) async {
    if (_working || widget.controller.state.isBusy) return;

    setState(() {
      _working = true;
      _message = null;
    });

    try {
      await action();
    } on AccountPasswordRequiredException {
      if (mounted) {
        setState(() {
          _showPasswordForm = true;
          _message = 'Informe a senha interna desta conta.';
        });
      }
    } on Exception catch (error) {
      if (mounted) {
        setState(() {
          _message = error is IdentityException && error.publicMessage != null
              ? error.publicMessage
              : 'Não foi possível concluir a entrada. Tente novamente.';
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          _working = false;

          _password.clear();
        });
      }
    }
  }

  Future<void> _select(SavedAccount account) async {
    if (_working || widget.controller.state.isBusy) return;

    setState(() {
      _selected = account;
      _email.text = account.email ?? '';
    });

    await _run(() => widget.controller.selectAccount(account.firebaseUid));
  }

  /// Abre a entrada interna sem associá-la a uma conta lembrada.
  void _openPasswordLogin() {
    if (_working || widget.controller.state.isBusy) return;

    setState(() {
      _showPasswordForm = true;
      _selected = null;

      _email.clear();
      _password.clear();

      _message = null;
    });
  }

  /// Descarta os campos de entrada ao retornar ao seletor, inclusive pelo Android.
  void _returnToAccounts() {
    if (_working || widget.controller.state.isBusy) return;

    setState(() {
      _showPasswordForm = false;
      _selected = null;

      _email.clear();
      _password.clear();

      _message = null;
    });
  }

  @override
  Widget build(BuildContext context) {
    if (widget.state is SessionSignedIn) {
      return Scaffold(
        body: Column(
          children: [
            Expanded(
              child: InstitutionalLoginPage(
                controller: widget.controller,
                state: widget.state,
              ),
            ),
            SafeArea(
              child: TextButton(
                onPressed: widget.state.isBusy
                    ? null
                    : () async {
                        final mode = await confirmAccountSignOut(
                          context,
                          canRequirePassword:
                              widget.controller.hasInternalPassword,
                        );

                        if (mode != null && mounted) {
                          await _run(
                            () => widget.controller.signOut(mode: mode),
                          );
                        }
                      },
                child: const Text('Voltar às contas salvas'),
              ),
            ),
          ],
        ),
      );
    }

    final busy = _working || widget.state.isBusy;
    final message = _message ?? sessionFailureMessage(widget.state);

    return PopScope(
      canPop: !_showPasswordForm && !busy,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop && _showPasswordForm) _returnToAccounts();
      },
      child: Scaffold(
        appBar: _showPasswordForm
            ? AppBar(
                title: const Text('Entrar com e-mail e senha'),
                leading: BackButton(onPressed: busy ? null : _returnToAccounts),
              )
            : null,
        bottomNavigationBar: _showPasswordForm
            ? null
            : _buildEntryOptions(busy),
        body: SafeArea(
          child: Align(
            alignment: _showPasswordForm
                ? Alignment.topCenter
                : Alignment.center,
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(24),
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 450),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    if (_showPasswordForm)
                      _buildPasswordForm(busy)
                    else
                      _buildSavedAccounts(busy),
                    if (widget.state.failureDetails?.cause
                        is SessionCacheRemovalException)
                      TextButton(
                        onPressed: busy
                            ? null
                            : () => _run(
                                widget.controller.retryForgottenAccountCleanup,
                              ),
                        child: const Text('Tentar remover os dados locais'),
                      ),
                    if (message != null)
                      Padding(
                        padding: const EdgeInsets.only(top: 16),
                        child: Semantics(
                          liveRegion: true,
                          child: Text(message),
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  /// Mantém as formas de entrada abaixo da lista, mesmo com muitas contas.
  Widget _buildEntryOptions(bool busy) => SafeArea(
    minimum: const EdgeInsets.fromLTRB(24, 0, 24, 24),
    child: Center(
      heightFactor: 1,
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 450),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            OutlinedButton.icon(
              onPressed: busy ? null : _openPasswordLogin,
              icon: const Icon(Icons.alternate_email),
              label: const Text('Entrar com e-mail e senha'),
            ),
            const SizedBox(height: 12),
            FilledButton.icon(
              onPressed: busy ? null : () => _run(widget.controller.signIn),
              icon: SvgPicture.asset(
                'assets/branding/microsoft.svg',
                width: 20,
                height: 20,
                excludeFromSemantics: true,
              ),
              label: const Text('Entrar com Microsoft'),
            ),
            const SizedBox(height: 8),
            const Text(
              'Primeiro acesso? Entre com Microsoft para criar sua conta.',
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    ),
  );

  Widget _buildSavedAccounts(bool busy) => FutureBuilder<List<SavedAccount>>(
    future: _accounts,
    builder: (context, snapshot) {
      if (snapshot.hasError) {
        return TextButton(
          onPressed: busy
              ? null
              : () => setState(() {
                  _accounts = widget.controller.savedAccounts();
                }),
          child: const Text('Tentar carregar contas salvas'),
        );
      }

      if (snapshot.connectionState != ConnectionState.done) {
        return const LinearProgressIndicator();
      }

      final accounts = snapshot.data ?? <SavedAccount>[];

      if (accounts.isEmpty) {
        return const SizedBox.shrink();
      }

      return Material(
        color: Theme.of(context).colorScheme.surface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(20),
          side: const BorderSide(color: Color(0xFF8B5CF6), width: 2),
        ),
        clipBehavior: Clip.antiAlias,
        child: Padding(
          padding: const EdgeInsets.all(8),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              for (final account in accounts)
                ListTile(
                  leading: Icon(
                    account.requiresPassword
                        ? Icons.lock_outline
                        : Icons.account_circle_outlined,
                  ),
                  title: Text(account.name ?? account.email ?? 'Conta salva'),
                  subtitle:
                      account.email != null && account.email != account.name
                      ? Text(account.email!)
                      : null,
                  onTap: busy ? null : () => _select(account),
                  trailing: PopupMenuButton<String>(
                    enabled: !busy,
                    itemBuilder: (_) => [
                      const PopupMenuItem(
                        value: 'forget',
                        child: Text('Esquecer conta'),
                      ),
                    ],
                    onSelected: (_) => _run(() async {
                      await widget.controller.forgetSavedAccount(
                        account.firebaseUid,
                      );

                      if (mounted) {
                        setState(() {
                          _accounts = widget.controller.savedAccounts();
                          _selected = null;

                          _email.clear();
                        });
                      }
                    }),
                  ),
                ),
            ],
          ),
        ),
      );
    },
  );

  /// Pede senha por escolha explícita ou para o UID da conta salva selecionada.
  Widget _buildPasswordForm(bool busy) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      if (_selected != null) ...[
        Text(_selected!.name ?? _selected!.email ?? 'Conta salva'),
        const SizedBox(height: 16),
      ],
      TextField(
        controller: _email,
        enabled: !busy && _selected == null,
        keyboardType: TextInputType.emailAddress,
        autofillHints: const [AutofillHints.username],
        decoration: const InputDecoration(labelText: 'E-mail da conta CApp'),
      ),
      const SizedBox(height: 12),
      TextField(
        controller: _password,
        enabled: !busy,
        obscureText: true,
        autofillHints: const [AutofillHints.password],
        decoration: const InputDecoration(labelText: 'Senha'),
      ),
      const SizedBox(height: 16),
      FilledButton(
        onPressed: busy
            ? null
            : () => _run(
                () => _selected == null
                    ? widget.controller.signInWithPassword(
                        email: _email.text,
                        password: _password.text,
                      )
                    : widget.controller.selectAccount(
                        _selected!.firebaseUid,
                        password: _password.text,
                      ),
              ),
        child: Text(busy ? 'Entrando...' : 'Entrar'),
      ),
      if (_selected != null)
        TextButton(
          onPressed: busy ? null : _returnToAccounts,
          child: const Text('Usar outra conta'),
        ),
      TextButton(
        onPressed: busy
            ? null
            : () => _run(() async {
                await widget.controller.sendPasswordReset(_email.text);

                if (mounted) {
                  setState(
                    () => _message = 'Se houver uma conta para este e-mail, você receberá as instruções de recuperação.',
                  );
                }
              }),
        child: const Text('Esqueci minha senha'),
      ),
    ],
  );
}
