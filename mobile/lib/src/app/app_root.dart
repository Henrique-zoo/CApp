/// Propriedade da composição durante o ciclo de vida da interface.
///
/// Esta biblioteca resolve a diferença entre recursos duradouros da aplicação
/// e widgets que podem ser removidos ou substituídos. AppRoot recebe uma composição
/// já iniciada e passa a solicitar seu encerramento quando deixa de utilizá-la.
///
/// ## Montagem e substituição
///
/// O proprietário monta SessionLifecycleObserver ao redor de CApp. O observador
/// informa retomadas ao controlador, e CApp apresenta o destino da sessão e o
/// diagnóstico independente de prontidão recebido da mesma composição.
/// Nenhum desses passos recria os adaptadores a cada build.
///
/// Reconstruções com a mesma composição preservam a subárvore. Uma composição
/// com identidade diferente recebe outra chave, recria a interface e inicia o
/// encerramento da anterior. As duas composições devem possuir recursos cujo
/// descarte não invalide a outra.
///
/// ## Descarte assíncrono
///
/// O callback dispose de um widget é síncrono. AppRoot acompanha separadamente
/// o Future de encerramento e encaminha sua exceção e pilha a FlutterError.
/// Não acessa o estado do widget depois de aguardar esse Future.
///
/// O encerramento forçado do processo pode não chamar esse callback. A
/// persistência necessária à restauração acontece durante as operações de sessão,
/// e não depende do descarte visual. Esta biblioteca não inicia login ou logout.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:uuid/uuid.dart';

import '../modules/demands/demands.dart';
import '../modules/demands/demands_widgets.dart';
import '../modules/elections/elections_widgets.dart';
import '../modules/identity/identity.dart';
import 'app_dependencies.dart';
import 'capp.dart';
import 'session_lifecycle_observer.dart';

/// Widget que assume o encerramento das dependências da aplicação.
///
/// O chamador deve construir e iniciar [dependencies] antes de montar este
/// widget. A transferência de propriedade significa que a mesma composição
/// não deve ser compartilhada entre proprietários independentes nem reutilizada
/// depois que seu encerramento tiver começado.
///
/// Reconstruções com a mesma instância preservam as dependências e a interface.
/// A substituição por outra instância inicia o encerramento da anterior e
/// recria a subárvore de [CApp] com a nova composição.
///
/// Não inicia serviços, executa logout ou remove o cache institucional.
/// Também não observa o estado da sessão nem decide qual tela apresentar.
class AppRoot extends StatefulWidget {
  /// Cria o proprietário de uma composição já iniciada.
  ///
  /// [dependencies] deve permanecer utilizável enquanto estiver associada
  /// a este widget. Seu encerramento será solicitado na substituição da
  /// composição ou no descarte deste proprietário.
  ///
  /// [key] participa da identificação do widget na árvore Flutter.
  const AppRoot({
    super.key,
    required this.dependencies,
    this.electionId = '',
    this.manifestHash = '',
    this.academicCenterId = '',
    this.academicCenterName = '',
    this.isCaManager = false,
  });

  /// Composição utilizada pela interface e pertencente a este proprietário.
  ///
  /// Sua identidade determina se uma atualização preserva a composição atual
  /// ou inicia seu encerramento.
  final AppDependencies dependencies;

  /// Identificador da eleição e hash público distribuído pela comissão.
  final String electionId;

  /// Âncora independente usada para conferir o manifesto servido pela API.
  final String manifestHash;

  /// Identificador (UUID) do Centro Acadêmico ativo para consulta de demandas.
  final String academicCenterId;

  /// Nome de exibição opcional do Centro Acadêmico ativo.
  final String academicCenterName;

  /// Indica se a sessão atual atua com papel de gestão do Centro Acadêmico.
  final bool isCaManager;

  /// Cria o estado responsável pela montagem e pelo descarte da composição.
  @override
  State<AppRoot> createState() => _AppRootState();
}

