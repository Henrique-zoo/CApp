//! Codec e assinaturas do protocolo público CAPP-ELECTIONS/1.
//!
//! JSON recebido deve passar por `parse_strict_json` antes de virar `Value`:
//! desserializar diretamente em mapas perderia propriedades duplicadas. A chave
//! do eleitor, quando fornecida pelo contexto confiável, deve coincidir com o
//! envelope. Este adaptador não decide cadastro, elegibilidade, prazo ou commit.

use std::{collections::HashSet, fmt};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use crypto_bigint::U2048;
use rand::rngs::OsRng;
use rsa::{
    BigUint, Pss, RsaPrivateKey, RsaPublicKey,
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePublicKey},
    traits::PublicKeyParts,
};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::domain::cryptography::elgamal::{
    ElGamalCiphertext, EncryptedBallot, group_modulus, validate_public_key as validate_elgamal_key,
};

const PREFIX: &[u8] = b"CAPP-ELECTIONS/1\n";
/// Limite do envelope original antes de qualquer interpretação.
pub const MAX_SIGNED_MESSAGE_BYTES: usize = 262_144;

/// Falhas públicas sem incluir chaves privadas ou o conteúdo recebido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CryptoWireError {
    /// JSON, esquema ou representação escalar inválidos.
    #[error("Codificação eleitoral inválida")]
    Encoding,
    /// RSA fora do perfil ou DER não canônico.
    #[error("Chave eleitoral inválida")]
    PublicKey,
    /// Assinatura não confere para a chave esperada.
    #[error("Assinatura eleitoral inválida")]
    Signature,
    /// Eleição, eleitor, cadastro, manifesto ou ordem de opções diverge.
    #[error("Cédula incompatível com o cadastro ou manifesto")]
    Context,
    /// Cifra fora do grupo permitido.
    #[error("Cifra eleitoral inválida")]
    Ciphertext,
    /// Configuração privada ou operação de assinatura indisponível.
    #[error("Assinatura do servidor indisponível")]
    Signing,
}

/// Contexto da cédula obtido da conta autenticada e do manifesto oficial.
#[derive(Debug, Clone)]
pub struct BallotContext {
    /// Eleição da rota e do manifesto.
    pub election_id: Uuid,
    /// Digest do manifesto oficial.
    pub manifest_hash: String,
    /// Identidade eleitoral vinculada à conta autenticada.
    pub voter_id: Uuid,
    /// Chave pública SPKI em Base64url do voto já aceito, quando houver.
    pub expected_public_key: Option<String>,
    /// Ordem imutável de opções, incluindo Nulo na última posição.
    pub option_ids: Vec<Uuid>,
}

/// Resultado de verificação; não afirma persistência nem validade binária do voto.
#[derive(Clone)]
pub struct VerifiedWireBallot {
    /// Envelope público original, com a assinatura do eleitor.
    pub envelope: Value,
    /// SHA-256 do prefixo e payload canônico.
    pub ballot_hash: String,
    /// SHA-256 do DER da chave pública da cédula.
    pub key_fingerprint: String,
    /// Cifras validadas, na ordem do manifesto, para acumulação transacional.
    pub ciphertexts: Vec<ElGamalCiphertext>,
}

impl fmt::Debug for VerifiedWireBallot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerifiedWireBallot")
            .field("ballot_hash", &self.ballot_hash)
            .field("key_fingerprint", &self.key_fingerprint)
            .field("ciphertext_count", &self.ciphertexts.len())
            .finish()
    }
}

/// Evidências a persistir atomicamente antes de responder ao eleitor.
#[derive(Debug, Clone)]
pub struct SignedRecord {
    /// Envelope assinado do registro público.
    pub board_entry: Value,
    /// Envelope assinado do recibo, preservado nas repetições.
    pub receipt: Value,
    /// Digest do payload do registro, incluindo a assinatura aninhada.
    pub entry_hash: String,
}

/// Manifesto autenticado e validado antes de congelar a configuração no banco.
#[derive(Debug, Clone)]
pub struct VerifiedManifest {
    /// Envelope original com assinatura do servidor confiável.
    pub envelope: Value,
    /// Digest do payload do manifesto.
    pub manifest_hash: String,
    /// Eleição da configuração.
    pub election_id: Uuid,
    /// IDs na ordem imutável do vetor cifrado.
    pub option_ids: Vec<Uuid>,
}

