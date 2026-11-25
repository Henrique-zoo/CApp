/// Testes unitários do controlador [DemandsController] e regras de domínio.
library;

import 'dart:async';

import 'package:capp/src/modules/demands/demands.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:uuid/uuid.dart';

import '../../../../support/demands_fakes.dart';

void main() {
  const caId = UuidValue.raw('00000000-0000-0000-0000-000000000001');
  const userId = UuidValue.raw('00000000-0000-0000-0000-000000000010');

  test(
    'alterna fóruns e limpa status exclusivo ao ir para espaço físico',
    () async {
      final gateway = FakeDemandsGateway(
        initialDemands: [
          sampleDemand(
            forum: DemandForum.institutional,
            status: DemandStatus.forwardedToCoordination,
            courseName: 'Engenharia de Software',
          ),
          sampleDemand(
            id: const UuidValue.raw('00000000-0000-0000-0000-000000000021'),
            forum: DemandForum.physicalSpace,
            title: 'Reparo da porta',
          ),
        ],
      );
      final controller = DemandsController(
        gateway: gateway,
        academicCenterId: caId,
        currentUserId: userId,
        userCourseName: 'Engenharia de Software',
      );
      addTearDown(controller.dispose);

      await controller.loadDemands();
      expect(controller.visibleDemands, hasLength(1));

      await controller.selectStatus(DemandStatus.forwardedToCoordination);
      expect(controller.selectedStatus, DemandStatus.forwardedToCoordination);

      await controller.selectForum(DemandForum.physicalSpace);
      expect(controller.selectedForum, DemandForum.physicalSpace);
      expect(controller.selectedStatus, isNull);
      expect(controller.visibleDemands.single.title, 'Reparo da porta');
    },
  );

  test(
    'marca fórum institucional proibido em 403 sem impedir troca de fórum',
    () async {
      final gateway = FakeDemandsGateway(
        forbidInstitutional: true,
        initialDemands: [
          sampleDemand(
            forum: DemandForum.physicalSpace,
            title: 'Troca de lâmpada',
          ),
        ],
      );
      final controller = DemandsController(
        gateway: gateway,
        academicCenterId: caId,
        currentUserId: userId,
      );
      addTearDown(controller.dispose);

      await controller.loadDemands();
      expect(controller.isInstitutionalForumForbidden, isTrue);
      expect(controller.canCreateInSelectedForum, isFalse);

      await controller.selectForum(DemandForum.physicalSpace);
      expect(controller.isInstitutionalForumForbidden, isFalse);
      expect(controller.visibleDemands.single.title, 'Troca de lâmpada');
    },
  );

  test('ignora resposta antiga ao trocar de fórum rapidamente', () async {
    final firstCompleter = Completer<List<Demand>>();
    final gateway = FakeDemandsGateway(
      initialDemands: [
        sampleDemand(
          forum: DemandForum.physicalSpace,
          title: 'Demanda espaço físico',
        ),
      ],
    );
    var calls = 0;
    gateway.onListDemands = ({forum, status, mine = false}) async {
      calls++;
      if (calls == 1) return firstCompleter.future;
      return gateway.defaultList(forum: forum, status: status, mine: mine);
    };

    final controller = DemandsController(
      gateway: gateway,
      academicCenterId: caId,
      currentUserId: userId,
    );
    addTearDown(controller.dispose);

    final firstLoad = controller.loadDemands();
    await controller.selectForum(DemandForum.physicalSpace);
    expect(controller.visibleDemands.single.title, 'Demanda espaço físico');

    firstCompleter.complete([
      sampleDemand(
        forum: DemandForum.institutional,
        title: 'Resposta atrasada',
        courseName: 'Engenharia de Software',
      ),
    ]);
    await firstLoad;

    expect(controller.selectedForum, DemandForum.physicalSpace);
    expect(controller.visibleDemands.single.title, 'Demanda espaço físico');
  });

  test(
    'criação, comentário e atualização de status mantêm estado sincronizado',
    () async {
      final gateway = FakeDemandsGateway();
      final controller = DemandsController(
        gateway: gateway,
        academicCenterId: caId,
        currentUserId: userId,
        userCourseName: 'Engenharia de Software',
        isCaManager: true,
      );
      addTearDown(controller.dispose);

      final created = await controller.createDemand(
        CreateDemandInput(
          forum: DemandForum.physicalSpace,
          title: 'Bebedouro com vazamento',
          description: 'O bebedouro do CA está vazando no corredor.',
        ),
      );
      expect(controller.selectedForum, DemandForum.physicalSpace);
      expect(controller.visibleDemands.single.id, created.id);

      await controller.postComment(created.id, 'Equipe acionada hoje.');
      expect(controller.commentCountFor(created.id), 1);

      await expectLater(
        controller.updateDemandStatus(
          demandId: created.id,
          status: DemandStatus.forwardedToCoordination,
          opinion: 'Tentativa inválida',
        ),
        throwsFormatException,
      );

      final updated = await controller.updateDemandStatus(
        demandId: created.id,
        status: DemandStatus.inAnalysis,
        opinion: 'Manutenção agendada para amanhã.',
      );
      expect(updated.status, DemandStatus.inAnalysis);
      expect(
        controller.demandById(created.id)?.status,
        DemandStatus.inAnalysis,
      );
      expect(controller.commentCountFor(created.id), 1);
    },
  );

  test('CreateDemandInput valida obrigatoriedade de curso e descrição', () {
    expect(
      () => CreateDemandInput(
        forum: DemandForum.physicalSpace,
        description: '   ',
      ),
      throwsFormatException,
    );
    expect(
      () => CreateDemandInput(
        forum: DemandForum.physicalSpace,
        description: 'Descrição válida',
        courseName: 'Engenharia de Software',
      ),
      throwsFormatException,
    );
    expect(
      () => CreateDemandInput(
        forum: DemandForum.institutional,
        description: 'Descrição válida',
      ),
      throwsFormatException,
    );
  });
}