/// Coordena a troca de composição e seu encerramento assíncrono.
///
/// Os callbacks do framework permanecem síncronos. Os Futures de encerramento
/// são acompanhados por [_disposeDependencies], que encaminha suas falhas
/// ao mecanismo de diagnóstico do Flutter.
class _AppRootState extends State<AppRoot> {
  /// Encerra a composição anterior quando sua instância é substituída.
  ///
  /// [oldWidget] contém a configuração anteriormente associada a este estado.
  /// A comparação por identidade evita descartar recursos em reconstruções
  /// que continuam usando a mesma composição.
  ///
  /// O encerramento anterior pode ocorrer enquanto a nova interface é montada.
  /// As composições devem, portanto, possuir recursos cujo descarte não invalide
  /// a composição substituta.
  ///
  /// Não chama `setState`: o framework executa [build] após esta atualização.
  @override
  void didUpdateWidget(covariant AppRoot oldWidget) {
    super.didUpdateWidget(oldWidget);

    if (oldWidget.electionId != widget.electionId ||
        oldWidget.manifestHash != widget.manifestHash) {
      _localManifestHash = null;
    }
    if (oldWidget.academicCenterId != widget.academicCenterId) {
      _localAcademicCenterId = null;
      _disposeDemandsController();
    }
    if (!identical(oldWidget.dependencies, widget.dependencies)) {
      _disposeDemandsController();
      unawaited(_disposeDependencies(oldWidget.dependencies));
    }
  }

  String? _localManifestHash;
  String? _localAcademicCenterId;
  DemandsController? _demandsController;
  String? _demandsControllerKey;

  void _disposeDemandsController() {
    _demandsController?.dispose();
    _demandsController = null;
    _demandsControllerKey = null;
  }