/// Lê JSON sem perder duplicatas e recusa números com fração/expoente/zero negativo.
/// Espaços e ordem de propriedades são permitidos: JCS é aplicado à assinatura.
pub fn parse_strict_json(raw: &[u8]) -> Result<Value, CryptoWireError> {
    if raw.len() > MAX_SIGNED_MESSAGE_BYTES {
        return Err(CryptoWireError::Encoding);
    }
    let mut decoder = serde_json::Deserializer::from_slice(raw);
    let value = StrictValue::deserialize(&mut decoder).map_err(|_| CryptoWireError::Encoding)?;
    decoder.end().map_err(|_| CryptoWireError::Encoding)?;
    Ok(value.0)
}

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("JSON sem propriedades duplicadas ou números fracionários")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Bool(v)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(v.to_owned())))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(v)))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                if v > 9_007_199_254_740_991 {
                    return Err(E::custom("inteiro fora de I-JSON"));
                }
                Ok(StrictValue(Value::from(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                if v.unsigned_abs() > 9_007_199_254_740_991 {
                    return Err(E::custom("inteiro fora de I-JSON"));
                }
                Ok(StrictValue(Value::from(v)))
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
                Err(E::custom("literal numérico não inteiro"))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element::<StrictValue>()? {
                    values.push(value.0);
                }
                Ok(StrictValue(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom("propriedade duplicada"));
                    }
                    values.insert(key, map.next_value::<StrictValue>()?.0);
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        decoder.deserialize_any(StrictVisitor)
    }
}

/// Bytes JCS do payload, sem o prefixo de domínio.
pub fn canonical_payload(payload: &Value) -> Result<Vec<u8>, CryptoWireError> {
    let bytes = serde_jcs::to_vec(payload).map_err(|_| CryptoWireError::Encoding)?;
    if bytes.len() > MAX_SIGNED_MESSAGE_BYTES {
        return Err(CryptoWireError::Encoding);
    }
    Ok(bytes)
}

fn signature_input(payload: &Value) -> Result<Vec<u8>, CryptoWireError> {
    let mut input = PREFIX.to_vec();
    input.extend(canonical_payload(payload)?);
    Ok(input)
}

/// Digest público de qualquer payload do contrato, excluindo a assinatura externa.
pub fn object_digest(payload: &Value) -> Result<String, CryptoWireError> {
    Ok(hex_digest(&signature_input(payload)?))
}

fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        write!(encoded, "{byte:02x}").expect("escrever String");
    }
    encoded
}

fn decode_base64(value: &str) -> Result<Vec<u8>, CryptoWireError> {
    if value.is_empty() || value.len() > 1024 {
        return Err(CryptoWireError::Encoding);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| CryptoWireError::Encoding)?;
    if URL_SAFE_NO_PAD.encode(&bytes) != value {
        return Err(CryptoWireError::Encoding);
    }
    Ok(bytes)
}

fn decode_public_key(encoded: &str) -> Result<(RsaPublicKey, Vec<u8>), CryptoWireError> {
    let bytes = decode_base64(encoded)?;
    let key = RsaPublicKey::from_public_key_der(&bytes).map_err(|_| CryptoWireError::PublicKey)?;
    check_rsa_profile(&key)?;
    // A reserialização também impõe DER/SPKI rsaEncryption canônico, sem cauda.
    let canonical = key
        .to_public_key_der()
        .map_err(|_| CryptoWireError::PublicKey)?;
    if canonical.as_bytes() != bytes {
        return Err(CryptoWireError::PublicKey);
    }
    Ok((key, bytes))
}

fn check_rsa_profile(key: &RsaPublicKey) -> Result<(), CryptoWireError> {
    if !matches!(key.n().bits(), 2048 | 3072 | 4096) || key.e() != &BigUint::from(65_537_u32) {
        return Err(CryptoWireError::PublicKey);
    }
    Ok(())
}

/// Confere SPKI, tamanho, expoente e Base64url; retorna fingerprint do DER.
pub fn validate_public_key(encoded: &str) -> Result<String, CryptoWireError> {
    let (_, bytes) = decode_public_key(encoded)?;
    Ok(hex_digest(&bytes))
}

