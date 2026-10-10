import '../support/step_registry.dart';

import 'academic_identity/academic_data_steps.dart';
import 'authentication/authentication_steps.dart';
import 'shared/access_control_steps.dart';
import 'shared/navigation_steps.dart';
import 'shared/session_steps.dart';

final stepDefinitions = <StepDefinition>[
  ...authenticationSteps,
  ...academicDataSteps,
  ...sessionSteps,
  ...accessControlSteps,
  ...navigationSteps,
];
