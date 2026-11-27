//! DTOs da API eleitoral.
//!
//! Os tipos de entrada representam dados já desserializados e usam tipos
//! adequados, sem transportar Base64url, UUIDs ou números grandes como String.
//! As regras sobre a eleição, a assinatura e a participação pertencem ao domínio
//! e aos casos de uso, não à DTO.

use ::serde::{Deserialize, Serialize};
use crypto_bigint::U2048;
use uuid::Uuid;

use crate::modules::elections::{
    api::serde as wire_serde,
    application::{
        preparation::AuthorityRegistrationProof,
        registration::{RegistrationProof, RegistrationStatus},
        tally::{OptionShare, ShareSubmission},
    },
    domain::{
        Fingerprint,
        cryptography::elgamal::{DecryptionShare, ElGamalCiphertext},
        entities::{
            authority::{Authority, AuthorityRole, KeyProof},
            ballot::{BallotCiphertexts, BallotContent, Ciphertext, SignedBallot},
        },
        identifiers::{AccountId, AuthorityId, ElectionId, SlateId},
    },
};

/// Envelope de uma cédula confidencial recebida pela API.
///
/// A assinatura cobre a mensagem criptográfica reconstruída a partir deste
/// payload e dos IDs confiáveis da rota e da conta autenticada. A codificação
/// para assinatura pertence ao adaptador criptográfico, não à DTO.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BallotRequestDto {
    pub(super) payload: BallotPayloadDto,
    #[serde(deserialize_with = "wire_serde::base64url")]
    pub(super) signature: Vec<u8>,
}

/// O corpo transporta as cifras, a chave e o hash do manifesto. A assinatura
/// também cobre os IDs obtidos na rota e na autenticação e os campos do protocolo.
///
/// O manifesto permanece identificado pelo digest autenticado. A chave
/// pública acompanha a cédula nas duas modalidades de cadastro da eleição.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct BallotPayloadDto {
    #[serde(deserialize_with = "wire_serde::sha256_hex")]
    pub(super) manifest_hash: Fingerprint,

    #[serde(deserialize_with = "wire_serde::base64url")]
    pub(super) public_key: Vec<u8>,

    pub(super) ciphertexts: Vec<BallotCiphertextDto>,
}

/// Uma posição da cédula, associada à respectiva opção do manifesto.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct BallotCiphertextDto {
    /// O UUID nulo é permitido apenas como representação da opção Nulo.
    /// Sua posição e sua pertinência ao manifesto são verificadas no domínio.
    #[serde(deserialize_with = "wire_serde::option_uuid")]
    pub(super) option_id: Uuid,

    #[serde(deserialize_with = "wire_serde::decimal_u2048")]
    pub(super) a: U2048,

    #[serde(deserialize_with = "wire_serde::decimal_u2048")]
    pub(super) b: U2048,
}

/// Falhas na passagem da representação HTTP para as invariantes do domínio.
#[derive(Debug, thiserror::Error)]
pub(super) enum BallotDtoError {
    #[error("Cifras eleitorais inválidas")]
    InvalidCiphertexts,
}

/// A fronteira HTTP associa os dados recebidos à eleição da rota e à conta
/// autenticada. Nenhuma identidade do eleitor é aceita do corpo da requisição.
///
/// A entidade resultante ainda não foi verificada criptograficamente.
impl BallotRequestDto {
    pub(super) fn into_signed_ballot(
        self,
        election_id: ElectionId,
        account_id: AccountId,
    ) -> Result<SignedBallot, BallotDtoError> {
        let payload = self.payload;

        let items = payload
            .ciphertexts
            .into_iter()
            .map(|item| {
                Ciphertext::new(
                    item.option_id,
                    ElGamalCiphertext {
                        a: item.a,
                        b: item.b,
                    },
                )
            })
            .collect();

        let ciphertexts =
            BallotCiphertexts::try_new(items).map_err(|_| BallotDtoError::InvalidCiphertexts)?;

        Ok(SignedBallot::new(
            election_id,
            account_id,
            payload.public_key,
            self.signature,
            BallotContent::Confidential {
                manifest_hash: payload.manifest_hash,
                ciphertexts,
            },
        ))
    }
}