  Future<void> _enterAcademicCenterId(BuildContext context) async {
    final input = TextEditingController(
      text: _localAcademicCenterId ?? widget.academicCenterId,
    );
    final rawId = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Centro Acadêmico'),
        content: TextField(
          controller: input,
          decoration: const InputDecoration(
            labelText: 'Identificador UUID do Centro Acadêmico',
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, input.text.trim()),
            child: const Text('Confirmar'),
          ),
        ],
      ),
    );
    input.dispose();
    if (!mounted || !context.mounted || rawId == null) return;
    if (!Uuid.isValidUUIDFormat(fromString: rawId)) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Informe um UUID válido para o CA.')),
      );
      return;
    }
    setState(() {
      _localAcademicCenterId = rawId.toLowerCase();
      _disposeDemandsController();
    });
  }

  Future<void> _enterManifestHash(BuildContext context) async {
    final reference = TextEditingController();
    final hash = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Referência do manifesto'),
        content: TextField(
          controller: reference,
          decoration: const InputDecoration(
            labelText: 'Hash recebido da comissão',
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, reference.text.trim()),
            child: const Text('Confirmar'),
          ),
        ],
      ),
    );
    reference.dispose();
    if (!mounted || !context.mounted || hash == null) return;
    if (!RegExp(r'^[0-9a-f]{64}$').hasMatch(hash)) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Informe 64 dígitos hexadecimais minúsculos.'),
        ),
      );
      return;
    }
    setState(() => _localManifestHash = hash);
  }

  Widget _buildElectionHome(BuildContext context) {
    if (widget.electionId.isEmpty && widget.manifestHash.isEmpty) {
      return const Center(child: Text('Nenhuma eleição disponível.'));
    }
    if (!Uuid.isValidUUIDFormat(fromString: widget.electionId)) {
      return const Center(child: Text('Eleição não configurada.'));
    }
    final election = UuidValue.withFormatValidation(
      widget.electionId.toLowerCase(),
    );
    final dependencies = widget.dependencies;
    final hash = _localManifestHash ?? widget.manifestHash;
    void openAuthority() {
      Navigator.of(context).push<void>(
        MaterialPageRoute(
          builder: (_) => ElectionAuthorityPage(
            session: dependencies.secretVoting.authority(election),
            manifestHash: hash,
          ),
        ),
      );
    }

    if (!RegExp(r'^[0-9a-f]{64}$').hasMatch(hash)) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            FilledButton(
              onPressed: () => _enterManifestHash(context),
              child: const Text('Informar hash do manifesto'),
            ),
            TextButton(
              onPressed: openAuthority,
              child: const Text('Autoridades da eleição'),
            ),
          ],
        ),
      );
    }
    return ElectionHome(
      key: ValueKey((election, hash)),
      classifyFailure: dependencies.classifyVotingFailure,
      openAuthority: openAuthority,
      audit: (download, {bundleJson, anchorJson}) =>
          dependencies.secretVoting.audit(
            election,
            trustedManifestHash: hash,
            download: download,
            bundleJson: bundleJson,
            anchorJson: anchorJson,
          ),
      inspect: () => dependencies.secretVoting.inspect(
        election,
        trustedManifestHash: hash,
      ),
      register: () async {
        await dependencies.voterRegistration.register(election);
      },
    );
  }

  Widget _buildDemandsHome(BuildContext context, SessionSignedIn state) {
    final rawCenterId = _localAcademicCenterId ?? widget.academicCenterId;
    if (rawCenterId.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Text('Nenhum Centro Acadêmico ativo configurado.'),
              const SizedBox(height: 12),
              FilledButton(
                onPressed: () => _enterAcademicCenterId(context),
                child: const Text('Informar Centro Acadêmico'),
              ),
            ],
          ),
        ),
      );
    }
    if (!Uuid.isValidUUIDFormat(fromString: rawCenterId)) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Text('Centro Acadêmico não configurado.'),
              const SizedBox(height: 12),
              FilledButton(
                onPressed: () => _enterAcademicCenterId(context),
                child: const Text('Informar Centro Acadêmico'),
              ),
            ],
          ),
        ),
      );
    }

    final academicCenterId = UuidValue.withFormatValidation(
      rawCenterId.toLowerCase(),
    );
    final affiliation = state.verification?.profile.affiliation;
    final courseName = affiliation is StudentAffiliation
        ? affiliation.courseName
        : null;
    final key =
        '${state.firebaseUid}|${academicCenterId.uuid}|${state.user?.id.uuid}|'
        '$courseName|${widget.isCaManager}';

    if (_demandsController == null || _demandsControllerKey != key) {
      _demandsController?.dispose();
      _demandsControllerKey = key;
      _demandsController = DemandsController(
        gateway: widget.dependencies.demandsGateway,
        academicCenterId: academicCenterId,
        academicCenterName: widget.academicCenterName.isEmpty
            ? null
            : widget.academicCenterName,
        currentUserId: state.user?.id,
        userCourseName: courseName,
        hasStudentMembership: affiliation is StudentAffiliation,
        isCaManager: widget.isCaManager,
      );
    }

    return DemandsHome(
      controller: _demandsController!,
      onChangeAcademicCenter: () => _enterAcademicCenterId(context),
    );
  }

  /// Monta a interface com os controladores da composição atual.
  ///
  /// A identidade da composição preserva a interface em reconstruções
  /// normais e a recria quando as dependências são substituídas.
  ///
  /// Ambos os controladores já devem estar iniciados pelo chamador.
  @override
  Widget build(BuildContext context) {
    return SessionLifecycleObserver(
      key: ObjectKey(widget.dependencies),
      controller: widget.dependencies.sessionController,
      child: CApp(
        electionHome: Builder(builder: _buildElectionHome),
        demandsHomeBuilder: _buildDemandsHome,
        sessionController: widget.dependencies.sessionController,
        backendReadinessController:
            widget.dependencies.backendReadinessController,
      ),
    );
  }

  /// Solicita o encerramento da composição atual ao descartar este estado.
  ///
  /// O callback é síncrono: o framework não aguarda a conclusão do descarte
  /// das dependências. Seu Future é acompanhado separadamente, sem acessar
  /// este estado após a conclusão.
  ///
  /// Não representa um callback garantido de fechamento do processo.
  /// Dados necessários à restauração devem ser persistidos durante as
  /// operações correspondentes, sem depender deste método.
  @override
  void dispose() {
    _disposeDemandsController();
    unawaited(_disposeDependencies(widget.dependencies));

    super.dispose();
  }

  /// Aguarda o encerramento de [dependencies] e reporta eventuais falhas.
  ///
  /// O encerramento efetivo pertence a [AppDependencies.dispose], que
  /// compartilha o mesmo Future entre chamadas repetidas.
  ///
  /// Uma falha é encaminhada a [FlutterError.reportError] com a exceção
  /// e a pilha originais. Este método não executa nova tentativa nem
  /// apresenta uma mensagem na interface.
  ///
  /// Com o tratamento padrão do Flutter, a falha reportada não é relançada.
  /// Um manipulador de erros personalizado pode, entretanto, lançar sua
  /// própria exceção durante o processamento do diagnóstico.
  Future<void> _disposeDependencies(AppDependencies dependencies) async {
    try {
      await dependencies.dispose();
    } catch (error, stackTrace) {
      FlutterError.reportError(
        FlutterErrorDetails(
          exception: error,
          stack: stackTrace,
          library: 'capp',
          context: ErrorDescription('ao encerrar dependências da aplicação'),
        ),
      );
    }
  }
}
