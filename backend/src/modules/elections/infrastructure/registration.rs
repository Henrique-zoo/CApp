//! Persistência do cadastro prévio e leitura transacional da elegibilidade.
//!
//! O cadastro e o voto usam o mesmo bloqueio do quadro. A evidência vigente é
//! lida junto da associação de cursos do CA e permanece estável até o commit.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::codec::decode_base64_url;
use crate::modules::elections::{
    application::{
        error::RegistrationError,
        ports::{VoterKeyProofVerifier, VoterRegistry},
        registration::{
            RegistrationAction, RegistrationStatus, VoterIdentity, registration_action,
        },
    },
    crypto_wire::verify_registration,
};

/// Valida o envelope assinado com as regras do protocolo v1.
pub struct WireVoterKeyProofVerifier;

impl VoterKeyProofVerifier for WireVoterKeyProofVerifier {
    fn verify(
        &self,
        proof: &[u8],
        election_id: Uuid,
        account_id: Uuid,
    ) -> Result<Vec<u8>, RegistrationError> {
        let key = verify_registration(proof, election_id, account_id)
            .map_err(|_| RegistrationError::InvalidProof)?;
        decode_base64_url(&key).map_err(|_| RegistrationError::InvalidProof)
    }
}

/// Adaptador PostgreSQL para o cadastro privado da chave eleitoral.
///
/// Clones compartilham o mesmo pool de conexões.
#[derive(Clone)]
pub struct PgVoterRegistry {
    pool: PgPool,
}

impl From<sqlx::Error> for RegistrationError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(Box::new(error))
    }
}

impl PgVoterRegistry {
    /// Usa o pool compartilhado com o quadro e o módulo de identidade.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl VoterRegistry for PgVoterRegistry {
    async fn status(
        &self,
        election_id: Uuid,
        account_id: Uuid,
    ) -> Result<RegistrationStatus, RegistrationError> {
        let mut conn = self.pool.acquire().await?;
        registration_status(&mut conn, election_id, account_id).await
    }

    async fn register_verified_key(
        &self,
        election_id: Uuid,
        identity: VoterIdentity,
        public_key: Vec<u8>,
    ) -> Result<RegistrationStatus, RegistrationError> {
        let mut tx = self.pool.begin().await?;
        let (starts_at, closed): (DateTime<Utc>, bool) = sqlx::query_as(
            "SELECT starts_at, status = 'closed' FROM election_board
             WHERE election_id = $1 FOR UPDATE",
        )
        .bind(election_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(RegistrationError::NotFound)?;

        let status = registration_status(&mut tx, election_id, identity.account_id).await?;
        let eligible_until = if status.requires_registration && status.public_key.is_none() {
            eligibility_deadline(&mut tx, election_id, identity).await?
        } else {
            None
        };
        let now = database_now(&mut tx).await?;
        let result = match registration_action(
            &status,
            &public_key,
            starts_at,
            closed,
            now,
            eligible_until,
        )? {
            RegistrationAction::Reuse => status,
            RegistrationAction::Insert => {
                sqlx::query(
                    "INSERT INTO election_voter_registration
                         (election_id, account_id, public_key, registered_at)
                     VALUES ($1, $2, $3, $4)",
                )
                .bind(election_id)
                .bind(identity.account_id)
                .bind(&public_key)
                .bind(now)
                .execute(&mut *tx)
                .await?;

                RegistrationStatus {
                    requires_registration: true,
                    voter_id: Some(identity.account_id),
                    public_key: Some(public_key),
                    participated: false,
                }
            }
        };

        tx.commit().await?;
        Ok(result)
    }
}

#[derive(sqlx::FromRow)]
struct RegistrationRow {
    requires_registration: bool,
    voter_id: Option<Uuid>,
    registered_key: Option<Vec<u8>>,
    accepted_key: Option<String>,
}

async fn registration_status(
    conn: &mut PgConnection,
    election_id: Uuid,
    account_id: Uuid,
) -> Result<RegistrationStatus, RegistrationError> {
    let row: RegistrationRow = sqlx::query_as(
        "SELECT b.requires_voter_registration AS requires_registration,
                p.voter_id, r.public_key AS registered_key,
                e.signed_ballot->'payload'->>'publicKey' AS accepted_key
         FROM election_board b
         LEFT JOIN election_voter_registration r ON r.election_id = b.election_id AND r.account_id = $2
         LEFT JOIN election_participation p ON p.election_id = b.election_id AND p.account_id = $2
         LEFT JOIN election_board_entry e ON e.election_id = p.election_id AND e.voter_id = p.voter_id
         WHERE b.election_id = $1",
    )
    .bind(election_id)
    .bind(account_id)
    .fetch_optional(conn)
    .await?
    .ok_or(RegistrationError::NotFound)?;

    let public_key = match row.accepted_key {
        Some(key) => Some(
            decode_base64_url(&key).map_err(|error| RegistrationError::Storage(Box::new(error)))?,
        ),
        None => row.registered_key,
    };
    let participated = row.voter_id.is_some();

    Ok(RegistrationStatus {
        requires_registration: row.requires_registration,
        voter_id: row
            .voter_id
            .or_else(|| public_key.as_ref().map(|_| account_id)),
        public_key,
        participated,
    })
}

/// Obtém o prazo da comprovação do aluno em um curso representado pelo CA
/// do manifesto. As linhas usadas pelo cálculo ficam estáveis até o commit.
pub(super) async fn eligibility_deadline(
    tx: &mut Transaction<'_, Postgres>,
    election_id: Uuid,
    identity: VoterIdentity,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT v.expires_at FROM election_board b
         JOIN academic_center c ON c.id = (b.manifest->'payload'->>'academicCenterId')::uuid
         JOIN institution i ON i.id = c.institution_id
         JOIN institutional_verification v ON v.institution_id = i.id AND v.user_id = $2
         JOIN academic_center_course cc ON cc.academic_center_id = c.id AND cc.course_name = btrim(v.course_name)
         JOIN app_user u ON u.id = v.user_id
         WHERE b.election_id = $1 AND c.status = 'active' AND i.status = 'active'
           AND u.status <> 'inactive' AND v.student_membership_confirmed
           AND v.microsoft_object_id = $3 AND v.microsoft_tenant_id = i.microsoft_tenant_id
           AND v.microsoft_student_group_id = i.microsoft_student_group_id
           AND v.checked_at <= clock_timestamp()
         FOR SHARE OF c, i, v, cc, u",
    )
    .bind(election_id)
    .bind(identity.account_id)
    .bind(identity.microsoft_object_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Usa o instante real do banco após adquirir os bloqueios da operação.
pub(super) async fn database_now(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<DateTime<Utc>, sqlx::Error> {
    sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut **tx)
        .await
}
