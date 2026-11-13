/// Confirmação da saída e da forma de acesso futuro à conta selecionada.
///
/// A janela devolve uma intenção; somente a composição/controlador encerra a
/// sessão ou limpa o cache após confirmação. Cancelar, voltar ou tocar fora
/// devolve null e preserva a sessão. Cada abertura começa sem esquecer a conta
/// e exigindo login, quando a conta já possui a credencial interna necessária.
///
/// Esquecer tem precedência: seleciona e bloqueia Exigir login. Ao desmarcar
/// Esquecer conta, a proteção continua selecionada e pode ser editada novamente.
/// Contas incompletas sem senha podem conservar a sessão ou ser esquecidas;
/// exigir uma senha ainda inexistente não é uma operação válida nesse estado.
library;

import 'package:flutter/material.dart';

import '../application/models/saved_account.dart';

/// Confirma a saída sem executar operações de autenticação ou armazenamento.
Future<AccountSignOutMode?> confirmAccountSignOut(
  BuildContext context, {
  bool canRequirePassword = true,
}) => showDialog<AccountSignOutMode>(
  context: context,
  builder: (_) => _AccountSignOutDialog(canRequirePassword: canRequirePassword),
);

class _AccountSignOutDialog extends StatefulWidget {
  const _AccountSignOutDialog({required this.canRequirePassword});

  final bool canRequirePassword;

  @override
  State<_AccountSignOutDialog> createState() => _AccountSignOutDialogState();
}

class _AccountSignOutDialogState extends State<_AccountSignOutDialog> {
  bool _forget = false;
  late bool _requireLogin = widget.canRequirePassword;

  AccountSignOutMode get _mode => _forget
      ? AccountSignOutMode.forgetAccount
      : _requireLogin
      ? AccountSignOutMode.requirePassword
      : AccountSignOutMode.keepSession;

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: const Text('Tem certeza que deseja sair?'),
    content: SingleChildScrollView(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          CheckboxListTile(
            contentPadding: EdgeInsets.zero,
            controlAffinity: ListTileControlAffinity.leading,
            title: const Text('Esquecer conta'),
            subtitle: const Text(
              'Remover a conta e seus dados salvos deste dispositivo.',
            ),
            value: _forget,
            onChanged: (value) => setState(() {
              _forget = value ?? false;

              if (_forget && widget.canRequirePassword) _requireLogin = true;
            }),
          ),
          CheckboxListTile(
            contentPadding: EdgeInsets.zero,
            controlAffinity: ListTileControlAffinity.leading,
            title: const Text('Exigir login'),
            subtitle: Text(
              widget.canRequirePassword || _forget
                  ? 'Pedir autenticação novamente na próxima entrada.'
                  : 'Disponível após configurar a senha da conta.',
            ),
            value: _forget || _requireLogin,
            onChanged: _forget || !widget.canRequirePassword
                ? null
                : (value) => setState(() => _requireLogin = value ?? false),
          ),
        ],
      ),
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.pop(context),
        child: const Text('Cancelar'),
      ),
      FilledButton(
        style: FilledButton.styleFrom(
          backgroundColor: const Color(0xFFDC2626),
          foregroundColor: Colors.white,
        ),
        onPressed: () => Navigator.pop(context, _mode),
        child: const Text('Sair'),
      ),
    ],
  );
}
