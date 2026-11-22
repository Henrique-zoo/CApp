import Flutter
import Foundation
import Security

enum VoterKeys {
  // Serializa a criação do par, mesmo com duas chamadas em motores distintos.
  private static let queue = DispatchQueue(label: "br.com.capp.elections.voterkeys")
  private enum KeyError: Error { case unavailable }

  static func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    guard let args = call.arguments as? [String: Any],
          let alias = args["alias"] as? String,
          valid(alias) else {
      result(FlutterError(code: "invalid_alias", message: "Identificador de chave inválido", details: nil))
      return
    }
    queue.async {
      let response: Any?
      do {
        switch call.method {
        case "publicKey":
          if let privateKey = try key(alias, create: args["create"] as? Bool == true) {
            guard let publicKey = SecKeyCopyPublicKey(privateKey),
                  let pkcs1 = SecKeyCopyExternalRepresentation(publicKey, nil) as Data? else {
              throw KeyError.unavailable
            }
            response = FlutterStandardTypedData(bytes: spki(pkcs1))
          } else {
            response = nil
          }
        case "sign":
          guard let input = args["message"] as? FlutterStandardTypedData,
                let privateKey = try key(alias, create: false),
                let signature = SecKeyCreateSignature(
                  privateKey, .rsaSignatureMessagePSSSHA256, input.data as CFData, nil
                ) as Data? else { throw KeyError.unavailable }
          response = FlutterStandardTypedData(bytes: signature)
        default:
          response = FlutterMethodNotImplemented
        }
      } catch {
        response = FlutterError(code: "voter_key_unavailable",
                                message: "Operação com a chave eleitoral indisponível", details: nil)
      }
      DispatchQueue.main.async { result(response) }
    }
  }

  private static func valid(_ alias: String) -> Bool {
    let parts = alias.components(separatedBy: ".")
    return parts.count == 5 && Array(parts.prefix(3)) == ["capp", "elections", "v1"] &&
      parts.suffix(2).allSatisfy { UUID(uuidString: $0)?.uuidString.lowercased() == $0 }
  }

  private static func key(_ alias: String, create: Bool) throws -> SecKey? {
    let tag = Data(alias.utf8)
    let query: [CFString: Any] = [
      kSecClass: kSecClassKey, kSecAttrKeyType: kSecAttrKeyTypeRSA,
      kSecAttrKeyClass: kSecAttrKeyClassPrivate, kSecAttrApplicationTag: tag, kSecReturnRef: true,
    ]
    var found: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &found)
    if status == errSecSuccess, let found = found { return (found as! SecKey) }
    guard status == errSecItemNotFound else { throw KeyError.unavailable }
    guard create else { return nil }
    let attributes: [CFString: Any] = [
      kSecAttrKeyType: kSecAttrKeyTypeRSA, kSecAttrKeySizeInBits: 2048,
      kSecPrivateKeyAttrs: [
        kSecAttrIsPermanent: true, kSecAttrApplicationTag: tag,
        kSecAttrAccessible: kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
      ] as [CFString: Any],
    ]
    guard let generated = SecKeyCreateRandomKey(attributes as CFDictionary, nil) else {
      throw KeyError.unavailable
    }
    return generated
  }

  // Security.framework exporta a chave pública RSA em PKCS#1. O protocolo
  // exige SubjectPublicKeyInfo com rsaEncryption, NULL e zero bits ociosos.
  private static func spki(_ pkcs1: Data) -> Data {
    let algorithm = Data([0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86,
                          0xf7, 0x0d, 0x01, 0x01, 0x01, 0x05, 0x00])
    return der(0x30, algorithm + der(0x03, Data([0]) + pkcs1))
  }

  private static func der(_ tag: UInt8, _ content: Data) -> Data {
    var length = content.count
    if length < 128 { return Data([tag, UInt8(length)]) + content }
    var bytes: [UInt8] = []
    while length > 0 {
      bytes.insert(UInt8(length & 255), at: 0)
      length >>= 8
    }
    return Data([tag, 0x80 | UInt8(bytes.count)] + bytes) + content
  }
}
