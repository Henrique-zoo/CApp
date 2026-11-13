/// Navegação da área autenticada entre o início do aplicativo e a conta.
///
/// A camada app possui o rodapé porque ele conecta áreas do aplicativo. A página
/// de conta continua no módulo de identidade, recebendo somente a fotografia da
/// sessão e o callback de saída. Nenhum SDK ou caso de uso é criado aqui.
///
/// Cada entrada na área pronta começa em Início. Trocar de aba não autentica,
/// renova ou encerra a sessão; atualizações da mesma conta preservam a seleção.
/// O SessionGate descarta toda a navegação quando o destino ou a identidade muda.
/// O espaço inicial está disponível para as funcionalidades principais, sem
/// apresentar automaticamente os dados pessoais da conta.
/// O rodapé mostra somente ícones; os rótulos identificam os destinos para
/// leitores de tela e dicas de acessibilidade.
library;

import 'package:flutter/material.dart';

import '../modules/identity/identity.dart';
import '../modules/identity/identity_widgets.dart';

/// Compõe o rodapé global e a apresentação da conta autenticada.
class MainPage extends StatefulWidget {
  const MainPage({super.key, required this.state, required this.onSignOut});

  /// Fotografia do ramo pronto; a identidade da subárvore pertence ao gate.
  final SessionSignedIn state;

  /// Executa o modo confirmado na Conta, com tratamento de falhas na composição.
  final Future<void> Function(AccountSignOutMode mode) onSignOut;

  @override
  State<MainPage> createState() => _MainPageState();
}

class _MainPageState extends State<MainPage> {
  int _selectedIndex = 0;

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(_selectedIndex == 0 ? 'CApp' : 'Conta')),
    body: _selectedIndex == 0
        ? const SizedBox.expand()
        : AccountPage(state: widget.state, onSignOut: widget.onSignOut),
    bottomNavigationBar: NavigationBar(
      labelBehavior: NavigationDestinationLabelBehavior.alwaysHide,
      selectedIndex: _selectedIndex,
      onDestinationSelected: widget.state.isBusy
          ? null
          : (index) => setState(() => _selectedIndex = index),
      destinations: const [
        NavigationDestination(
          icon: Icon(Icons.home_outlined),
          selectedIcon: Icon(Icons.home),
          label: 'Início',
        ),
        NavigationDestination(
          icon: Icon(Icons.person_outline),
          selectedIcon: Icon(Icons.person),
          label: 'Conta',
        ),
      ],
    ),
  );
}
