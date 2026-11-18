//! Segredo cifrado associado a um participante do programa.
//!
//! Guarda a chave de campo, a versão e o valor já cifrado, com referência à
//! chave externa. Aplica unicidade `(participant_id, field_key, version)` e só
//! aceita nova versão para participante ativo. Não cifra, não decifra e não
//! calcula hash; o valor cifrado permanece separado de snapshots, sugestões e
//! vínculos.

use crate::modules::mentorship::domain::entities::participant::MentorshipParticipant;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Fotografia persistida de um segredo de matching. Não é um contrato HTTP.
///
/// Campos privados expõem leitura por métodos e alteração pelo descarte
/// lógico. A reconstituição PostgreSQL usa o mapeamento `sqlx::FromRow`
/// adotado no projeto.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MatchingSecret {
    id: Uuid,
    participant_id: Uuid,
    field_key: String,
    version: i32,
    encrypted_value: Vec<u8>,
    encryption_key_ref: String,
    created_at: DateTime<Utc>,
    discarded_at: Option<DateTime<Utc>>,
}

/// Uma criação ou descarte de segredo que não preservaria as regras do domínio.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Segredo de matching inválido")]
pub struct InvalidSecret;

impl MatchingSecret {
    /// Identificador interno estável deste segredo.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Participante ao qual o segredo está vinculado.
    pub fn participant_id(&self) -> Uuid {
        self.participant_id
    }

    /// Código da característica secreta, sem o valor em claro.
    pub fn field_key(&self) -> &str {
        &self.field_key
    }

    /// Versão deste registro no escopo `(participante, field_key)`.
    pub fn version(&self) -> i32 {
        self.version
    }

    /// Bytes já cifrados; a entidade não interpreta o conteúdo.
    pub fn encrypted_value(&self) -> &[u8] {
        &self.encrypted_value
    }

    /// Localizador da chave no serviço de segredos; não contém a chave.
    pub fn encryption_key_ref(&self) -> &str {
        &self.encryption_key_ref
    }

    /// Instante de criação do registro.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Instante do descarte lógico, quando registrado.
    pub fn discarded_at(&self) -> Option<DateTime<Utc>> {
        self.discarded_at
    }

    /// Indica se o segredo já foi descartado logicamente.
    pub fn is_discarded(&self) -> bool {
        self.discarded_at.is_some()
    }

    /// Registra uma nova versão do segredo para participante ativo.
    ///
    /// # Parâmetros e pré-condições
    ///
    /// - `participant` precisa estar ativo.
    /// - `field_key` e `encryption_key_ref` não vazios após `trim`.
    /// - `encrypted_value` chega já cifrado; não é transformado aqui.
    /// - `existing` traz segredos já conhecidos (inclusive descartados) para
    ///   conferir a unicidade `(participant_id, field_key, version)`.
    ///
    /// Não cifra, não decifra e não inclui o valor em snapshot, sugestão ou vínculo.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidSecret`] se o participante estiver inativo, as chaves
    /// textuais forem vazias ou a versão já existir para o mesmo campo.
    pub fn record(
        id: Uuid,
        participant: &MentorshipParticipant,
        field_key: String,
        version: i32,
        encrypted_value: Vec<u8>,
        encryption_key_ref: String,
        created_at: DateTime<Utc>,
        existing: &[Self],
    ) -> Result<Self, InvalidSecret> {
        if !participant.is_active() {
            return Err(InvalidSecret);
        }

        let field_key = normalize_non_empty(&field_key)?;
        let encryption_key_ref = normalize_non_empty(&encryption_key_ref)?;

        if existing.iter().any(|s| {
            s.participant_id == participant.id()
                && s.field_key == field_key
                && s.version == version
        }) {
            return Err(InvalidSecret);
        }

        Ok(Self {
            id,
            participant_id: participant.id(),
            field_key,
            version,
            encrypted_value,
            encryption_key_ref,
            created_at,
            discarded_at: None,
        })
    }

    /// Descarta logicamente o segredo: preenche `discarded_at` e mantém o conteúdo.
    ///
    /// Em `Ok(())`, o valor cifrado permanece no registro. A remoção efetiva dos
    /// bytes é política de retenção futura, não desta transição.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidSecret`] se o segredo já estiver descartado. Em erro, o
    /// objeto fica intacto.
    pub fn discard(&mut self, discarded_at: DateTime<Utc>) -> Result<(), InvalidSecret> {
        if self.is_discarded() {
            return Err(InvalidSecret);
        }

        self.discarded_at = Some(discarded_at);
        Ok(())
    }
}

