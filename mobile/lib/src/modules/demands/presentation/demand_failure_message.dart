/// Tradução das falhas do módulo de demandas para mensagens públicas ao usuário.
library;

import '../application/error/demand_exception.dart';

/// Converte [failure] em texto legível para exibição em banners e formulários.
String demandFailureMessage(DemandException failure) {
  switch (failure.kind) {
    case DemandFailureKind.forbiddenCaForum:
      return 'O fórum institucional é restrito a estudantes dos cursos '
          'representados pelo Centro Acadêmico e à gestão do CA.';
    case DemandFailureKind.studentMembershipRequired:
      return 'Esta ação exige vínculo estudantil ativo confirmado na '
          'instituição.';
    case DemandFailureKind.academicCenterNotFound:
      return 'Centro Acadêmico não encontrado.';
    case DemandFailureKind.academicCenterInactive:
      return 'Este Centro Acadêmico está inativo no momento.';
    case DemandFailureKind.demandNotFound:
      return 'Demanda não encontrada no Centro Acadêmico ativo.';
    case DemandFailureKind.validationRejected:
      final message = failure.publicMessage;
      if (message != null && message.trim().isNotEmpty) return message;
      return 'Verifique os dados informados e tente novamente.';
    case DemandFailureKind.unauthorized:
      return 'Não foi possível validar sua sessão. Entre novamente para '
          'continuar.';
    case DemandFailureKind.appVerificationFailed:
      return 'Não foi possível comprovar este aplicativo. Tente novamente.';
    case DemandFailureKind.communicationFailed:
      return 'Não foi possível conectar ao servidor. Verifique sua conexão e '
          'tente novamente.';
    case DemandFailureKind.serviceUnavailable:
      return 'O serviço de demandas está temporariamente indisponível. Tente '
          'mais tarde.';
    case DemandFailureKind.invalidResponse:
      return 'Recebemos uma resposta inesperada do servidor. Tente novamente.';
    case DemandFailureKind.operationCancelled:
      return 'A operação foi cancelada.';
    case DemandFailureKind.unexpected:
      return 'Não foi possível concluir a operação. Tente novamente.';
  }
}
