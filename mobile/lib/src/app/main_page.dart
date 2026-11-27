/// Navegação da área autenticada entre Início, Materiais e Conta.
///
/// A camada app possui o rodapé porque ele conecta áreas do aplicativo. A página
/// de conta continua no módulo de identidade, recebendo somente a fotografia da
/// sessão e o callback de saída. Nenhum SDK ou caso de uso é criado aqui.
///
/// Cada entrada na área pronta começa em Início. Trocar de aba não autentica,
/// renova ou encerra a sessão; atualizações da mesma conta preservam a seleção.
/// O SessionGate descarta toda a navegação quando o destino ou a identidade muda.
/// A ausência de grupo ou curso confirmado mantém o aviso de acesso limitado
/// visível tanto no início quanto na conta, sem bloquear a entrada.
/// O espaço inicial está disponível para as funcionalidades principais, sem
/// apresentar automaticamente os dados pessoais da conta.
/// O rodapé mostra somente ícones; os rótulos identificam os destinos para
/// leitores de tela e dicas de acessibilidade.
library;

import 'package:flutter/material.dart';

import '../modules/identity/identity.dart';
import '../modules/identity/identity_widgets.dart';

/// Compõe o rodapé global e as áreas disponíveis para a conta autenticada.
class MainPage extends StatefulWidget {
  const MainPage({
    super.key,
    required this.state,
    required this.onSignOut,
    this.electionHome,
    this.materialsHome,
  });

  /// Fotografia do ramo pronto; a identidade da subárvore pertence ao gate.
  final SessionSignedIn state;

  /// Eleição apresentada na aba Início da sessão atual.
  final Widget? electionHome;

  /// Área de materiais disponível para a sessão estudantil.
  final Widget? materialsHome;

  /// Executa o modo confirmado na Conta, com tratamento de falhas na composição.
  final Future<void> Function(AccountSignOutMode mode) onSignOut;

  @override
  State<MainPage> createState() => _MainPageState();
}

enum _Destination {
  home('CApp', 'Início', Icons.home_outlined, Icons.home),
  materials('Materiais', 'Materiais', Icons.school_outlined, Icons.school),
  account('Conta', 'Conta', Icons.person_outline, Icons.person);

  const _Destination(this.title, this.label, this.icon, this.selectedIcon);

  final String title;
  final String label;
  final IconData icon;
  final IconData selectedIcon;
}

class _MainPageState extends State<MainPage> {
  _Destination _selected = _Destination.home;

  @override
  Widget build(BuildContext context) {
    final destinations = [
      _Destination.home,
      if (widget.materialsHome != null) _Destination.materials,
      _Destination.account,
    ];
    final current = destinations.contains(_selected)
        ? _selected
        : _Destination.home;
    return Scaffold(
      appBar: AppBar(title: Text(current.title)),
      body: Column(
        children: [
          if (widget.state.verification!.profile.affiliation
              is NonStudentAffiliation)
            const SafeArea(
              bottom: false,
              child: Padding(
                padding: EdgeInsets.fromLTRB(16, 16, 16, 0),
                child: LimitedStudentAccessNotice(),
              ),
            ),
          Expanded(
            child: switch (current) {
              _Destination.home =>
                widget.electionHome ?? const SizedBox.expand(),
              _Destination.materials => widget.materialsHome!,
              _Destination.account => AccountPage(
                state: widget.state,
                onSignOut: widget.onSignOut,
              ),
            },
          ),
        ],
      ),
      bottomNavigationBar: NavigationBar(
        labelBehavior: NavigationDestinationLabelBehavior.alwaysHide,
        selectedIndex: destinations.indexOf(current),
        onDestinationSelected: widget.state.isBusy
            ? null
            : (index) => setState(() => _selected = destinations[index]),
        destinations: [
          for (final destination in destinations)
            NavigationDestination(
              icon: Icon(destination.icon),
              selectedIcon: Icon(destination.selectedIcon),
              label: destination.label,
            ),
        ],
      ),
    );
  }
}
