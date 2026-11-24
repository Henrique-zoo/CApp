//! Persistência das designações explícitas e publicação das provas do manifesto.

use chrono::{DateTime, Utc};
use rsa::BigUint;
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::modules::elections::{
    application::{
        board::{BoardError, FrozenBoard},
        ports::{
            BoardEvidenceSigner, PreparationRepository, PreparationSigner, PreparationTransaction,
        },
        preparation::{
            AssignedAuthority, AuthorityAssignment, AuthorityRegistrationProof, Preparation,
            PreparedPublication,
        },
    },
    crypto_wire::{
        CryptoWireError, ServerSigner, group_parameters, string, tally::authority_key,
        validate_public_key, verify_authority_knowledge, verify_signed_object,
    },
    domain::{
        cryptography::elgamal::generate_authority_key_pair,
        entities::{authority::AuthorityRole, election::ElectionPeriod},
        identifiers::{AcademicCenterId, AccountId, AuthorityId, ElectionId, SlateId},
    },
};

use super::{
    registration::database_now,
    repository::{PgElectionBoardRepository, insert_publication, mark_published},
};

impl PreparationSigner for ServerSigner {
    fn verify_credential(&self, proof: &AuthorityRegistrationProof) -> Result<Value, BoardError> {
        let envelope = super::cryptography::authority_codec::registration_envelope(proof);
        let credential = &envelope["payload"]["authority"];
        let key = string(credential, "signingKey")?;
        verify_signed_object(&envelope, key)?;
        verify_authority_knowledge(proof.election.into_uuid(), credential)?;
        Ok(credential.clone())
    }

    fn prepare_publication(&self, setup: &Preparation) -> Result<PreparedPublication, BoardError> {
        let server_key = self.public_key_base64()?;
        let mut authorities = Vec::with_capacity(setup.authorities.len() + 1);
        let mut options = Vec::new();
        let (prime, _) = group_parameters();
        let mut external_product = BigUint::from(1u8);
        for row in &setup.authorities {
            let credential = row
                .credential
                .as_ref()
                .ok_or(BoardError::PreparationIncomplete)?;
            let key = string(credential, "signingKey")?;
            if key == server_key {
                return Err(BoardError::InvalidEvidence(Box::new(
                    CryptoWireError::PublicKey,
                )));
            }
            validate_public_key(key)?;
            external_product = external_product
                * verify_authority_knowledge(setup.election.into_uuid(), credential)?
                % prime;
            if let AuthorityRole::Slate(slate) = row.role {
                let name = row
                    .slate_name
                    .as_deref()
                    .ok_or(BoardError::PreparationIncomplete)?;
                options.push(json!({"id": slate.into_uuid(), "kind": "slate", "name": name}));
            }
            authorities.push(credential.clone());
        }
        if options.is_empty() {
            return Err(BoardError::PreparationIncomplete);
        }
        options.push(json!({"id": Uuid::nil(), "kind": "null", "name": "Nulo"}));

        // Se a chave conjunta for a identidade, só o servidor refaz o par.
        let (server, combined) = loop {
            let server = generate_authority_key_pair().map_err(CryptoWireError::from)?;
            let public = BigUint::from_bytes_be(&server.public_value().to_be_bytes());
            let combined = &external_product * public % prime;
            if combined != BigUint::from(1u8) {
                break (server, combined);
            }
        };
        let server_authority =
            AuthorityId::try_from_uuid(Uuid::new_v4()).expect("a generated v4 UUID is non-nil");
        authorities.insert(
            0,
            authority_key(
                setup.election,
                server_authority,
                AuthorityRole::Server,
                &server_key,
                &server,
            )?,
        );
        let manifest = self.sign_payload(&json!({
            "version": 1, "kind": "manifest",
            "electionId": setup.election.into_uuid(), "academicCenterId": setup.center.into_uuid(),
            "groupId": "rfc3526-modp14", "options": options, "authorities": authorities,
            "combinedPublicKey": combined.to_str_radix(10), "serverSigningKey": server_key,
        }))?;
        let encoded = serde_json::to_vec(&manifest).map_err(|_| CryptoWireError::Encoding)?;
        let board = self.verify_origin(FrozenBoard {
            election_id: setup.election,
            manifest: encoded,
            period: setup.period.clone(),
            requires_voter_registration: setup.requires_registration,
        })?;

        Ok(PreparedPublication {
            board,
            server_authority,
            server_secret: *server.secret(),
        })
    }
}

#[derive(sqlx::FromRow)]
struct AssignmentRow {
    authority_id: Uuid,
    role: String,
    slate_id: Option<Uuid>,
    registered: bool,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    cancelled: bool,
    manifest_hash: Option<String>,
}

#[derive(sqlx::FromRow)]
struct DraftRow {
    academic_center_id: Uuid,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    status: String,
    requires_voter_registration: bool,
}

#[derive(sqlx::FromRow)]
struct AuthorityRow {
    authority_id: Uuid,
    account_id: Uuid,
    role: String,
    slate_id: Option<Uuid>,
    slate_name: Option<String>,
    credential: Option<Value>,
}

/// Bloqueio da preparação, compartilhado pela publicação e pelas autoridades.
pub struct PgPreparationTransaction {
    transaction: Transaction<'static, Postgres>,
    election: ElectionId,
}

