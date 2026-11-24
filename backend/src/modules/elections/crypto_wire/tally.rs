//! Verifica o quadro encerrado e as contribuições antes de recuperar os totais.
//!
//! A apuração parte das cédulas aceitas no quadro. Confere o encadeamento e o
//! recibo final, recompõe o produto e só usa contribuições assinadas ligadas
//! ao mesmo manifesto, termo de encerramento e agregado.

use std::collections::{HashMap, HashSet};

use crate::modules::elections::{
    application::board::BoardSnapshot,
    domain::{
        cryptography::elgamal::{
            ElGamalAuthorityKeyPair, ElGamalCiphertext, accumulate_ballot, empty_accumulator,
            recover_totals,
        },
        entities::authority::AuthorityRole,
        error::CryptographyError,
        identifiers::{AuthorityId, ElectionId},
        profile::MAX_VOTER_COUNT,
    },
};

use super::*;

/// Produz a transcrição pública da chave de uma autoridade em preparação.
/// O valor de assinatura é a chave pública RSA da mesma autoridade em DER/base64url.
pub fn authority_key(
    election: ElectionId,
    authority: AuthorityId,
    role: AuthorityRole,
    signing_key: &str,
    key_pair: &ElGamalAuthorityKeyPair,
) -> Result<Value, CryptoWireError> {
    validate_public_key(signing_key)?;
    let (role, slate_id) = match role {
        AuthorityRole::Server => ("server", None),
        AuthorityRole::Commission => ("commission", None),
        AuthorityRole::Slate(id) => ("slate", Some(id.into_uuid())),
    };
    let public = big(*key_pair.public_value()).to_str_radix(10);
    let proof = key_pair.prove_key(|commitment| {
        let transcript = json!({
            "version": 1, "kind": "authority-key-proof",
            "electionId": election.into_uuid(), "groupId": "rfc3526-modp14",
            "authorityId": authority.into_uuid(), "role": role, "slateId": slate_id,
            "signingKey": signing_key, "publicValue": public,
            "commitment": big(*commitment).to_str_radix(10),
        });
        Ok::<_, CryptoWireError>(Sha256::digest(signature_input(&transcript)?).into())
    })?;

    Ok(json!({
        "id": authority.into_uuid(), "role": role, "slateId": slate_id,
        "signingKey": signing_key, "publicValue": public,
        "keyProof": {
            "commitment": big(*proof.commitment()).to_str_radix(10),
            "response": big(*proof.response()).to_str_radix(10),
        },
    }))
}

impl From<CryptographyError> for CryptoWireError {
    fn from(error: CryptographyError) -> Self {
        match error {
            CryptographyError::RandomnessUnavailable(_) => Self::Signing,
            _ => Self::Ciphertext,
        }
    }
}

/// Origem e agregado que passaram pela verificação completa do quadro.
pub struct VerifiedTally {
    manifest: VerifiedManifest,
    closure_hash: String,
    aggregate_hash: String,
    voter_count: u32,
    accumulator: Vec<ElGamalCiphertext>,
}

/// Contribuição autenticada de uma autoridade para um único encerramento.
pub struct VerifiedShare {
    authority_id: Uuid,
    aggregate_hash: String,
    values: Vec<U2048>,
}

impl VerifiedShare {
    /// Autoridade declarada e autenticada pela chave publicada no manifesto.
    pub const fn authority_id(&self) -> Uuid {
        self.authority_id
    }
}

