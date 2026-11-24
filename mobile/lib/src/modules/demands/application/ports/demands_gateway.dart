/// Contrato de porta da aplicação para operações remotas de demandas.
library;

import 'package:uuid/uuid.dart';

import '../models/create_demand_input.dart';
import '../models/demand.dart';
import '../models/demand_comment.dart';
import '../models/demand_forum.dart';
import '../models/demand_status.dart';

/// Porta consumida pelos casos de uso e controlador do módulo de demandas.
abstract interface class DemandsGateway {
  /// Lista as demandas de um Centro Acadêmico aplicando filtros opcionais.
  Future<List<Demand>> listDemands({
    required UuidValue academicCenterId,
    DemandForum? forum,
    DemandStatus? status,
    bool mine = false,
    int? limit,
    int? offset,
  });

  /// Cria uma nova demanda no Centro Acadêmico informado.
  Future<Demand> createDemand({
    required UuidValue academicCenterId,
    required CreateDemandInput input,
  });

  /// Consulta detalhes e histórico de status de uma demanda específica.
  Future<Demand> getDemand({
    required UuidValue academicCenterId,
    required UuidValue demandId,
  });

  /// Atualiza o status de uma demanda pela gestão do CA com parecer obrigatório.
  Future<Demand> updateDemandStatus({
    required UuidValue academicCenterId,
    required UuidValue demandId,
    required DemandStatus status,
    required String opinion,
  });

  /// Lista os comentários de uma demanda em ordem cronológica ascendente.
  Future<List<DemandComment>> listComments({
    required UuidValue academicCenterId,
    required UuidValue demandId,
  });

  /// Publica um comentário ou resposta na linha do tempo da demanda.
  Future<DemandComment> createComment({
    required UuidValue academicCenterId,
    required UuidValue demandId,
    required String content,
  });
}