fn exact_fields<'a>(
    value: &'a Value,
    fields: &[&str],
) -> Result<&'a Map<String, Value>, CryptoWireError> {
    let object = value.as_object().ok_or(CryptoWireError::Encoding)?;
    if object.len() != fields.len() || !fields.iter().all(|field| object.contains_key(*field)) {
        return Err(CryptoWireError::Encoding);
    }
    Ok(object)
}

fn string<'a>(object: &'a Value, name: &str) -> Result<&'a str, CryptoWireError> {
    object
        .get(name)
        .and_then(Value::as_str)
        .ok_or(CryptoWireError::Encoding)
}

fn version_kind(payload: &Value, kind: &str) -> Result<(), CryptoWireError> {
    if payload.get("version").and_then(Value::as_u64) != Some(1) || string(payload, "kind")? != kind
    {
        return Err(CryptoWireError::Encoding);
    }
    Ok(())
}

fn uuid_value(value: &str, allow_nil: bool) -> Result<Uuid, CryptoWireError> {
    let parsed = Uuid::parse_str(value).map_err(|_| CryptoWireError::Encoding)?;
    if parsed.to_string() != value || (!allow_nil && parsed.is_nil()) {
        return Err(CryptoWireError::Encoding);
    }
    Ok(parsed)
}

fn hash(value: &str) -> Result<String, CryptoWireError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(CryptoWireError::Encoding);
    }
    Ok(value.to_owned())
}

fn decimal(value: &str) -> Result<U2048, CryptoWireError> {
    if value.is_empty()
        || value.len() > 617
        || !value.bytes().all(|b| b.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(CryptoWireError::Encoding);
    }
    U2048::from_str_radix_vartime(value, 10).map_err(|_| CryptoWireError::Ciphertext)
}

/// Confere somente envelope e assinatura; contexto/esquema do payload são separados.
/// `envelope` deve vir de `parse_strict_json` quando sua origem for não confiável.
pub fn verify_signed_object(
    envelope: &Value,
    trusted_public_key: &str,
) -> Result<(), CryptoWireError> {
    exact_fields(envelope, &["payload", "signature"])?;
    if !envelope["payload"].is_object() {
        return Err(CryptoWireError::Encoding);
    }
    let (public_key, _) = decode_public_key(trusted_public_key)?;
    let signature = decode_base64(string(envelope, "signature")?)?;
    let digest = Sha256::digest(signature_input(&envelope["payload"])?);
    public_key
        .verify(Pss::new_with_salt::<Sha256>(32), &digest, &signature)
        .map_err(|_| CryptoWireError::Signature)
}

/// Prova privada de posse destinada à conta autenticada e à eleição da rota.
pub fn verify_registration(
    raw: &[u8],
    election_id: Uuid,
    account_id: Uuid,
) -> Result<String, CryptoWireError> {
    let envelope = parse_strict_json(raw)?;
    let payload = &envelope["payload"];
    exact_fields(
        payload,
        &["version", "kind", "electionId", "accountId", "publicKey"],
    )?;
    version_kind(payload, "voter-registration")?;
    if uuid_value(string(payload, "electionId")?, false)? != election_id
        || uuid_value(string(payload, "accountId")?, false)? != account_id
    {
        return Err(CryptoWireError::Context);
    }
    let public_key = string(payload, "publicKey")?;
    verify_signed_object(&envelope, public_key)?;
    Ok(public_key.to_owned())
}

fn group_parameters() -> &'static (BigUint, BigUint) {
    static PARAMETERS: std::sync::OnceLock<(BigUint, BigUint)> = std::sync::OnceLock::new();
    PARAMETERS.get_or_init(|| {
        let p = BigUint::from_bytes_be(group_modulus().to_be_bytes().as_slice());
        let q = (&p - BigUint::from(1_u8)) >> 1;
        (p, q)
    })
}

fn group_element(encoded: &str) -> Result<BigUint, CryptoWireError> {
    let value = decimal(encoded)?;
    validate_elgamal_key("rfc3526-modp14", &value).map_err(|_| CryptoWireError::Ciphertext)?;
    Ok(BigUint::from_bytes_be(value.to_be_bytes().as_slice()))
}