impl VerifiedTally {
    /// Confere manifesto, cédulas, assinaturas, unicidade, checkpoint e agregado.
    ///
    /// A chave do servidor vem de configuração confiável. A comparação do
    /// produto impede usar um subconjunto de cédulas ou trocar uma posição.
    /// Rejeita um quadro ainda aberto ou qualquer divergência nas evidências.
    pub fn from_board(board: &BoardSnapshot, trusted_key: &str) -> Result<Self, CryptoWireError> {
        if board.cancellation.is_some() {
            return Err(CryptoWireError::Context);
        }
        let manifest = verify_manifest(&wire_bytes(&board.manifest)?, trusted_key)?;
        let closure = board.closure.as_ref().ok_or(CryptoWireError::Context)?;
        let aggregate = board.aggregate.as_ref().ok_or(CryptoWireError::Context)?;
        let voter_count =
            u32::try_from(board.entries.len()).map_err(|_| CryptoWireError::Context)?;
        if voter_count > MAX_VOTER_COUNT {
            return Err(CryptoWireError::Context);
        }

        let mut accumulator = empty_accumulator(manifest.option_ids.len())
            .map_err(|_| CryptoWireError::Ciphertext)?;
        let mut head = manifest.manifest_hash.clone();
        let mut voters = HashSet::with_capacity(board.entries.len());
        let mut last_ballot = None;

        for (position, entry) in board.entries.iter().enumerate() {
            let payload = &entry["payload"]["ballot"]["payload"];
            let voter = uuid_value(string(payload, "accountId")?, false)?;
            let account_id =
                AccountId::try_from_uuid(voter).map_err(|_| CryptoWireError::Context)?;
            if !voters.insert(account_id) {
                return Err(CryptoWireError::Context);
            }
            let context = BallotContext {
                election_id: manifest.election_id,
                manifest_hash: manifest.manifest_hash.clone(),
                account_id,
                expected_public_key: None,
                option_ids: manifest.option_ids.clone(),
            };
            let (ballot, next_head) =
                verify_entry(entry, &context, position as u32 + 1, &head, trusted_key)?;
            accumulator = accumulate_ballot(accumulator, &ballot.ciphertexts)
                .map_err(|_| CryptoWireError::Ciphertext)?;
            head = next_head;
            last_ballot = Some((context, ballot.ballot_hash));
        }

        match (last_ballot, &board.checkpoint) {
            (None, None) => {}
            (Some((context, ballot_hash)), Some(checkpoint)) => {
                verify_receipt(
                    checkpoint,
                    &context,
                    voter_count,
                    &ballot_hash,
                    &head,
                    trusted_key,
                )?;
            }
            _ => return Err(CryptoWireError::Context),
        }

        verify_closure(
            &wire_bytes(closure)?,
            &manifest,
            &head,
            voter_count,
            trusted_key,
        )?;
        let closure_hash = object_digest(&closure["payload"])?;
        let aggregate_hash = verify_aggregate(
            aggregate,
            &manifest,
            &closure_hash,
            &accumulator,
            trusted_key,
        )?;

        Ok(Self {
            manifest,
            closure_hash,
            aggregate_hash,
            voter_count,
            accumulator,
        })
    }

    /// Autentica a contribuição e confere uma prova por opção do agregado.
    ///
    /// A prova demonstra que a mesma chave secreta produziu o valor público
    /// da autoridade e a contribuição, sem revelar esse segredo.
    pub fn verify_share(&self, envelope: &Value) -> Result<VerifiedShare, CryptoWireError> {
        let payload = &envelope["payload"];
        exact_fields(
            payload,
            &[
                "version",
                "kind",
                "electionId",
                "manifestHash",
                "closureHash",
                "aggregateHash",
                "authorityId",
                "shares",
            ],
        )?;
        version_kind(payload, "decryption-share")?;
        if uuid_value(string(payload, "electionId")?, false)? != self.manifest.election_id
            || string(payload, "manifestHash")? != self.manifest.manifest_hash
            || string(payload, "closureHash")? != self.closure_hash
            || string(payload, "aggregateHash")? != self.aggregate_hash
        {
            return Err(CryptoWireError::Context);
        }
        let authority_id = uuid_value(string(payload, "authorityId")?, false)?;
        let authority = self
            .authorities()
            .iter()
            .find(|authority| authority["id"] == payload["authorityId"])
            .ok_or(CryptoWireError::Context)?;
        verify_signed_object(envelope, string(authority, "signingKey")?)?;

        let shares = payload["shares"]
            .as_array()
            .ok_or(CryptoWireError::Encoding)?;
        if shares.len() != self.accumulator.len() {
            return Err(CryptoWireError::Context);
        }

        let public = group_element(string(authority, "publicValue")?)?;
        let values = self
            .manifest
            .option_ids
            .iter()
            .zip(&self.accumulator)
            .zip(shares)
            .map(|((option_id, cipher), share)| {
                self.verify_option_share(authority, &public, *option_id, cipher, share)
            })
            .collect::<Result<_, _>>()?;

        Ok(VerifiedShare {
            authority_id,
            aggregate_hash: self.aggregate_hash.clone(),
            values,
        })
    }

