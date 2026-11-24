package br.com.capp

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.StandardMethodCodec

class MainActivity : FlutterActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        val messenger = flutterEngine.dartExecutor.binaryMessenger
        MethodChannel(
            messenger, "br.com.capp/elections/voter-keys",
            StandardMethodCodec.INSTANCE, messenger.makeBackgroundTaskQueue()
        ).setMethodCallHandler(VoterKeys)
        MethodChannel(
            messenger, "br.com.capp/elections/authority-secrets",
            StandardMethodCodec.INSTANCE, messenger.makeBackgroundTaskQueue()
        ).setMethodCallHandler(AuthoritySecrets(applicationContext))

    }
}