/// Confere assinatura com âncora externa, configuração, grupo e provas públicas.
/// Não gera chaves/provas nem decide autorização administrativa para abrir eleição.
pub fn verify_manifest(
    raw: &[u8],
    trusted_server_key: &str,
) -> Result<VerifiedManifest, CryptoWireError> {
    let envelope = parse_strict_json(raw)?;
    verify_signed_object(&envelope, trusted_server_key)?;
    let payload = &envelope["payload"];
    exact_fields(
        payload,
        &[
            "version",
            "kind",
            "electionId",
            "academicCenterId",
            "groupId",
            "options",
            "authorities",
            "combinedPublicKey",
            "serverSigningKey",
            "calendarOriginHash",
        ],
    )?;
    version_kind(payload, "manifest")?;
    let election_id = uuid_value(string(payload, "electionId")?, false)?;
    uuid_value(string(payload, "academicCenterId")?, false)?;
    hash(string(payload, "calendarOriginHash")?)?;
    if string(payload, "serverSigningKey")? != trusted_server_key
        || string(payload, "groupId")? != "rfc3526-modp14"
    {
        return Err(CryptoWireError::Context);
    }
    let options = payload["options"]
        .as_array()
        .ok_or(CryptoWireError::Encoding)?;
    if !(2..=51).contains(&options.len()) {
        return Err(CryptoWireError::Context);
    }
    let mut option_ids = Vec::with_capacity(options.len());
    for (index, option) in options.iter().enumerate() {
        exact_fields(option, &["id", "kind", "name"])?;
        let id = uuid_value(string(option, "id")?, true)?;
        let name = string(option, "name")?;
        if name.is_empty() || name.chars().count() > 200 {
            return Err(CryptoWireError::Encoding);
        }
        if index == options.len() - 1 {
            if !id.is_nil() || string(option, "kind")? != "null" || name != "Nulo" {
                return Err(CryptoWireError::Context);
            }
        } else if id.is_nil() || string(option, "kind")? != "slate" {
            return Err(CryptoWireError::Context);
        }
        option_ids.push(id);
    }
    if option_ids.iter().collect::<HashSet<_>>().len() != option_ids.len() {
        return Err(CryptoWireError::Context);
    }
    let authorities = payload["authorities"]
        .as_array()
        .ok_or(CryptoWireError::Encoding)?;
    if authorities.len() != options.len() + 1 {
        return Err(CryptoWireError::Context);
    }
    let mut ids = HashSet::new();
    let mut keys = HashSet::new();
    let mut public_values = HashSet::new();
    let mut slates = HashSet::new();
    let (mut server_count, mut commission_count) = (0, 0);
    let (prime, order) = group_parameters();
    let mut combined = BigUint::from(1_u8);
    for authority in authorities {
        exact_fields(
            authority,
            &[
                "id",
                "role",
                "slateId",
                "signingKey",
                "publicValue",
                "keyProof",
            ],
        )?;
        let id = uuid_value(string(authority, "id")?, false)?;
        let key = string(authority, "signingKey")?;
        let public_encoded = string(authority, "publicValue")?;
        validate_public_key(key)?;
        if !ids.insert(id) || !keys.insert(key) || !public_values.insert(public_encoded) {
            return Err(CryptoWireError::Context);
        }
        match string(authority, "role")? {
            "server" => {
                server_count += 1;
                if !authority["slateId"].is_null() {
                    return Err(CryptoWireError::Context);
                }
            }
            "commission" => {
                commission_count += 1;
                if !authority["slateId"].is_null() {
                    return Err(CryptoWireError::Context);
                }
            }
            "slate" => {
                if !slates.insert(uuid_value(string(authority, "slateId")?, false)?) {
                    return Err(CryptoWireError::Context);
                }
            }
            _ => return Err(CryptoWireError::Context),
        }
        let proof = &authority["keyProof"];
        exact_fields(proof, &["commitment", "response"])?;
        let public = group_element(public_encoded)?;
        let commitment = group_element(string(proof, "commitment")?)?;
        let response = BigUint::from_bytes_be(
            decimal(string(proof, "response")?)?
                .to_be_bytes()
                .as_slice(),
        );
        if &response >= order {
            return Err(CryptoWireError::Ciphertext);
        }
        let challenge_payload = json!({
            "version":1,"kind":"authority-key-proof","electionId":payload["electionId"],
            "groupId":payload["groupId"],"authorityId":authority["id"],"role":authority["role"],
            "slateId":authority["slateId"],"signingKey":authority["signingKey"],
            "publicValue":authority["publicValue"],"commitment":proof["commitment"],
        });
        let challenge =
            BigUint::from_bytes_be(&Sha256::digest(signature_input(&challenge_payload)?)) % order;
        if BigUint::from(2_u8).modpow(&response, prime)
            != commitment * public.modpow(&challenge, prime) % prime
        {
            return Err(CryptoWireError::Signature);
        }
        combined = combined * public % prime;
    }
    let expected_slates: HashSet<_> = option_ids[..option_ids.len() - 1].iter().copied().collect();
    if server_count != 1
        || commission_count != 1
        || slates != expected_slates
        || group_element(string(payload, "combinedPublicKey")?)? != combined
    {
        return Err(CryptoWireError::Context);
    }
    let manifest_hash = object_digest(payload)?;
    Ok(VerifiedManifest {
        envelope,
        manifest_hash,
        election_id,
        option_ids,
    })
}

