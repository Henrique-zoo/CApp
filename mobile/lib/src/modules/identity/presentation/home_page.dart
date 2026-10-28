/// Apresentação da tela inicial para navegação no aplicativo.
///
/// Traduz o perfil confirmado em dados visuais e oferece uma ação de saída.
/// A composição fornece o callback; o controlador conserva a coordenação do
/// provedor e do cache. A tela bloqueia a ação enquanto a sessão está ocupada.
///
/// A mensagem de vínculo não estudantil é informativa e permanece no layout.
/// Falhas operacionais da saída são apresentadas pela composição por meio do
/// estado publicado, sem interpretar SDKs ou apagar diagnósticos nesta tela.
library;

import 'package:capp/src/modules/identity/identity.dart';
import 'package:flutter/material.dart';
import 'package:capp/src/presentation/presentation.dart';

/// Apresenta o perfil institucional e encaminha a intenção de encerrar a sessão.
///
/// O estado deve pertencer ao ramo pronto do gate, com verificação presente.
/// O proprietário acompanha as falhas do callback e fornece novos estados;
/// esta tela não possui controlador, inscrição ou recursos de autenticação.
class HomePage extends StatelessWidget {
  /// Recebe a fotografia da sessão e a operação de saída da composição.
  ///
  /// [state] fornece perfil e atividade; [onSignOut] deve tratar as falhas
  /// operacionais de seu Future e manter os diagnósticos observáveis.
  const HomePage({super.key, required this.state, required this.onSignOut});

  /// Fotografia atual do perfil e da operação em andamento.
  final SessionSignedIn state;

  /// Operação solicitada pelo botão, sem acesso direto ao provedor nesta tela.
  final Future<void> Function() onSignOut;

  /// Constrói o ponto inicial da área principal autenticada.
  ///
  /// [state] é fornecido pelo ramo do gate que exige verificação válida.
  /// Por esse contrato, seu perfil está disponível nesta construção.
  ///
  /// O vínculo não estudantil gera uma mensagem informativa e não impede
  /// a montagem desta área. As funcionalidades principais podem ser
  /// acrescentadas a partir desta tela.
  ///
  /// O botão fica desabilitado durante qualquer operação e indica a saída
  /// em andamento. O callback fornecido possui o tratamento de suas falhas.
  @override
  Widget build(BuildContext context) {
    final profile = state.verification!.profile;
    final affiliation = profile.affiliation;

    return Scaffold(
      appBar: AppBar(
        title: const Text('CApp'),
        actions: [
          TextButton.icon(
            onPressed: state.isBusy ? null : onSignOut,
            icon: const Icon(Icons.logout),
            label: Text(
              state.operation == SessionOperation.signingOut
                  ? 'Saindo...'
                  : 'Sair',
            ),
          ),
        ],
      ),
      body: SafeArea(
        child: Center(
          child: Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    'Dados institucionais',
                    style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
                  ),

                  const SizedBox(height: 16),

                  InputDecorator(
                    decoration: const InputDecoration(labelText: 'Nome'),
                    child: Text(profile.name ?? 'Não informado'),
                  ),

                  const SizedBox(height: 16),

                  InputDecorator(
                    decoration: const InputDecoration(labelText: 'E-mail'),
                    child: Text(profile.email ?? 'Não informado'),
                  ),

                  if (affiliation is StudentAffiliation) ...[
                    const SizedBox(height: 16),

                    InputDecorator(
                      decoration: const InputDecoration(labelText: 'Curso'),
                      child: Text(affiliation.courseName ?? 'Não informado'),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ),
      bottomNavigationBar: affiliation is NonStudentAffiliation
          ? const SafeArea(
              child: Padding(
                padding: EdgeInsets.all(16),
                child: WarningBox(
                  message:
                      'Você não foi reconhecido como estudante da sua '
                      'instituição. Isso significa que você não terá acesso '
                      'a diversas funcionalidades exclusivas de estudantes.',
                ),
              ),
            )
          : null,
    );
  }
}