#[async_trait::async_trait]
impl PreparationRepository for PgElectionBoardRepository {
    type Transaction = PgPreparationTransaction;

    async fn assignment(
        &self,
        election: ElectionId,
        account: AccountId,
    ) -> Result<AuthorityAssignment, BoardError> {
        let row: AssignmentRow = sqlx::query_as(
            "SELECT a.authority_id, a.role, a.slate_id, b.manifest_hash,
                a.credential IS NOT NULL OR b.manifest_hash IS NOT NULL AS registered,
                p.starts_at, p.ends_at, p.status = 'cancelled' AS cancelled
             FROM election_authority a
             JOIN election p ON p.election_id = a.election_id
             LEFT JOIN election_board b ON b.election_id = a.election_id
             WHERE a.election_id = $1 AND a.account_id = $2",
        )
        .bind(election.into_uuid())
        .bind(account.into_uuid())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(BoardError::AuthorityNotAssigned)?;
        Ok(AuthorityAssignment {
            authority_id: row.authority_id,
            role: row.role,
            slate_id: row.slate_id,
            registered: row.registered,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            cancelled: row.cancelled,
            manifest_hash: row.manifest_hash,
        })
    }

    async fn lock_preparation(
        &self,
        election: ElectionId,
        account: AccountId,
    ) -> Result<(Preparation, Self::Transaction), BoardError> {
        self.assignment(election, account).await?;
        let mut transaction = self.pool.begin().await?;
        let row: DraftRow = sqlx::query_as(
            "SELECT academic_center_id, starts_at, ends_at, status, requires_voter_registration
             FROM election WHERE election_id = $1 FOR UPDATE",
        )
        .bind(election.into_uuid())
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(BoardError::NotFound)?;
        match row.status.as_str() {
            "preparing" => {}
            "cancelled" => return Err(BoardError::Cancelled),
            _ => return Err(BoardError::AlreadyExists),
        }
        let now = database_now(&mut transaction).await?;
        let authorities: Vec<AuthorityRow> = sqlx::query_as(
            "SELECT authority_id, account_id, role, slate_id, slate_name, credential
             FROM election_authority WHERE election_id = $1 AND role <> 'server'
             ORDER BY position NULLS FIRST FOR SHARE",
        )
        .bind(election.into_uuid())
        .fetch_all(&mut *transaction)
        .await?;
        let authorities = authorities
            .into_iter()
            .map(|row| {
                let role = match (row.role.as_str(), row.slate_id) {
                    ("commission", None) => AuthorityRole::Commission,
                    ("slate", Some(slate)) => AuthorityRole::Slate(
                        SlateId::try_from_uuid(slate)
                            .map_err(|_| BoardError::PreparationIncomplete)?,
                    ),
                    _ => return Err(BoardError::PreparationIncomplete),
                };
                Ok(AssignedAuthority {
                    id: AuthorityId::try_from_uuid(row.authority_id)
                        .map_err(|_| BoardError::PreparationIncomplete)?,
                    account: AccountId::try_from_uuid(row.account_id)
                        .map_err(|_| BoardError::PreparationIncomplete)?,
                    role,
                    slate_name: row.slate_name,
                    credential: row.credential,
                })
            })
            .collect::<Result<_, BoardError>>()?;
        let setup = Preparation {
            election,
            now,
            center: AcademicCenterId::try_from_uuid(row.academic_center_id)
                .map_err(|_| BoardError::PreparationIncomplete)?,
            period: ElectionPeriod::try_new(row.starts_at, row.ends_at)
                .map_err(|_| BoardError::PreparationIncomplete)?,
            requires_registration: row.requires_voter_registration,
            authorities,
        };
        Ok((
            setup,
            PgPreparationTransaction {
                transaction,
                election,
            },
        ))
    }
}

#[async_trait::async_trait]
impl PreparationTransaction for PgPreparationTransaction {
    async fn save_credential(
        mut self,
        authority: AuthorityId,
        credential: &Value,
    ) -> Result<(), BoardError> {
        sqlx::query("UPDATE election_authority SET credential = $3 WHERE election_id = $1 AND authority_id = $2")
            .bind(self.election.into_uuid()).bind(authority.into_uuid()).bind(credential)
            .execute(&mut *self.transaction).await?;
        self.transaction.commit().await?;
        Ok(())
    }

    async fn publish(mut self, publication: &PreparedPublication) -> Result<(), BoardError> {
        if !insert_publication(&mut self.transaction, &publication.board).await? {
            return Err(BoardError::AlreadyExists);
        }
        let id = self.election.into_uuid();
        sqlx::query(
            "INSERT INTO election_authority (election_id, authority_id, role, server_secret)
             VALUES ($1, $2, 'server', $3)",
        )
        .bind(id)
        .bind(publication.server_authority.into_uuid())
        .bind(publication.server_secret.to_be_bytes().as_slice())
        .execute(&mut *self.transaction)
        .await?;
        sqlx::query("UPDATE election_authority SET credential = NULL WHERE election_id = $1")
            .bind(id)
            .execute(&mut *self.transaction)
            .await?;
        mark_published(&mut self.transaction, id).await?;

        self.transaction.commit().await?;
        Ok(())
    }
}