fn timestamp(value: &str) -> Result<chrono::DateTime<chrono::Utc>, CryptoWireError> {
    let parsed = chrono::DateTime::parse_from_rfc3339(value)
        .map_err(|_| CryptoWireError::Encoding)?
        .with_timezone(&chrono::Utc);
    if parsed.format("%Y-%m-%dT%H:%M:%SZ").to_string() != value {
        return Err(CryptoWireError::Encoding);
    }
    Ok(parsed)
}

/// Confere calendário original assinado e sua referência no manifesto congelado.
pub fn verify_calendar_origin(
    raw: &[u8],
    manifest: &VerifiedManifest,
    trusted_server_key: &str,
) -> Result<Value, CryptoWireError> {
    let envelope = parse_strict_json(raw)?;
    verify_signed_object(&envelope, trusted_server_key)?;
    let payload = &envelope["payload"];
    exact_fields(
        payload,
        &["version", "kind", "electionId", "startsAt", "endsAt"],
    )?;
    version_kind(payload, "calendar-origin")?;
    if uuid_value(string(payload, "electionId")?, false)? != manifest.election_id
        || object_digest(payload)? != string(&manifest.envelope["payload"], "calendarOriginHash")?
        || timestamp(string(payload, "startsAt")?)? >= timestamp(string(payload, "endsAt")?)?
    {
        return Err(CryptoWireError::Context);
    }
    Ok(envelope)
}

/// Confere fechamento contra a cabeça/contagem e calendário já validados.
/// Não implementa política de prorrogação; o chamador fornece o calendário efetivo.
pub fn verify_closure(
    raw: &[u8],
    manifest: &VerifiedManifest,
    schedule: &Value,
    expected_head: &str,
    expected_count: u32,
    trusted_server_key: &str,
) -> Result<Value, CryptoWireError> {
    let envelope = parse_strict_json(raw)?;
    verify_signed_object(&envelope, trusted_server_key)?;
    verify_signed_object(schedule, trusted_server_key)?;
    let payload = &envelope["payload"];
    exact_fields(
        payload,
        &[
            "version",
            "kind",
            "electionId",
            "manifestHash",
            "scheduleHash",
            "headHash",
            "voterCount",
            "closedAt",
        ],
    )?;
    version_kind(payload, "closure")?;
    hash(expected_head)?;
    if !matches!(
        string(&schedule["payload"], "kind")?,
        "calendar-origin" | "schedule-change"
    ) || uuid_value(string(&schedule["payload"], "electionId")?, false)? != manifest.election_id
        || expected_count > 100_000
        || (expected_count == 0 && expected_head != manifest.manifest_hash)
        || uuid_value(string(payload, "electionId")?, false)? != manifest.election_id
        || string(payload, "manifestHash")? != manifest.manifest_hash
        || string(payload, "scheduleHash")? != object_digest(&schedule["payload"])?
        || string(payload, "headHash")? != expected_head
        || payload["voterCount"].as_u64() != Some(u64::from(expected_count))
        || timestamp(string(payload, "closedAt")?)?
            < timestamp(string(&schedule["payload"], "endsAt")?)?
    {
        return Err(CryptoWireError::Context);
    }
    Ok(envelope)
}