/// Prova de cadastro enviada pelo dispositivo, sem identidade informada no corpo.
///
/// Serde rejeita propriedades repetidas, desconhecidas e tipos incorretos.
/// A validação da assinatura pertence ao verificador criptográfico.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegistrationRequestDto {
    payload: RegistrationPayloadDto,
    #[serde(deserialize_with = "wire_serde::base64url")]
    signature: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RegistrationPayloadDto {
    #[serde(deserialize_with = "wire_serde::base64url")]
    public_key: Vec<u8>,
}

impl RegistrationRequestDto {
    pub(super) fn into_proof(self) -> RegistrationProof {
        RegistrationProof::new(self.payload.public_key, self.signature)
    }
}

/// Envelope assinado do cadastro de uma autoridade. O formato público existente
/// é preservado; os campos constantes são conferidos durante a desserialização.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuthorityRegistrationRequestDto {
    payload: AuthorityRegistrationPayloadDto,
    #[serde(deserialize_with = "wire_serde::base64url")]
    signature: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthorityRegistrationPayloadDto {
    #[serde(rename = "version", deserialize_with = "wire_serde::version_one")]
    _version: (),
    #[serde(rename = "kind")]
    _kind: AuthorityRegistrationKind,
    #[serde(deserialize_with = "wire_serde::non_nil_uuid")]
    election_id: Uuid,
    authority: AuthorityDto,
}

