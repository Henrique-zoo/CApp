//! Registro privado e verificação institucional compartilhados pelo cadastro e pelo voto.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::super::{
    application::{
        board::BoardError,
        registration::{RegistrationError, RegistrationStatus, VoterIdentity},
    },
    crypto_wire::verify_registration,
};

/// Persistência do cadastro, sem precisar da chave privada de assinatura do servidor.
pub struct PgVoterRegistry;

impl From<sqlx::Error> for RegistrationError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(Box::new(error))
    }
}

impl PgVoterRegistry {
    /// Configura o curso no fluxo confiável de preparação. O CA vem exclusivamente
    /// do manifesto oficial. Uma configuração já publicada só admite repetição idêntica.
    pub async fn configure_course(
        pool: &PgPool,
        election_id: Uuid,
        course: &str,
    ) -> Result<(), BoardError> {
        let course = course.trim();
        if course.is_empty() {
            return Err(BoardError::InvalidArtifact("curso eleitoral vazio"));
        }
        let mut tx = pool.begin().await?;
        let (center, starts_at, state): (Uuid, DateTime<Utc>, String) = sqlx::query_as(
            "SELECT (manifest->'payload'->>'academicCenterId')::uuid, starts_at, status
             FROM election_board WHERE election_id=$1 FOR UPDATE",
        )
        .bind(election_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(BoardError::NotFound)?;
        let previous: Option<String> = sqlx::query_scalar(
            "SELECT course_name FROM election_eligibility_course WHERE election_id=$1",
        )
        .bind(election_id)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(previous) = previous {
            return if previous == course {
                Ok(())
            } else {
                Err(BoardError::AlreadyExists)
            };
        }
        if state != "open" || database_now(&mut tx).await? >= starts_at {
            return Err(BoardError::VotingEnded);
        }
        let affected = sqlx::query(
            "INSERT INTO election_eligibility_course (election_id,academic_center_id,course_name)
             SELECT $1,c.id,$3 FROM academic_center c JOIN institution i ON i.id=c.institution_id
             WHERE c.id=$2 AND c.status='active' AND i.status='active'",
        )
        .bind(election_id)
        .bind(center)
        .bind(course)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if affected != 1 {
            return Err(BoardError::InvalidArtifact(
                "CA ou instituição sem cadastro ativo",
            ));
        }
        tx.commit().await?;
        Ok(())
    }

    /// O relógio e o lock do quadro determinam o instante limite do cadastro.
    /// Uma repetição com a mesma prova ou nova assinatura válida da mesma chave
    /// recupera o resultado de um cadastro anterior, inclusive após a abertura.
    pub async fn register(
        pool: &PgPool,
        election_id: Uuid,
        identity: VoterIdentity,
        raw: &[u8],
    ) -> Result<RegistrationStatus, RegistrationError> {
        let account_id = identity.account_id;
        let key = verify_registration(raw, election_id, account_id)
            .map_err(|_| RegistrationError::InvalidProof)?;
        let mut tx = pool.begin().await?;
        let (starts_at, state): (DateTime<Utc>, String) = sqlx::query_as(
            "SELECT starts_at,status FROM election_board WHERE election_id=$1 FOR UPDATE",
        )
        .bind(election_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(RegistrationError::NotFound)?;
        let status = Self::read_state(&mut tx, election_id, account_id).await?;
        if let Some(previous) = &status.public_key {
            return if previous == &key {
                Ok(status)
            } else {
                Err(RegistrationError::KeyConflict)
            };
        }
        let valid_until = eligibility_deadline(&mut tx, election_id, identity).await?;
        let now = database_now(&mut tx).await?;
        if state != "open" || now >= starts_at {
            return Err(RegistrationError::RegistrationClosed);
        }
        if valid_until.is_none_or(|end| now >= end) {
            return Err(RegistrationError::NotEligible);
        }
        let voter_id = sqlx::query_scalar(
            "INSERT INTO election_voter_registration(election_id,account_id,public_key,registered_at)
             VALUES($1,$2,$3,$4) RETURNING voter_id",
        )
        .bind(election_id)
        .bind(account_id)
        .bind(&key)
        .bind(now)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(RegistrationStatus {
            voter_id: Some(voter_id),
            public_key: Some(key),
            participated: false,
        })
    }

    /// Não exige elegibilidade vigente para consultar uma participação já confirmada.
    pub async fn status(
        pool: &PgPool,
        election_id: Uuid,
        account_id: Uuid,
    ) -> Result<RegistrationStatus, RegistrationError> {
        let mut conn = pool.acquire().await?;
        Self::read_state(&mut conn, election_id, account_id).await
    }

    async fn read_state(
        conn: &mut sqlx::PgConnection,
        election_id: Uuid,
        account_id: Uuid,
    ) -> Result<RegistrationStatus, RegistrationError> {
        // Votos aceitos antes da migration preservam seu identificador e chave; não
        // simulamos um cadastro prévio retroativo e nunca autorizamos novos votos por isso.
        let (voter_id, public_key, participated) = sqlx::query_as(
            "SELECT COALESCE(r.voter_id,p.voter_id) AS voter_id,
                    COALESCE(r.public_key,e.signed_ballot->'payload'->>'publicKey') AS public_key,
                    p.account_id IS NOT NULL AS participated
             FROM election_board b
             LEFT JOIN election_voter_registration r ON r.election_id=b.election_id AND r.account_id=$2
             LEFT JOIN election_participation p ON p.election_id=b.election_id AND p.account_id=$2
             LEFT JOIN election_board_entry e ON e.election_id=p.election_id AND e.voter_id=p.voter_id
             WHERE b.election_id=$1",
        )
        .bind(election_id)
        .bind(account_id)
        .fetch_optional(conn)
        .await?
        .ok_or(RegistrationError::NotFound)?;
        Ok(RegistrationStatus {
            voter_id,
            public_key,
            participated,
        })
    }
}

/// A leitura bloqueia modificações concorrentes da verificação e da configuração
/// institucional até o commit. O chamador compara a validade com o relógio do banco
/// depois de obter esses locks, inclusive quando aguardou uma revalidação concorrente.
pub(super) async fn eligibility_deadline(
    tx: &mut Transaction<'_, Postgres>,
    election_id: Uuid,
    identity: VoterIdentity,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT v.expires_at FROM election_eligibility_course ec
         JOIN academic_center c ON c.id=ec.academic_center_id
         JOIN institution i ON i.id=c.institution_id
         JOIN institutional_verification v ON v.institution_id=i.id AND v.user_id=$2
         JOIN app_user u ON u.id=v.user_id
         WHERE ec.election_id=$1 AND c.status='active' AND i.status='active' AND u.status<>'inactive'
           AND v.microsoft_object_id=$3 AND v.student_membership_confirmed AND btrim(v.course_name)=ec.course_name
           AND v.microsoft_tenant_id=i.microsoft_tenant_id
           AND v.microsoft_student_group_id=i.microsoft_student_group_id
           AND v.checked_at <= clock_timestamp()
         FOR SHARE OF ec,c,i,v,u",
    )
    .bind(election_id)
    .bind(identity.account_id)
    .bind(identity.microsoft_object_id)
    .fetch_optional(&mut **tx)
    .await
}

pub(super) async fn database_now(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<DateTime<Utc>, sqlx::Error> {
    sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut **tx)
        .await
}
