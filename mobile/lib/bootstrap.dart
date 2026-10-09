import 'package:firebase_core/firebase_core.dart';

import 'firebase_options.dart';

Future<void> initializeServices() async {
  if (Firebase.apps.isEmpty) {
    await Firebase.initializeApp(
      options: DefaultFirebaseOptions.currentPlatform,
    );
  }
}