    /// Assina uma contribuição após verificar o quadro completo e a chave local.
    /// A mesma autoridade só contribui para o agregado que originou este contexto.
    pub fn sign_share(
        &self,
        authority_id: AuthorityId,
        key_pair: &ElGamalAuthorityKeyPair,
        signer: &ServerSigner,
    ) -> Result<Value, CryptoWireError> {
        let authority = self
            .authorities()
            .iter()
            .find(|value| value["id"] == json!(authority_id.into_uuid()))
            .ok_or(CryptoWireError::Context)?;
        if string(authority, "signingKey")? != signer.public_key_base64()?
            || decimal(string(authority, "publicValue")?)? != *key_pair.public_value()
        {
            return Err(CryptoWireError::Context);
        }

        let shares = self
            .manifest
            .option_ids
            .iter()
            .zip(&self.accumulator)
            .map(|(option_id, cipher)| {
                let mut transcript = self.transcript(authority, *option_id, cipher);
                let share =
                    key_pair.contribute(&cipher.a, |value, commitment_g, commitment_a| {
                        transcript["value"] = json!(big(*value).to_str_radix(10));
                        transcript["commitmentG"] = json!(big(*commitment_g).to_str_radix(10));
                        transcript["commitmentA"] = json!(big(*commitment_a).to_str_radix(10));
                        Ok::<_, CryptoWireError>(
                            Sha256::digest(signature_input(&transcript)?).into(),
                        )
                    })?;
                Ok(json!({
                    "optionId": option_id, "value": big(share.value).to_str_radix(10),
                    "proof": {
                        "commitmentG": big(share.commitment_g).to_str_radix(10),
                        "commitmentA": big(share.commitment_a).to_str_radix(10),
                        "response": big(share.response).to_str_radix(10),
                    },
                }))
            })
            .collect::<Result<Vec<_>, CryptoWireError>>()?;
        signer.sign_payload(&json!({
            "version": 1, "kind": "decryption-share",
            "electionId": self.manifest.election_id, "manifestHash": self.manifest.manifest_hash,
            "closureHash": self.closure_hash, "aggregateHash": self.aggregate_hash,
            "authorityId": authority_id.into_uuid(), "shares": shares,
        }))
    }

    /// Confere que o identificador no manifesto pertence ao servidor.
    pub fn is_server_authority(&self, authority: AuthorityId) -> bool {
        self.authorities()
            .iter()
            .any(|row| row["id"] == json!(authority.into_uuid()) && row["role"] == "server")
    }

    /// Calcula o payload de resultado pronto para assinatura ou auditoria.
    ///
    /// Contribuições ausentes deixam a apuração incompleta e sem contagens.
    /// Com todas as contribuições, recupera uma contagem por opção. Divergências
    /// de faixa ou de soma produzem um desfecho anulado explícito.
    pub fn result_payload(&self, shares: &[VerifiedShare]) -> Result<Value, CryptoWireError> {
        let mut received = HashMap::with_capacity(shares.len());
        for share in shares {
            if share.aggregate_hash != self.aggregate_hash
                || received
                    .insert(share.authority_id, share.values.as_slice())
                    .is_some()
            {
                return Err(CryptoWireError::Context);
            }
        }
        let mut missing = Vec::new();
        let mut ordered = Vec::new();
        for authority in self.authorities() {
            let id = uuid_value(string(authority, "id")?, false)?;
            match received.remove(&id) {
                Some(values) => ordered.push(values),
                None => missing.push(id),
            }
        }
        if !received.is_empty() {
            return Err(CryptoWireError::Context);
        }
        let (status, reason, counts) = if !missing.is_empty() {
            ("incomplete", Some("missing_authority"), None)
        } else {
            match recover_totals(&self.accumulator, self.voter_count, &ordered) {
                Ok(totals) => {
                    let consistent = totals.iter().sum::<u32>() == self.voter_count;
                    let counts: Vec<_> = self
                        .manifest
                        .option_ids
                        .iter()
                        .zip(totals)
                        .map(|(id, total)| json!({"optionId": id, "count": total}))
                        .collect();
                    if consistent {
                        ("valid", None, Some(counts))
                    } else {
                        ("annulled", Some("aggregate_sum_mismatch"), Some(counts))
                    }
                }
                Err(CryptographyError::TotalOutOfRange) => {
                    ("annulled", Some("tally_out_of_range"), None)
                }
                Err(_) => return Err(CryptoWireError::Ciphertext),
            }
        };

        Ok(json!({
            "version": 1,
            "kind": "result",
            "electionId": self.manifest.election_id,
            "manifestHash": self.manifest.manifest_hash,
            "closureHash": self.closure_hash,
            "aggregateHash": self.aggregate_hash,
            "cancellationHash": null,
            "voterCount": self.voter_count,
            "status": status,
            "reason": reason,
            "counts": counts,
            "missingAuthorities": missing,
        }))
    }

    fn authorities(&self) -> &[Value] {
        // O array já foi exigido por verify_manifest.
        self.manifest.envelope["payload"]["authorities"]
            .as_array()
            .unwrap()
    }

