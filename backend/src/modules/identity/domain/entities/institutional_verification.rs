//! Resultado de verificação institucional e associação de dados acadêmicos.
//!
//! A tabela guarda um registro por usuário interno, com chave primária e FK para app_user.
//! Uma nova resposta conclusiva pode substituir a anterior; indisponibilidade do
//! provedor não é registrada como ausência de vínculo. O repositório só reutiliza
//! resultados conclusivos, não expirados e compatíveis com a configuração atual.

use std::{error::Error, fmt};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::identity::domain::{
    entities::academic_data::{AcademicData, AcademicStanding},
    rules::institutional_verification::{
        validate_profile_name, validate_student_course, validate_verification_period,
    },
};

/// Erros de validação e consistência da verificação institucional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstitutionalVerificationError {
    /// O período de validade possui expiração anterior ou igual ao instante de checagem.
    InconsistentTimestamps {
        /// Instante da checagem.
        checked_at: DateTime<Utc>,
        /// Instante de expiração.
        expires_at: DateTime<Utc>,
    },
    /// Discente com grupo estudantil confirmado exige a presença de um curso não vazio.
    MissingCourseForStudent,
    /// Usuário não reconhecido como discente não pode conter curso associado.
    CourseProvidedForNonStudent,
    /// O nome do curso não pode ser vazio ou composto unicamente por espaços em branco.
    EmptyCourseName,
}

impl fmt::Display for InstitutionalVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InconsistentTimestamps {
                checked_at,
                expires_at,
            } => {
                write!(
                    f,
                    "período de verificação inválido: expiração ({expires_at}) deve ser posterior à checagem ({checked_at})"
                )
            }
            Self::MissingCourseForStudent => {
                write!(
                    f,
                    "estudante com grupo confirmado exige a identificação do curso"
                )
            }
            Self::CourseProvidedForNonStudent => {
                write!(
                    f,
                    "não é permitido associar curso a usuário sem vínculo discente confirmado"
                )
            }
            Self::EmptyCourseName => {
                write!(f, "o nome do curso não pode ser vazio ou em branco")
            }
        }
    }
}

impl Error for InstitutionalVerificationError {}

/// Resultado persistido da última verificação institucional registrada.
///
/// Não representa uma sessão nem um usuário interno do CApp.
/// Tokens e credenciais não são armazenados neste registro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstitutionalVerification {
    /// Conta interna proprietária da evidência.
    user_id: Uuid,

    /// Fotografia externa obtida na mesma consulta que confirmou o vínculo.
    profile_name: Option<String>,

    /// Curso externo não vazio, necessário para confirmar o vínculo estudantil.
    course_name: Option<String>,

    /// Instituição à qual a consulta se refere.
    institution_id: Uuid,

    /// Identificador Microsoft confrontado com a identidade Firebase.
    microsoft_object_id: Uuid,

    /// Tenant institucional usado durante a verificação.
    microsoft_tenant_id: Uuid,

    /// Grupo de estudantes consultado durante a verificação.
    microsoft_student_group_id: Uuid,

    /// Indica se o Graph confirmou grupo estudantil e curso não vazio.
    student_membership_confirmed: bool,

    /// Início da consulta institucional, usado para ordenar atualizações.
    checked_at: DateTime<Utc>,

    /// Limite para reutilizar uma confirmação positiva.
    expires_at: DateTime<Utc>,
}

impl InstitutionalVerification {
    /// Cria uma nova verificação institucional validando as invariantes de negócio.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_id: Uuid,
        profile_name: Option<String>,
        course_name: Option<String>,
        institution_id: Uuid,
        microsoft_object_id: Uuid,
        microsoft_tenant_id: Uuid,
        microsoft_student_group_id: Uuid,
        student_membership_confirmed: bool,
        checked_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, InstitutionalVerificationError> {
        validate_verification_period(checked_at, expires_at)?;
        let validated_course =
            validate_student_course(student_membership_confirmed, course_name.as_deref())?;
        let validated_profile = validate_profile_name(profile_name.as_deref());

        Ok(Self {
            user_id,
            profile_name: validated_profile,
            course_name: validated_course,
            institution_id,
            microsoft_object_id,
            microsoft_tenant_id,
            microsoft_student_group_id,
            student_membership_confirmed,
            checked_at,
            expires_at,
        })
    }

    /// Reconstrói uma verificação a partir de dados de persistência já validados ou legados.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        user_id: Uuid,
        profile_name: Option<String>,
        course_name: Option<String>,
        institution_id: Uuid,
        microsoft_object_id: Uuid,
        microsoft_tenant_id: Uuid,
        microsoft_student_group_id: Uuid,
        student_membership_confirmed: bool,
        checked_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, InstitutionalVerificationError> {
        let profile_name = profile_name.filter(|s| !s.trim().is_empty());
        Ok(Self {
            user_id,
            profile_name,
            course_name,
            institution_id,
            microsoft_object_id,
            microsoft_tenant_id,
            microsoft_student_group_id,
            student_membership_confirmed,
            checked_at,
            expires_at,
        })
    }

