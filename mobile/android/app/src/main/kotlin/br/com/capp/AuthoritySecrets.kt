package br.com.capp

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** Armazena apenas o escalar da autoridade cifrado com uma chave do Android Keystore. */
internal class AuthoritySecrets(context: Context) : MethodChannel.MethodCallHandler {
    private val storage = context.getSharedPreferences("election-authority-secrets", Context.MODE_PRIVATE)
    private val uuid = "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
    private val aliasPattern = Regex("capp\\.authority\\.v1\\.$uuid\\.$uuid\\.$uuid")

    @Synchronized
    override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        val alias = call.argument<String>("alias")
        if (alias == null || !aliasPattern.matches(alias)) {
            result.error("invalid_alias", "Identificador de autoridade inválido", null)
            return
        }

        try {
            val stored = storage.getString(alias, null)
            when (call.method) {
                "read" -> result.success(stored?.let { decrypt(alias, it) })
                "storeIfAbsent" -> {
                    if (stored != null) {
                        result.success(decrypt(alias, stored))
                        return
                    }
                    val input = requireNotNull(call.argument<ByteArray>("secret"))
                    require(input.size == 256)
                    val encrypted = encrypt(alias, input)
                    check(storage.edit().putString(alias, encrypted).commit())
                    result.success(input)
                }
                else -> result.notImplemented()
            }
        } catch (_: Exception) {
            result.error("authority_secret_unavailable", "Chave da autoridade indisponível neste aparelho", null)
        }
    }

    private fun key(create: Boolean): SecretKey {
        val alias = "capp.authority.v1.storage"
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        val existing = store.getKey(alias, null) as? SecretKey
        if (existing != null) return existing

        require(create)
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        generator.init(
            KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setKeySize(256)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .build()
        )
        return generator.generateKey()
    }

    private fun encrypt(alias: String, secret: ByteArray): String {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key(create = true))
        cipher.updateAAD(alias.toByteArray())
        return Base64.encodeToString(cipher.iv + cipher.doFinal(secret), Base64.NO_WRAP)
    }

    private fun decrypt(alias: String, stored: String): ByteArray {
        val bytes = Base64.decode(stored, Base64.NO_WRAP)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, key(create = false), GCMParameterSpec(128, bytes, 0, 12))
        cipher.updateAAD(alias.toByteArray())
        return cipher.doFinal(bytes, 12, bytes.size - 12)
    }
}