    fn verify_option_share(
        &self,
        authority: &Value,
        public: &BigUint,
        option_id: Uuid,
        cipher: &ElGamalCiphertext,
        share: &Value,
    ) -> Result<U2048, CryptoWireError> {
        exact_fields(share, &["optionId", "value", "proof"])?;
        if uuid_value(string(share, "optionId")?, true)? != option_id {
            return Err(CryptoWireError::Context);
        }
        let proof = &share["proof"];
        exact_fields(proof, &["commitmentG", "commitmentA", "response"])?;
        let value = decimal(string(share, "value")?)?;
        let commitment_g = group_element(string(proof, "commitmentG")?)?;
        let commitment_a = group_or_identity(decimal(string(proof, "commitmentA")?)?)?;
        let response = big(decimal(string(proof, "response")?)?);
        let (prime, order) = group_parameters();
        if &response >= order || (value == U2048::ONE) != (cipher.a == U2048::ONE) {
            return Err(CryptoWireError::Ciphertext);
        }

        let base = big(cipher.a);
        let mut transcript = self.transcript(authority, option_id, cipher);
        transcript["value"] = share["value"].clone();
        transcript["commitmentG"] = proof["commitmentG"].clone();
        transcript["commitmentA"] = proof["commitmentA"].clone();
        let challenge =
            BigUint::from_bytes_be(&Sha256::digest(signature_input(&transcript)?)) % order;
        let shared = group_or_identity(value)?;
        if BigUint::from(2_u8).modpow(&response, prime)
            != commitment_g * public.modpow(&challenge, prime) % prime
            || base.modpow(&response, prime)
                != commitment_a * shared.modpow(&challenge, prime) % prime
        {
            return Err(CryptoWireError::Signature);
        }
        Ok(value)
    }

    fn transcript(&self, authority: &Value, option_id: Uuid, cipher: &ElGamalCiphertext) -> Value {
        json!({
            "version": 1, "kind": "decryption-proof",
            "electionId": self.manifest.election_id, "groupId": "rfc3526-modp14",
            "manifestHash": self.manifest.manifest_hash, "closureHash": self.closure_hash,
            "aggregateHash": self.aggregate_hash, "authorityId": authority["id"],
            "optionId": option_id, "publicValue": authority["publicValue"],
            "a": big(cipher.a).to_str_radix(10), "b": big(cipher.b).to_str_radix(10),
        })
    }
}

fn verify_aggregate(
    aggregate: &Value,
    manifest: &VerifiedManifest,
    closure_hash: &str,
    ciphertexts: &[ElGamalCiphertext],
    trusted_key: &str,
) -> Result<String, CryptoWireError> {
    verify_signed_object(aggregate, trusted_key)?;
    let payload = &aggregate["payload"];
    exact_fields(
        payload,
        &[
            "version",
            "kind",
            "electionId",
            "manifestHash",
            "closureHash",
            "ciphertexts",
        ],
    )?;
    version_kind(payload, "aggregate")?;
    if uuid_value(string(payload, "electionId")?, false)? != manifest.election_id
        || string(payload, "manifestHash")? != manifest.manifest_hash
        || string(payload, "closureHash")? != closure_hash
    {
        return Err(CryptoWireError::Context);
    }
    let rows = payload["ciphertexts"]
        .as_array()
        .ok_or(CryptoWireError::Encoding)?;
    if rows.len() != ciphertexts.len() {
        return Err(CryptoWireError::Context);
    }
    for ((row, expected), option_id) in rows.iter().zip(ciphertexts).zip(&manifest.option_ids) {
        exact_fields(row, &["optionId", "a", "b"])?;
        if uuid_value(string(row, "optionId")?, true)? != *option_id
            || decimal(string(row, "a")?)? != expected.a
            || decimal(string(row, "b")?)? != expected.b
        {
            return Err(CryptoWireError::Context);
        }
    }
    object_digest(payload)
}

fn big(value: U2048) -> BigUint {
    BigUint::from_bytes_be(value.to_be_bytes().as_slice())
}

fn group_or_identity(value: U2048) -> Result<BigUint, CryptoWireError> {
    if value != U2048::ONE {
        validate_elgamal_key("rfc3526-modp14", &value).map_err(|_| CryptoWireError::Ciphertext)?;
    }
    Ok(big(value))
}

fn wire_bytes(value: &Value) -> Result<Vec<u8>, CryptoWireError> {
    serde_json::to_vec(value).map_err(|_| CryptoWireError::Encoding)
}