    /// Conta interna proprietária da evidência.
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Fotografia do nome do perfil retornado pela instituição.
    pub fn profile_name(&self) -> Option<&str> {
        self.profile_name.as_deref()
    }

    /// Nome do curso verificado, quando aplicável.
    pub fn course_name(&self) -> Option<&str> {
        self.course_name.as_deref()
    }

    /// Identificador da instituição à qual a verificação se refere.
    pub fn institution_id(&self) -> Uuid {
        self.institution_id
    }

    /// Identificador do objeto Microsoft associado.
    pub fn microsoft_object_id(&self) -> Uuid {
        self.microsoft_object_id
    }

    /// UUID do tenant do diretório Microsoft.
    pub fn microsoft_tenant_id(&self) -> Uuid {
        self.microsoft_tenant_id
    }

    /// UUID do grupo de estudantes consultado.
    pub fn microsoft_student_group_id(&self) -> Uuid {
        self.microsoft_student_group_id
    }

    /// Indica se a participação no grupo e o curso foram confirmados.
    pub fn student_membership_confirmed(&self) -> bool {
        self.student_membership_confirmed
    }

    /// Instante de início da checagem institucional.
    pub fn checked_at(&self) -> DateTime<Utc> {
        self.checked_at
    }

    /// Instante de expiração da validade da evidência.
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    /// Indica se a evidência atesta que o usuário é estudante regular e possui curso.
    pub fn is_student(&self) -> bool {
        self.student_membership_confirmed && self.course_name.is_some()
    }

    /// Verifica se a evidência é válida no instante informado.
    pub fn is_valid_at(&self, at: DateTime<Utc>) -> bool {
        at >= self.checked_at && at < self.expires_at
    }

    /// Verifica se a evidência já expirou no instante informado.
    pub fn is_expired(&self, at: DateTime<Utc>) -> bool {
        at >= self.expires_at
    }

    /// Confere se tenant e grupo correspondem à configuração institucional vigente.
    pub fn has_matching_tenant_and_group(&self, tenant_id: Uuid, group_id: Uuid) -> bool {
        self.microsoft_tenant_id == tenant_id && self.microsoft_student_group_id == group_id
    }

    /// Extrai os dados acadêmicos associados a esta verificação institucional.
    pub fn academic_data(&self) -> AcademicData {
        let standing = if self.is_student() {
            AcademicStanding::ActiveStudent
        } else {
            AcademicStanding::NonStudent
        };

        AcademicData::new(
            self.user_id,
            self.institution_id,
            self.course_name.clone(),
            standing,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn should_create_valid_student_verification() {
        let now = Utc::now();
        let verification = InstitutionalVerification::new(
            Uuid::new_v4(),
            Some("Estudante Teste".into()),
            Some("Engenharia de Software".into()),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            true,
            now,
            now + Duration::days(120),
        )
        .expect("should create valid verification");

        assert!(verification.is_student());
        assert_eq!(verification.course_name(), Some("Engenharia de Software"));
        assert_eq!(verification.profile_name(), Some("Estudante Teste"));
        assert!(verification.is_valid_at(now + Duration::hours(1)));
        assert!(!verification.is_expired(now + Duration::hours(1)));

        let academic = verification.academic_data();
        assert!(academic.is_student());
        assert_eq!(academic.standing(), AcademicStanding::ActiveStudent);
    }

    #[test]
    fn should_create_valid_non_student_verification() {
        let now = Utc::now();
        let verification = InstitutionalVerification::new(
            Uuid::new_v4(),
            None,
            None,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            false,
            now,
            now + Duration::days(120),
        )
        .expect("should create valid non-student verification");

        assert!(!verification.is_student());
        assert_eq!(verification.course_name(), None);

        let academic = verification.academic_data();
        assert!(!academic.is_student());
        assert_eq!(academic.standing(), AcademicStanding::NonStudent);
    }

    #[test]
    fn should_fail_when_timestamps_are_inconsistent() {
        let now = Utc::now();
        let result = InstitutionalVerification::new(
            Uuid::new_v4(),
            None,
            None,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            false,
            now,
            now - Duration::seconds(1),
        );

        assert!(matches!(
            result,
            Err(InstitutionalVerificationError::InconsistentTimestamps { .. })
        ));
    }

    #[test]
    fn should_fail_when_student_has_no_course() {
        let now = Utc::now();
        let result = InstitutionalVerification::new(
            Uuid::new_v4(),
            None,
            None,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            true,
            now,
            now + Duration::days(120),
        );

        assert_eq!(
            result.unwrap_err(),
            InstitutionalVerificationError::MissingCourseForStudent
        );
    }

    #[test]
    fn should_fail_when_non_student_has_course() {
        let now = Utc::now();
        let result = InstitutionalVerification::new(
            Uuid::new_v4(),
            None,
            Some("Computação".into()),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            false,
            now,
            now + Duration::days(120),
        );

        assert_eq!(
            result.unwrap_err(),
            InstitutionalVerificationError::CourseProvidedForNonStudent
        );
    }
}