/// Verifica uma cédula confidencial contra manifesto e chave esperada, se presente.
pub fn verify_ballot(
    raw: &[u8],
    context: &BallotContext,
) -> Result<VerifiedWireBallot, CryptoWireError> {
    verify_ballot_value(parse_strict_json(raw)?, context)
}

fn verify_ballot_value(
    envelope: Value,
    context: &BallotContext,
) -> Result<VerifiedWireBallot, CryptoWireError> {
    exact_fields(&envelope, &["payload", "signature"])?;
    let payload = &envelope["payload"];
    exact_fields(
        payload,
        &[
            "version",
            "kind",
            "electionId",
            "manifestHash",
            "voterId",
            "submissionId",
            "publicKey",
            "ciphertexts",
        ],
    )?;
    version_kind(payload, "ballot")?;
    let election_id = uuid_value(string(payload, "electionId")?, false)?;
    let voter_id = uuid_value(string(payload, "voterId")?, false)?;
    let submission_id = uuid_value(string(payload, "submissionId")?, false)?;
    let manifest_hash = hash(string(payload, "manifestHash")?)?;
    let public_key = string(payload, "publicKey")?;
    if election_id != context.election_id
        || voter_id != context.voter_id
        || manifest_hash != context.manifest_hash
        || context
            .expected_public_key
            .as_deref()
            .is_some_and(|expected| public_key != expected)
    {
        return Err(CryptoWireError::Context);
    }
    let key_fingerprint = validate_public_key(public_key)?;
    verify_signed_object(&envelope, public_key)?;
    let values = payload["ciphertexts"]
        .as_array()
        .ok_or(CryptoWireError::Encoding)?;
    if !(2..=51).contains(&values.len())
        || values.len() != context.option_ids.len()
        || context.option_ids.last() != Some(&Uuid::nil())
        || context.option_ids.iter().collect::<HashSet<_>>().len() != context.option_ids.len()
    {
        return Err(CryptoWireError::Context);
    }
    let mut crypto_values = Vec::with_capacity(values.len());
    for (value, expected_option) in values.iter().zip(&context.option_ids) {
        exact_fields(value, &["optionId", "a", "b"])?;
        let option_id = uuid_value(string(value, "optionId")?, true)?;
        if option_id != *expected_option {
            return Err(CryptoWireError::Context);
        }
        crypto_values.push(ElGamalCiphertext {
            a: decimal(string(value, "a")?)?,
            b: decimal(string(value, "b")?)?,
        });
    }
    EncryptedBallot::new(crypto_values.clone()).map_err(|_| CryptoWireError::Ciphertext)?;
    let ballot_hash = object_digest(payload)?;
    let _ = submission_id;
    Ok(VerifiedWireBallot {
        envelope,
        ballot_hash,
        key_fingerprint,
        ciphertexts: crypto_values,
    })
}

/// Chave RSA do servidor, injetada pela configuração; nunca gerada por requisição.
/// Não implementa Debug/Serialize para evitar exposição acidental da chave.
#[derive(Clone)]
pub struct ServerSigner {
    key: RsaPrivateKey,
}

impl ServerSigner {
    /// Carrega PKCS#8 PEM e confere os parâmetros exigidos pelo protocolo.
    pub fn from_pkcs8_pem(pem: &str) -> Result<Self, CryptoWireError> {
        let key = RsaPrivateKey::from_pkcs8_pem(pem).map_err(|_| CryptoWireError::Signing)?;
        key.validate().map_err(|_| CryptoWireError::Signing)?;
        check_rsa_profile(&RsaPublicKey::from(&key)).map_err(|_| CryptoWireError::Signing)?;
        Ok(Self { key })
    }

    /// Chave pública SPKI DER em Base64url canônico, sem padding.
    pub fn public_key_base64(&self) -> Result<String, CryptoWireError> {
        let der = RsaPublicKey::from(&self.key)
            .to_public_key_der()
            .map_err(|_| CryptoWireError::Signing)?;
        Ok(URL_SAFE_NO_PAD.encode(der.as_bytes()))
    }

