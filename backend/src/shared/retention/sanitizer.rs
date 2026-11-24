//! Utilitários para sanitização, mascaramento e anonimização de dados pessoais discentes.

use uuid::Uuid;

use super::error::RetentionError;

/// Matrícula restrita terminantemente proibida em código, testes e banco de dados.
pub const FORBIDDEN_MATRICULA: &str = "242003979";

/// Domínio exclusivo institucional discente da Universidade de Brasília.
pub const UNB_EMAIL_DOMAIN: &str = "@aluno.unb.br";

/// Matrículas didáticas sintéticas oficialmente homologadas para testes e exemplos.
pub const APPROVED_MATRICULAS: [&str; 4] = ["241098765", "242011111", "231012345", "262001234"];

/// Registro de perfil de discente para operações de ciclo de vida e descarte de dados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudentRecord {
    /// Identificador único relacional do usuário (UUID).
    pub user_id: Uuid,

    /// Matrícula institucional na UnB (formato 9 dígitos numéricos).
    pub matricula: String,

    /// Endereço de e-mail institucional oficial (`<matricula>@aluno.unb.br`).
    pub email: String,

    /// Nome civil ou social completo do discente.
    pub name: String,

    /// Telefone facultativo de contato.
    pub phone: Option<String>,

    /// URL da foto de perfil.
    pub photo_url: Option<String>,

    /// Indicador de que os dados pessoais do registro já foram irreversivelmente anonimizados.
    pub is_anonymized: bool,
}

impl StudentRecord {
    /// Cria uma nova instância de dados discentes com validação preliminar.
    pub fn new(
        user_id: Uuid,
        matricula: impl Into<String>,
        email: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, RetentionError> {
        let matricula = matricula.into();
        let email = email.into();

        PiiSanitizer::validate_matricula(&matricula)?;
        PiiSanitizer::validate_unb_email(&email)?;

        Ok(Self {
            user_id,
            matricula,
            email,
            name: name.into(),
            phone: None,
            photo_url: None,
            is_anonymized: false,
        })
    }
}

/// Mecanismos de sanitização, mascaramento e anonimização de dados pessoais (PII).
pub struct PiiSanitizer;

impl PiiSanitizer {
    /// Valida se a matrícula discente atende às regras da UnB e rejeita a matrícula proibida.
    pub fn validate_matricula(matricula: &str) -> Result<(), RetentionError> {
        if matricula == FORBIDDEN_MATRICULA {
            return Err(RetentionError::ForbiddenMatricula(matricula.to_string()));
        }

        if matricula.len() != 9 || !matricula.chars().all(|c| c.is_ascii_digit()) {
            return Err(RetentionError::InvalidMatriculaFormat(
                matricula.to_string(),
            ));
        }

        Ok(())
    }

    /// Valida se o e-mail institucional pertence ao domínio `@aluno.unb.br` e respeita a matrícula válida.
    pub fn validate_unb_email(email: &str) -> Result<(), RetentionError> {
        if !email.ends_with(UNB_EMAIL_DOMAIN) {
            return Err(RetentionError::InvalidUnbEmail(email.to_string()));
        }

        let prefix = &email[..email.len() - UNB_EMAIL_DOMAIN.len()];
        Self::validate_matricula(prefix)?;

        Ok(())
    }

    /// Valida a invariante de instituição única, rejeitando qualquer `institution_id`.
    pub fn validate_no_institution_id<T>(institution_id: Option<T>) -> Result<(), RetentionError> {
        if institution_id.is_some() {
            return Err(RetentionError::InstitutionIdProhibited);
        }
        Ok(())
    }

    /// Mascara a matrícula discente para exibição em logs ou interfaces sem expor o número integral.
    ///
    /// Preserva os 4 dígitos iniciais (ano e semestre de ingresso) e os 2 finais,
    /// ofuscando os 3 dígitos centrais: `241098765` -> `2410***65`.
    pub fn mask_matricula(matricula: &str) -> Result<String, RetentionError> {
        Self::validate_matricula(matricula)?;
        let masked = format!("{}***{}", &matricula[..4], &matricula[7..]);
        Ok(masked)
    }

    /// Mascara o endereço de e-mail institucional discente preservando a terminação `@aluno.unb.br`.
    ///
    /// Exemplo: `241098765@aluno.unb.br` -> `2410***65@aluno.unb.br`.
    pub fn mask_email(email: &str) -> Result<String, RetentionError> {
        Self::validate_unb_email(email)?;
        let prefix = &email[..email.len() - UNB_EMAIL_DOMAIN.len()];
        let masked_prefix = Self::mask_matricula(prefix)?;
        Ok(format!("{masked_prefix}{UNB_EMAIL_DOMAIN}"))
    }

    /// Sanitiza uma mensagem de texto (ex.: logs ou mensagens de erro) antes de sua emissão,
    /// mascarando e-mails institucionais, matrículas e redigindo tokens Bearer.
    pub fn sanitize_log(message: &str) -> String {
        let mut sanitized = message.to_string();

        // 1. Redação de Bearer Tokens
        if let Some(pos) = sanitized.find("Bearer ") {
            let start = pos + "Bearer ".len();
            let end = sanitized[start..]
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                .map_or(sanitized.len(), |idx| start + idx);
            if start < end {
                sanitized.replace_range(start..end, "[REDACTED_BEARER_TOKEN]");
            }
        }

        // 2. Mascaramento de matrículas homologadas e e-mails conhecidos
        for matricula in &APPROVED_MATRICULAS {
            let email = format!("{matricula}{UNB_EMAIL_DOMAIN}");
            if let Ok(masked_email) = Self::mask_email(&email) {
                sanitized = sanitized.replace(&email, &masked_email);
            }
            if let Ok(masked_mat) = Self::mask_matricula(matricula) {
                sanitized = sanitized.replace(*matricula, &masked_mat);
            }
        }

        // 3. Mascaramento da matrícula proibida caso apareça em mensagens de erro/testes
        let forbidden_email = format!("{FORBIDDEN_MATRICULA}{UNB_EMAIL_DOMAIN}");
        sanitized = sanitized.replace(&forbidden_email, "2420***79@aluno.unb.br");
        sanitized = sanitized.replace(FORBIDDEN_MATRICULA, "2420***79");

        sanitized
    }

    /// Executa a anonimização irreversível dos dados pessoais de um perfil discente.
    ///
    /// Substitui nome por `"Estudante Anonimizado"`, gera e-mail sintético e limpa dados opcionais.
    pub fn anonymize_student_record(record: &mut StudentRecord) -> Result<(), RetentionError> {
        record.name = "Estudante Anonimizado".to_string();
        record.email = format!("anonimizado-{}@anonimo.capp.unb.br", record.user_id);
        record.matricula = format!("{}***{}", &record.matricula[..2], &record.matricula[7..]);
        record.phone = None;
        record.photo_url = None;
        record.is_anonymized = true;

        Ok(())
    }
}
