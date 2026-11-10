import '../support/step_registry.dart';
import 'academic_identity/academic_data_steps.dart';
import 'authentication/authentication_steps.dart';
import 'governance/institutional_publication_steps.dart';
import 'management/default_roles_steps.dart';
import 'management/management_members_steps.dart';
import 'management/multi_ca_context_steps.dart';
import 'management/role_deletion_steps.dart';
import 'news/news_publication_steps.dart';
import 'news/news_viewing_steps.dart';
import 'provisioning/user_provisioning_steps.dart';
import 'shared/access_control_steps.dart';
import 'shared/navigation_steps.dart';
import 'shared/session_steps.dart';
import 'shared/shared_steps.dart';

final stepDefinitions = <StepDefinition>[
  ...academicDataSteps,
  ...accessControlSteps,
  ...authenticationSteps,
  ...defaultRolesSteps,
  ...institutionalPublicationSteps,
  ...managementMembersSteps,
  ...multiCaContextSteps,
  ...navigationSteps,
  ...newsPublicationSteps,
  ...newsViewingSteps,
  ...roleDeletionSteps,
  ...sessionSteps,
  ...sharedSteps,
  ...userProvisioningSteps,
];
