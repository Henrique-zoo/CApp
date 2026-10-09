import '../../support/step_registry.dart';

import 'institutional_account_steps.dart';
import 'login_steps.dart';

final authenticationSteps = <StepDefinition>[
  ...institutionalAccountSteps,
  ...loginSteps,
];