#[derive(Debug, Deserialize)]
enum AuthorityRegistrationKind {
    #[serde(rename = "authority-registration")]
    Registration,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AuthorityDto {
    #[serde(deserialize_with = "wire_serde::non_nil_uuid")]
    id: Uuid,
    role: AuthorityRoleDto,
    #[serde(deserialize_with = "wire_serde::nullable_non_nil_uuid")]
    slate_id: Option<Uuid>,
    #[serde(deserialize_with = "wire_serde::base64url")]
    signing_key: Vec<u8>,
    #[serde(deserialize_with = "wire_serde::decimal_u2048")]
    public_value: U2048,
    key_proof: KeyProofDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AuthorityRoleDto {
    Commission,
    Slate,
    Server,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyProofDto {
    #[serde(deserialize_with = "wire_serde::decimal_u2048")]
    commitment: U2048,
    #[serde(deserialize_with = "wire_serde::decimal_u2048")]
    response: U2048,
}

/// Falha de conversão de dados tipados para as invariantes do domínio.
#[derive(Debug, thiserror::Error)]
#[error("Identificador, papel ou valor de autoridade inválido")]
pub(super) struct AuthorityDtoError;

impl AuthorityRegistrationRequestDto {
    pub(super) fn into_proof(self) -> Result<AuthorityRegistrationProof, AuthorityDtoError> {
        let dto = self.payload.authority;
        let role = match (dto.role, dto.slate_id) {
            (AuthorityRoleDto::Commission, None) => AuthorityRole::Commission,
            (AuthorityRoleDto::Server, None) => AuthorityRole::Server,
            (AuthorityRoleDto::Slate, Some(id)) => {
                AuthorityRole::Slate(SlateId::try_from_uuid(id).map_err(|_| AuthorityDtoError)?)
            }
            _ => return Err(AuthorityDtoError),
        };
        let authority = Authority::try_new(
            AuthorityId::try_from_uuid(dto.id).map_err(|_| AuthorityDtoError)?,
            role,
            dto.signing_key,
            dto.public_value,
            KeyProof::new(dto.key_proof.commitment, dto.key_proof.response),
        )
        .map_err(|_| AuthorityDtoError)?;
        Ok(AuthorityRegistrationProof {
            election: ElectionId::try_from_uuid(self.payload.election_id)
                .map_err(|_| AuthorityDtoError)?,
            authority,
            signature: self.signature,
        })
    }
}

/// Envelope assinado de uma contribuição de decifração.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ShareRequestDto {
    payload: SharePayloadDto,
    #[serde(deserialize_with = "wire_serde::base64url")]
    signature: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SharePayloadDto {
    #[serde(rename = "version", deserialize_with = "wire_serde::version_one")]
    _version: (),
    #[serde(rename = "kind")]
    _kind: ShareKind,
    #[serde(deserialize_with = "wire_serde::non_nil_uuid")]
    election_id: Uuid,
    #[serde(deserialize_with = "wire_serde::sha256_hex")]
    manifest_hash: Fingerprint,
    #[serde(deserialize_with = "wire_serde::sha256_hex")]
    closure_hash: Fingerprint,
    #[serde(deserialize_with = "wire_serde::sha256_hex")]
    aggregate_hash: Fingerprint,
    #[serde(deserialize_with = "wire_serde::non_nil_uuid")]
    authority_id: Uuid,
    shares: Vec<OptionShareDto>,
}

#[derive(Debug, Deserialize)]
enum ShareKind {
    #[serde(rename = "decryption-share")]
    Decryption,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OptionShareDto {
    #[serde(deserialize_with = "wire_serde::option_uuid")]
    option_id: Uuid,
    #[serde(deserialize_with = "wire_serde::decimal_u2048")]
    value: U2048,
    proof: DecryptionProofDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DecryptionProofDto {
    #[serde(deserialize_with = "wire_serde::decimal_u2048")]
    commitment_g: U2048,
    #[serde(deserialize_with = "wire_serde::decimal_u2048")]
    commitment_a: U2048,
    #[serde(deserialize_with = "wire_serde::decimal_u2048")]
    response: U2048,
}

impl ShareRequestDto {
    pub(super) fn into_submission(self) -> Result<ShareSubmission, AuthorityDtoError> {
        let payload = self.payload;
        Ok(ShareSubmission {
            election: ElectionId::try_from_uuid(payload.election_id)
                .map_err(|_| AuthorityDtoError)?,
            authority: AuthorityId::try_from_uuid(payload.authority_id)
                .map_err(|_| AuthorityDtoError)?,
            manifest_hash: payload.manifest_hash,
            closure_hash: payload.closure_hash,
            aggregate_hash: payload.aggregate_hash,
            shares: payload
                .shares
                .into_iter()
                .map(|item| OptionShare {
                    option_id: item.option_id,
                    share: DecryptionShare {
                        value: item.value,
                        commitment_g: item.proof.commitment_g,
                        commitment_a: item.proof.commitment_a,
                        response: item.proof.response,
                    },
                })
                .collect(),
            signature: self.signature,
        })
    }
}

/// Situação do cadastro ou do voto já confirmado para a conta autenticada.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RegistrationResponse {
    requires_registration: bool,
    voter_id: Option<Uuid>,
    #[serde(serialize_with = "wire_serde::optional_base64url")]
    public_key: Option<Vec<u8>>,
    participated: bool,
}

impl From<RegistrationStatus> for RegistrationResponse {
    fn from(status: RegistrationStatus) -> Self {
        Self {
            requires_registration: status.requires_registration,
            voter_id: status.account_id.map(AccountId::into_uuid),
            public_key: status.public_key,
            participated: status.participated,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::elections::api::dto::*;

    #[test]
    fn registration_dto_decodes_bytes_and_rejects_ambiguous_input() {
        let good = r#"{"payload":{"publicKey":"AQ"},"signature":"Ag"}"#;
        let dto: RegistrationRequestDto = serde_json::from_str(good).unwrap();
        let proof = dto.into_proof();
        assert_eq!(proof.public_key(), &[1]);
        assert_eq!(proof.signature(), &[2]);
        for bad in [
            r#"{"payload":{"publicKey":"AQ","publicKey":"AQ"},"signature":"Ag"}"#,
            r#"{"payload":{"publicKey":"AQ","accountId":"x"},"signature":"Ag"}"#,
            r#"{"payload":{"publicKey":"AQ"},"signature":"Ag="}"#,
            r#"{"payload":{"publicKey":"AQ"},"signature":"Ag","signature":"Ag"}"#,
            r#"{"payload":{"publicKey":"AQ"},"signature":"Ag","kind":"registration"}"#,
        ] {
            assert!(
                serde_json::from_str::<RegistrationRequestDto>(bad).is_err(),
                "{bad}"
            );
        }
    }

    fn authority_json() -> String {
        r#"{"payload":{"version":1,"kind":"authority-registration",
            "electionId":"00000000-0000-0000-0000-000000000001",
            "authority":{"id":"00000000-0000-0000-0000-000000000002",
            "role":"commission","slateId":null,"signingKey":"AQ",
            "publicValue":"4","keyProof":{"commitment":"2","response":"0"}}},
            "signature":"Ag"}"#
            .to_owned()
    }

    #[test]
    fn authority_dto_uses_domain_values() {
        let dto: AuthorityRegistrationRequestDto = serde_json::from_str(&authority_json()).unwrap();
        let proof = dto.into_proof().unwrap();
        assert_eq!(proof.election.into_uuid(), Uuid::from_u128(1));
        assert_eq!(proof.authority.id().into_uuid(), Uuid::from_u128(2));
        assert_eq!(proof.authority.role(), AuthorityRole::Commission);
        assert_eq!(proof.authority.signing_key(), &[1]);
        assert_eq!(*proof.authority.public_value(), U2048::from_u8(4));
        assert_eq!(proof.signature, vec![2]);
    }

    #[test]
    fn authority_dto_rejects_ambiguous_and_wrong_representations() {
        let raw = authority_json();
        for bad in [
            raw.replace("\"version\":1", "\"version\":2"),
            raw.replace("authority-registration", "decryption-share"),
            raw.replace("\"version\":1,", ""),
            raw.replace("\"slateId\":null,", ""),
            raw.replace(
                "\"commitment\":\"2\"",
                "\"commitment\":\"2\",\"commitment\":\"4\"",
            ),
            raw.replace("\"signingKey\":\"AQ\"", "\"signingKey\":\"AQ=\""),
            raw.replace("\"response\":\"0\"", "\"response\":\"00\""),
            raw.replace("\"response\":\"0\"", "\"response\":\"0\",\"extra\":1"),
        ] {
            assert!(
                serde_json::from_str::<AuthorityRegistrationRequestDto>(&bad).is_err(),
                "{bad}"
            );
        }
        let wrong_role = raw.replace("\"role\":\"commission\"", "\"role\":\"slate\"");
        let dto: AuthorityRegistrationRequestDto = serde_json::from_str(&wrong_role).unwrap();
        assert!(dto.into_proof().is_err());
    }

    fn share_json() -> String {
        serde_json::json!({
            "payload": {
                "version": 1, "kind": "decryption-share",
                "electionId": Uuid::from_u128(1), "authorityId": Uuid::from_u128(2),
                "manifestHash": "ab".repeat(32), "closureHash": "cd".repeat(32),
                "aggregateHash": "ef".repeat(32),
                "shares": [{
                    "optionId": Uuid::nil(), "value": "1",
                    "proof": {"commitmentG": "2", "commitmentA": "1", "response": "0"},
                }],
            }, "signature": "Ag",
        })
        .to_string()
    }

    #[test]
    fn share_dto_decodes_proofs_and_context() {
        let dto: ShareRequestDto = serde_json::from_str(&share_json()).unwrap();
        let submission = dto.into_submission().unwrap();
        assert_eq!(submission.election.into_uuid(), Uuid::from_u128(1));
        assert_eq!(submission.authority.into_uuid(), Uuid::from_u128(2));
        assert_eq!(submission.manifest_hash, [0xab; 32]);
        assert_eq!(submission.closure_hash, [0xcd; 32]);
        assert_eq!(submission.aggregate_hash, [0xef; 32]);
        assert!(submission.shares[0].option_id.is_nil());
        assert_eq!(submission.shares[0].share.value, U2048::ONE);
        assert_eq!(submission.shares[0].share.response, U2048::ZERO);
        assert_eq!(submission.signature, vec![2]);
    }

    #[test]
    fn share_dto_rejects_malformed_nested_fields() {
        let raw = share_json();
        for bad in [
            raw.replace("decryption-share", "authority-registration"),
            raw.replace("\"version\":1", "\"version\":2"),
            raw.replace(
                "\"commitmentG\":\"2\"",
                "\"commitmentG\":\"2\",\"commitmentG\":\"2\"",
            ),
            raw.replace("\"response\":\"0\"", "\"response\":0"),
            raw.replace("\"value\":\"1\"", "\"value\":\"01\""),
            raw.replace(&"ab".repeat(32), &"AB".repeat(32)),
            raw.replace(
                "\"signature\":\"Ag\"",
                "\"signature\":\"Ag\",\"extra\":true",
            ),
        ] {
            assert!(
                serde_json::from_str::<ShareRequestDto>(&bad).is_err(),
                "{bad}"
            );
        }
    }

    fn example_json() -> String {
        format!(
            r#"{{
                "payload": {{
                    "manifestHash":"{}",
                    "publicKey":"AQ",
                    "ciphertexts":[
                        {{"optionId":"00000000-0000-0000-0000-000000000004","a":"2","b":"4"}},
                        {{"optionId":"00000000-0000-0000-0000-000000000000","a":"8","b":"16"}}
                    ]
                }},
                "signature":"Ag"
            }}"#,
            "ab".repeat(32)
        )
    }

    #[test]
    fn decodes_typed_fields() {
        let dto: BallotRequestDto = serde_json::from_str(&example_json()).unwrap();
        assert_eq!(dto.payload.manifest_hash, [0xab; 32]);
        assert_eq!(dto.payload.public_key, vec![1]);
        assert_eq!(dto.signature, vec![2]);
        assert_eq!(dto.payload.ciphertexts[0].a, U2048::from_u8(2));
        assert!(dto.payload.ciphertexts[1].option_id.is_nil());
    }

    #[test]
    fn identifiers_come_from_authenticated_context() {
        let dto: BallotRequestDto = serde_json::from_str(&example_json()).unwrap();
        let election = ElectionId::try_from_uuid(Uuid::from_u128(99)).unwrap();
        let account = AccountId::try_from_uuid(Uuid::from_u128(98)).unwrap();
        let ballot = dto.into_signed_ballot(election, account).unwrap();
        assert_eq!(ballot.election_id(), election);
        assert_eq!(ballot.account_id(), account);
    }

    #[test]
    fn rejects_duplicate_and_unknown_fields() {
        let original = example_json();
        let duplicate = original.replace(
            "\"signature\":\"Ag\"",
            "\"signature\":\"Ag\",\"signature\":\"Ag\"",
        );
        assert!(serde_json::from_str::<BallotRequestDto>(&duplicate).is_err());
        let unknown = original.replace(
            "\"signature\":\"Ag\"",
            "\"signature\":\"Ag\",\"kind\":\"ballot\"",
        );
        assert!(serde_json::from_str::<BallotRequestDto>(&unknown).is_err());
    }

    #[test]
    fn rejects_noncanonical_encodings() {
        let original = example_json();
        let padded = original.replace("\"publicKey\":\"AQ\"", "\"publicKey\":\"AQ==\"");
        assert!(serde_json::from_str::<BallotRequestDto>(&padded).is_err());
        let leading_zero = original.replace("\"a\":\"2\"", "\"a\":\"02\"");
        assert!(serde_json::from_str::<BallotRequestDto>(&leading_zero).is_err());
        let uppercase_uuid = original.replace(
            "00000000-0000-0000-0000-000000000004",
            "00000000-0000-0000-0000-00000000000A",
        );
        assert!(serde_json::from_str::<BallotRequestDto>(&uppercase_uuid).is_err());
    }
}

/// Período proposto pela conta designada para representar a comissão.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PeriodRequest {
    pub(super) starts_at: chrono::DateTime<chrono::Utc>,
    pub(super) ends_at: chrono::DateTime<chrono::Utc>,
    pub(super) reason: String,
}

/// Justificativa da comissão para cancelar a eleição.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CancellationRequest {
    pub(super) reason: String,
}