    /// Assina payload já autorizado pelo caso de uso. Não decide sua validade.
    pub fn sign_payload(&self, payload: &Value) -> Result<Value, CryptoWireError> {
        if !payload.is_object() {
            return Err(CryptoWireError::Encoding);
        }
        let digest = Sha256::digest(signature_input(payload)?);
        let signature = self
            .key
            .sign_with_rng(
                &mut OsRng,
                Pss::new_blinded_with_salt::<Sha256>(32),
                &digest,
            )
            .map_err(|_| CryptoWireError::Signing)?;
        let envelope = json!({"payload":payload,"signature":URL_SAFE_NO_PAD.encode(signature)});
        if serde_json::to_vec(&envelope)
            .map_err(|_| CryptoWireError::Encoding)?
            .len()
            > MAX_SIGNED_MESSAGE_BYTES
        {
            return Err(CryptoWireError::Encoding);
        }
        Ok(envelope)
    }

    /// Prepara registro e recibo. O chamador persiste ambos antes de devolvê-los.
    pub fn issue_record(
        &self,
        ballot: &VerifiedWireBallot,
        index: u32,
        previous_hash: &str,
    ) -> Result<SignedRecord, CryptoWireError> {
        hash(previous_hash)?;
        let payload = &ballot.envelope["payload"];
        if !(1..=100_000).contains(&index)
            || (index == 1 && previous_hash != string(payload, "manifestHash")?)
        {
            return Err(CryptoWireError::Context);
        }
        let entry_payload = json!({
            "version":1,"kind":"board-entry","electionId":payload["electionId"],
            "manifestHash":payload["manifestHash"],"index":index,"previousHash":previous_hash,
            "ballot":ballot.envelope,
        });
        let entry_hash = object_digest(&entry_payload)?;
        let board_entry = self.sign_payload(&entry_payload)?;
        let receipt = self.sign_payload(&json!({
            "version":1,"kind":"receipt","electionId":payload["electionId"],
            "manifestHash":payload["manifestHash"],"ballotHash":ballot.ballot_hash,
            "index":index,"entryHash":entry_hash,
        }))?;
        Ok(SignedRecord {
            board_entry,
            receipt,
            entry_hash,
        })
    }
}

/// Confere registro, recibo, assinaturas e elo esperado ao ler evidências públicas.
/// A aplicação fornece cadastro/manifesto e predecessor obtidos de fontes confiáveis.
pub fn verify_record(
    board_raw: &[u8],
    receipt_raw: &[u8],
    context: &BallotContext,
    expected_index: u32,
    expected_previous_hash: &str,
    server_public_key: &str,
) -> Result<SignedRecord, CryptoWireError> {
    let board_entry = parse_strict_json(board_raw)?;
    let receipt = parse_strict_json(receipt_raw)?;
    verify_signed_object(&board_entry, server_public_key)?;
    verify_signed_object(&receipt, server_public_key)?;
    let entry = &board_entry["payload"];
    let issued = &receipt["payload"];
    exact_fields(
        entry,
        &[
            "version",
            "kind",
            "electionId",
            "manifestHash",
            "index",
            "previousHash",
            "ballot",
        ],
    )?;
    exact_fields(
        issued,
        &[
            "version",
            "kind",
            "electionId",
            "manifestHash",
            "ballotHash",
            "index",
            "entryHash",
        ],
    )?;
    version_kind(entry, "board-entry")?;
    version_kind(issued, "receipt")?;
    let ballot = verify_ballot_value(entry["ballot"].clone(), context)?;
    let entry_hash = object_digest(entry)?;
    hash(expected_previous_hash)?;
    if !(1..=100_000).contains(&expected_index)
        || (expected_index == 1 && expected_previous_hash != context.manifest_hash)
        || entry["index"].as_u64() != Some(u64::from(expected_index))
        || issued["index"] != entry["index"]
        || string(entry, "previousHash")? != expected_previous_hash
        || string(issued, "ballotHash")? != ballot.ballot_hash
        || string(issued, "entryHash")? != entry_hash
    {
        return Err(CryptoWireError::Context);
    }
    for value in [entry, issued] {
        if uuid_value(string(value, "electionId")?, false)? != context.election_id
            || string(value, "manifestHash")? != context.manifest_hash
        {
            return Err(CryptoWireError::Context);
        }
    }
    Ok(SignedRecord {
        board_entry,
        receipt,
        entry_hash,
    })
}

#[cfg(test)]
#[path = "crypto_wire/tests.rs"]
mod tests;
