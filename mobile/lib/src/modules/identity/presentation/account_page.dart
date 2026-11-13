/// Consulta da conta autenticada e confirmação de saída do dispositivo.
///
/// Esta página pertence à apresentação de identidade: mostra o perfil interno,
/// os dados institucionais e a origem dos complementos declarados. Não escolhe
/// o destino inicial do aplicativo nem monta a navegação entre módulos; a camada
/// app apresenta esta página somente ao selecionar a aba Conta.
///
/// Os valores do Graph têm precedência sobre complementos. O aviso de vínculo
/// não estudantil continua informativo e não é convertido em recusa de acesso.
/// A página não permite edição, consulta SDKs ou guarda credenciais.
/// Os dados aparecem diretamente como rótulos e valores, sem caixas de entrada
/// ou decoração de formulário.
///
/// O botão vermelho abre uma confirmação e somente então encaminha o modo de
/// saída à composição. A confirmação e a operação em curso bloqueiam novos
/// cliques. Falhas continuam no estado da sessão e são apresentadas pelo dono
/// do callback; uma troca de identidade descarta a página e sua confirmação.
library;

import 'package:flutter/material.dart';

import '../identity.dart';
import '../../../presentation/presentation.dart';
import 'account_sign_out_dialog.dart';

/// Exibe os dados da conta e encaminha uma saída explicitamente confirmada.
class AccountPage extends StatefulWidget {
  const AccountPage({super.key, required this.state, required this.onSignOut});

  /// Fotografia pronta da sessão, com perfil e evidência institucional vigentes.
  final SessionSignedIn state;

  /// O proprietário executa a intenção e acompanha as falhas da operação.
  final Future<void> Function(AccountSignOutMode mode) onSignOut;

  @override
  State<AccountPage> createState() => _AccountPageState();
}

class _AccountPageState extends State<AccountPage> {
  bool _confirming = false;

  Future<void> _signOut() async {
    if (_confirming || widget.state.isBusy) return;

    setState(() => _confirming = true);

    try {
      final mode = await confirmAccountSignOut(context);

      if (mode != null && mounted && !widget.state.isBusy) {
        await widget.onSignOut(mode);
      }
    } finally {
      if (mounted) setState(() => _confirming = false);
    }
  }

  Widget _detail(String label, String? value, {bool declared = false}) =>
      Padding(
        padding: const EdgeInsets.only(bottom: 20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              label,
              style: Theme.of(context).textTheme.labelMedium?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              value ?? 'Não informado',
              style: Theme.of(context).textTheme.bodyLarge,
            ),
            if (declared) ...[
              const SizedBox(height: 4),
              Text(
                'Informado por você',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ],
        ),
      );

  @override
  Widget build(BuildContext context) {
    final state = widget.state;
    final user = state.user;
    final verification = state.verification!;
    final profile = verification.profile;
    final affiliation = profile.affiliation;
    final institution = verification.institution;
    final busy = state.isBusy || _confirming;

    return SafeArea(
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 560),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const Text(
                  'Dados da conta',
                  style: TextStyle(fontSize: 22, fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 24),
                _detail('Apelido', user?.nickname),
                _detail('E-mail de entrada', user?.email),
                _detail(
                  'Nome',
                  profile.name ?? user?.name,
                  declared: profile.name == null && user?.name != null,
                ),
                _detail(
                  'E-mail institucional',
                  profile.email ?? user?.institutionalEmail,
                  declared:
                      profile.email == null && user?.institutionalEmail != null,
                ),
                _detail(
                  'Instituição',
                  '${institution.name}${institution.acronym == null ? '' : ' (${institution.acronym})'}',
                ),
                if (affiliation is StudentAffiliation)
                  _detail(
                    'Curso',
                    affiliation.courseName ?? user?.courseName,
                    declared:
                        affiliation.courseName == null &&
                        user?.courseName != null,
                  ),
                _detail(
                  'Período atual do curso',
                  user?.currentSemester?.toString(),
                ),
                _detail(
                  'Semestre de ingresso na instituição',
                  user?.institutionEntrySemester,
                ),
                if (affiliation is NonStudentAffiliation) ...[
                  const SizedBox(height: 16),
                  WarningBox(
                    message:
                        'Você não foi reconhecido como estudante da '
                        '${institution.acronym ?? institution.name}. '
                        'Isso significa que você não terá acesso '
                        'a diversas funcionalidades exclusivas de estudantes.',
                  ),
                ],
                const SizedBox(height: 24),
                FilledButton.icon(
                  style: FilledButton.styleFrom(
                    backgroundColor: const Color(0xFFDC2626),
                    foregroundColor: Colors.white,
                    minimumSize: const Size.fromHeight(48),
                  ),
                  onPressed: busy ? null : _signOut,
                  icon: const Icon(Icons.logout),
                  label: state.operation == SessionOperation.signingOut
                      ? const AnimatedEllipsisText('Saindo')
                      : const Text('Sair da conta'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
