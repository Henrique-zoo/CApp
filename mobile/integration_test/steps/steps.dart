import '../support/step_registry.dart';
import 'academic_identity/academic_data_steps.dart';
import 'authentication/authentication_steps.dart';
import 'elections/voting_steps.dart';
import 'governance/institutional_publication_steps.dart';
import 'management/management_members_steps.dart';
import 'management/multi_ca_context_steps.dart';
import 'shared/access_control_steps.dart';
import 'shared/navigation_steps.dart';
import 'shared/session_steps.dart';
import 'shared/shared_steps.dart';

final stepDefinitions = <StepDefinition>[
  ...academicDataSteps,
  ...accessControlSteps,
  ...authenticationSteps,
  ...institutionalPublicationSteps,
  ...managementMembersSteps,
  ...multiCaContextSteps,
  ...navigationSteps,
  ...sessionSteps,
  ...sharedSteps,
  ...votingSteps,
];
