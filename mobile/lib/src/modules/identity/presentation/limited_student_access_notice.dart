/// Explicação do acesso limitado após uma verificação institucional conclusiva.
///
/// A ausência de grupo estudantil ou de curso confirmado pelo Graph permite
/// continuar no aplicativo, mas não concede direitos de estudante em nenhum CA.
/// O cadastro e a área principal reutilizam este aviso persistente, inclusive
/// depois de restaurar a sessão. O consumidor decide quando apresentá-lo; este
/// componente não autentica, calcula permissões nem consulta provedores.
library;

import 'package:flutter/material.dart';

import '../../../presentation/presentation.dart';

/// Torna explícito que a entrada não equivale à confirmação de direitos estudantis.
class LimitedStudentAccessNotice extends StatelessWidget {
  const LimitedStudentAccessNotice({super.key});

  @override
  Widget build(BuildContext context) => Semantics(
    liveRegion: true,
    child: const WarningBox(
      message:
          'Acesso limitado: não foi possível confirmar seu vínculo de estudante '
          'e o curso pela instituição. Você pode continuar usando o CApp, '
          'mas não tem permissões de estudante em nenhum CA. '
          'O curso não pode ser informado manualmente.',
    ),
  );
}
