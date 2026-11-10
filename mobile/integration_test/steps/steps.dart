import '../support/step_registry.dart';
import 'academic_identity/academic_data_steps.dart';
import 'authentication/authentication_steps.dart';
import 'demands/demand_creation_steps.dart';
import 'demands/demand_tracking_steps.dart';
import 'elections/voting_steps.dart';
import 'events/event_browsing_steps.dart';
import 'events/event_capacity_steps.dart';
import 'events/event_management_steps.dart';
import 'faq/faq_consultation_steps.dart';
import 'governance/institutional_publication_steps.dart';
import 'management/default_roles_steps.dart';
import 'management/management_members_steps.dart';
import 'management/multi_ca_context_steps.dart';
import 'management/role_deletion_steps.dart';
import 'materials/material_organization_steps.dart';
import 'materials/material_viewing_steps.dart';
import 'mentoring/manual_pairing_steps.dart';
import 'news/news_publication_steps.dart';
import 'news/news_viewing_steps.dart';
import 'occurrences/occurrence_registration_steps.dart';
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
  ...demandCreationSteps,
  ...demandTrackingSteps,
  ...eventBrowsingSteps,
  ...eventCapacitySteps,
  ...eventManagementSteps,
  ...faqConsultationSteps,
  ...institutionalPublicationSteps,
  ...managementMembersSteps,
  ...manualPairingSteps,
  ...materialOrganizationSteps,
  ...materialViewingSteps,
  ...multiCaContextSteps,
  ...navigationSteps,
  ...newsPublicationSteps,
  ...newsViewingSteps,
  ...occurrenceRegistrationSteps,
  ...roleDeletionSteps,
  ...sessionSteps,
  ...sharedSteps,
  ...userProvisioningSteps,
  ...votingSteps,
];
