import 'package:integration_test/integration_test.dart';

import '../tool/backend_tls_test.dart' as checks;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  checks.main();
}
