import '../support/step_registry.dart';
import 'authentication/authentication_steps.dart';
import 'demands/demand_creation_steps.dart';
import 'governance/institutional_publication_steps.dart';
import 'management/multi_ca_context_steps.dart';
import 'shared/access_control_steps.dart';
import 'shared/navigation_steps.dart';
import 'shared/pending_product_steps.dart';
import 'shared/session_steps.dart';

final stepDefinitions = <StepDefinition>[
  ...accessControlSteps,
  ...authenticationSteps,
  ...demandCreationSteps,
  ...institutionalPublicationSteps,
  ...multiCaContextSteps,
  ...navigationSteps,
  ...pendingProductSteps,
  ...sessionSteps,
];
