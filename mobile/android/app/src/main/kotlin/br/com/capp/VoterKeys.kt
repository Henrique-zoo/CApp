package br.com.capp

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PrivateKey
import java.security.Signature

/** A chave privada RSA permanece no AndroidKeyStore; o canal só expõe SPKI e assinatura. */
internal object VoterKeys : MethodChannel.MethodCallHandler {
    private const val STORE = "AndroidKeyStore"
    private const val UUID = "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
    private val aliasPattern = Regex("capp\\.elections\\.v1\\.$UUID\\.$UUID")

    @Synchronized
    override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        val alias = call.argument<String>("alias")
        if (alias == null || !aliasPattern.matches(alias)) {
            result.error("invalid_alias", "Identificador de chave inválido", null)
            return
        }
        try {
            val store = KeyStore.getInstance(STORE).apply { load(null) }
            when (call.method) {
                "publicKey" -> {
                    if (!store.containsAlias(alias) && call.argument<Boolean>("create") == true) {
                        val generator = KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_RSA, STORE)
                        generator.initialize(
                            KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_SIGN)
                                .setKeySize(2048)
                                .setDigests(KeyProperties.DIGEST_SHA256)
                                .setSignaturePaddings(KeyProperties.SIGNATURE_PADDING_RSA_PSS)
                                .build()
                        )
                        generator.generateKeyPair()
                    }
                    result.success(store.getCertificate(alias)?.publicKey?.encoded)
                }
                "sign" -> {
                    val message = requireNotNull(call.argument<ByteArray>("message"))
                    val key = requireNotNull(store.getKey(alias, null) as? PrivateKey)
                    val signer = Signature.getInstance("SHA256withRSA/PSS")
                    signer.initSign(key)
                    signer.update(message)
                    result.success(signer.sign())
                }
                else -> result.notImplemented()
            }
        } catch (error: Exception) {
            result.error("voter_key_unavailable", "Operação com a chave eleitoral indisponível", null)
        }
    }
}