fn normalize_non_empty(value: &str) -> Result<String, InvalidSecret> {
    let value = value.trim();
    if value.is_empty() {
        return Err(InvalidSecret);
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::mentorship::domain::entities::program::{
        MentorshipProgram, ProgramConfig,
    };
    use crate::modules::mentorship::domain::{
        MatchingStrategy, MentorshipParticipantRole,
    };
    use chrono::TimeZone;

    fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    fn open_program() -> MentorshipProgram {
        let mut program = MentorshipProgram::draft(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ProgramConfig {
                name: "Apadrinhamento 2026.1".into(),
                matching_strategy: MatchingStrategy::Secrets,
                mentors_per_mentee: 2,
                allow_other_courses: false,
                starts_at: at(2026, 3, 1),
                ends_at: Some(at(2026, 7, 1)),
            },
        )
        .unwrap();
        program.open().unwrap();
        program
    }

    fn active_participant(program: &MentorshipProgram) -> MentorshipParticipant {
        MentorshipParticipant::enroll(
            Uuid::new_v4(),
            program,
            Uuid::new_v4(),
            Uuid::new_v4(),
            MentorshipParticipantRole::Calouro,
            false,
            at(2026, 4, 1),
            &[],
        )
        .unwrap()
    }

    #[test]
    fn record_stores_ciphertext_without_transforming_it() {
        let program = open_program();
        let participant = active_participant(&program);
        let ciphertext = b"already-encrypted".to_vec();

        let secret = MatchingSecret::record(
            Uuid::new_v4(),
            &participant,
            "  hometown  ".into(),
            1,
            ciphertext.clone(),
            "  vault/keys/matching-1  ".into(),
            at(2026, 4, 2),
            &[],
        )
        .unwrap();

        assert_eq!(secret.field_key(), "hometown");
        assert_eq!(secret.encryption_key_ref(), "vault/keys/matching-1");
        assert_eq!(secret.version(), 1);
        assert_eq!(secret.encrypted_value(), ciphertext.as_slice());
        assert_eq!(secret.participant_id(), participant.id());
        assert!(secret.discarded_at().is_none());
        assert!(!secret.is_discarded());
    }

    #[test]
    fn record_rejects_inactive_participant_empty_keys_and_duplicate_version() {
        let program = open_program();
        let mut participant = active_participant(&program);
        let ciphertext = vec![1_u8, 2, 3];

        let first = MatchingSecret::record(
            Uuid::new_v4(),
            &participant,
            "hometown".into(),
            1,
            ciphertext.clone(),
            "vault/keys/matching-1".into(),
            at(2026, 4, 2),
            &[],
        )
        .unwrap();

        let duplicate = MatchingSecret::record(
            Uuid::new_v4(),
            &participant,
            " hometown ".into(),
            1,
            vec![9],
            "vault/keys/matching-2".into(),
            at(2026, 4, 3),
            &[first.clone()],
        );
        assert_eq!(duplicate.err(), Some(InvalidSecret));

        let next_version = MatchingSecret::record(
            Uuid::new_v4(),
            &participant,
            "hometown".into(),
            2,
            vec![4, 5],
            "vault/keys/matching-1".into(),
            at(2026, 4, 3),
            &[first.clone()],
        )
        .unwrap();
        assert_eq!(next_version.version(), 2);

        for (field_key, key_ref) in [("", "vault/k"), (" ", "vault/k"), ("hometown", ""), ("hometown", " ")]
        {
            let result = MatchingSecret::record(
                Uuid::new_v4(),
                &participant,
                field_key.into(),
                3,
                ciphertext.clone(),
                key_ref.into(),
                at(2026, 4, 4),
                &[first.clone(), next_version.clone()],
            );
            assert_eq!(result.err(), Some(InvalidSecret));
        }

        participant.leave(at(2026, 5, 1)).unwrap();
        let after_leave = MatchingSecret::record(
            Uuid::new_v4(),
            &participant,
            "hometown".into(),
            3,
            ciphertext,
            "vault/keys/matching-1".into(),
            at(2026, 5, 2),
            &[first, next_version],
        );
        assert_eq!(after_leave.err(), Some(InvalidSecret));
    }

    #[test]
    fn discard_keeps_ciphertext_and_blocks_reuse_of_same_version() {
        let program = open_program();
        let participant = active_participant(&program);
        let ciphertext = b"keep-me".to_vec();

        let mut secret = MatchingSecret::record(
            Uuid::new_v4(),
            &participant,
            "pet".into(),
            1,
            ciphertext.clone(),
            "vault/keys/matching-1".into(),
            at(2026, 4, 2),
            &[],
        )
        .unwrap();

        secret.discard(at(2026, 5, 1)).unwrap();
        assert_eq!(secret.discarded_at(), Some(at(2026, 5, 1)));
        assert!(secret.is_discarded());
        assert_eq!(secret.encrypted_value(), ciphertext.as_slice());
        assert!(secret.discard(at(2026, 5, 2)).is_err());

        let reuse = MatchingSecret::record(
            Uuid::new_v4(),
            &participant,
            "pet".into(),
            1,
            vec![0],
            "vault/keys/matching-1".into(),
            at(2026, 5, 3),
            &[secret],
        );
        assert_eq!(reuse.err(), Some(InvalidSecret));
    }
}
