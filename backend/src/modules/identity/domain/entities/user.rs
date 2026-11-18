//! Conta interna provisionada a partir da identidade Firebase validada.
//!
//! O UID é estável entre os provedores vinculados no Firebase. A conta nasce
//! incompleta; dados institucionais e credenciais têm ciclos de vida separados.
//! Senhas e tokens pertencem ao Firebase e nunca são campos desta entidade.
//!
//! O provisionamento estabelece a identidade interna antes de concluir os dados
//! do perfil. [`User::complete_profile`] centraliza as invariantes da conclusão
//! e a transição para [`UserStatus::Active`]. A aplicação confere a credencial,
//! a validade institucional e a admissibilidade do nome declarado antes de chamar
//! esse método; a entidade não conhece provedores ou contratos HTTP.
//! A reconstituição PostgreSQL usa o mapeamento `sqlx::FromRow` adotado no projeto,
//! e as restrições do banco complementam os caminhos de alteração da entidade.

use crate::modules::identity::domain::UserStatus;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Fotografia persistida da conta interna. Não é um contrato HTTP.
///
/// Os campos privados expõem leitura por métodos e alteração pelo caminho de
/// conclusão validado. O curso pertence à evidência institucional; o e-mail aqui
/// é o escolhido para a credencial interna, nunca um segundo contato do Graph.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    id: Uuid,
    firebase_uid: String,
    name: Option<String>,
    nickname: Option<String>,
    email: Option<String>,
    current_semester: Option<i32>,
    institution_entry_semester: Option<String>,
    status: UserStatus,
    profile_completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_login_at: Option<DateTime<Utc>>,
}

/// Dados exigidos para concluir o perfil; o semestre de ingresso usa AAAA.1/AAAA.2.
///
/// Este agrupamento transporta valores até [`User::complete_profile`]; criá-lo,
/// inclusive com `Default`, não comprova validade. A origem da identidade e do
/// e-mail é conferida na aplicação, não recebida como parte deste formulário.
#[derive(Debug, Clone, Default)]
pub struct ProfileCompletion {
    /// Nome declarado quando o Graph não o informou; não constitui evidência.
    pub name: Option<String>,

    /// Apelido público obrigatório, aparado antes da persistência.
    pub nickname: String,

    /// Período atual do curso, maior ou igual a um.
    pub current_semester: i32,

    /// Semestre do primeiro ingresso na instituição, independente do período atual.
    pub institution_entry_semester: String,
}

/// Uma alteração que não preservaria as regras da conta.
#[derive(Debug, thiserror::Error)]
#[error("Dados de perfil inválidos ou conta inativa")]
pub struct InvalidProfile;

impl User {
    /// Identificador interno estável, independente dos provedores vinculados.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// UID verificado pelo Firebase; não vem do formulário de perfil.
    pub fn firebase_uid(&self) -> &str {
        &self.firebase_uid
    }

    /// Nome declarado para uma ausência do Graph; não substitui o apelido.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Apelido escolhido e normalizado na conclusão do perfil.
    pub fn nickname(&self) -> Option<&str> {
        self.nickname.as_deref()
    }

    /// E-mail da credencial interna confirmado pelo JWT Firebase.
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    /// Estado persistido; concluir o perfil nunca reativa uma conta inativa.
    pub fn status(&self) -> UserStatus {
        self.status
    }

    /// Período atual do curso, que pode divergir do tempo desde o ingresso.
    pub fn current_semester(&self) -> Option<i32> {
        self.current_semester
    }

    /// Semestre de ingresso na instituição no formato AAAA.1 ou AAAA.2.
    pub fn institution_entry_semester(&self) -> Option<&str> {
        self.institution_entry_semester.as_deref()
    }

    /// Instante da primeira conclusão, preservado em atualizações posteriores.
    pub fn profile_completed_at(&self) -> Option<DateTime<Utc>> {
        self.profile_completed_at
    }

    /// Instante de provisionamento da conta interna.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Instante da última atualização do perfil.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Última autenticação institucional registrada no provisionamento idempotente.
    pub fn last_login_at(&self) -> Option<DateTime<Utc>> {
        self.last_login_at
    }

    /// Conclui ou atualiza o perfil sem permitir reativação de uma conta inativa.
    ///
    /// # Parâmetros e pré-condições
    ///
    /// - `data`: valores do formulário. Apelido e nome declarado são aparados;
    ///   o ingresso deve ter exatamente o formato `AAAA.1` ou `AAAA.2`.
    /// - `email`: endereço não vazio da credencial interna, já conferido no JWT
    ///   Firebase pela aplicação. Não é aparado nem validado como endereço aqui.
    /// - `now`: instante atribuído à alteração; não é obtido de um relógio global.
    ///
    /// A aplicação deve confirmar a autenticação por senha, a evidência
    /// institucional e que um nome declarado pode completar uma ausência do Graph.
    /// Este método verifica a estrutura do perfil, não essas condições externas.
    ///
    /// # Retorno e alteração de estado
    ///
    /// Em `Ok(())`, substitui os dados internos do perfil, normaliza nome opcional,
    /// muda o estado para [`UserStatus::Active`] e atualiza `updated_at`. A primeira
    /// data de conclusão é preservada nas chamadas posteriores. ID, UID, criação
    /// e último login não mudam. A alteração ocorre somente em memória; o chamador
    /// precisa persistir a entidade pelo repositório.
    ///
    /// # Erros
    ///
    /// Retorna [`InvalidProfile`] para conta inativa, apelido vazio após `trim`,
    /// período menor que um, e-mail vazio ou ingresso fora do formato exigido.
    /// Toda a validação precede as atribuições, deixando o objeto intacto em erro.
    /// O formato de ingresso não comprova matrícula, plausibilidade do ano ou
    /// correspondência entre tempo de ingresso e período atual do curso.
    ///
    /// # Exemplos
    ///
    /// ```
    /// use backend::modules::identity::domain::{
    ///     UserStatus, entities::user::{InvalidProfile, ProfileCompletion, User},
    /// };
    /// use chrono::Utc;
    /// use uuid::Uuid;
    ///
    /// # fn main() -> Result<(), InvalidProfile> {
    /// let now = Utc::now();
    /// let mut user = User::provisioned(Uuid::new_v4(), "firebase-uid-exemplo".into(), now);
    /// user.complete_profile(ProfileCompletion {
    ///     nickname: " Pessoa ".into(), current_semester: 4,
    ///     institution_entry_semester: "2024.1".into(), name: None,
    /// }, "pessoa@example.test", now)?;
    /// assert_eq!(user.nickname(), Some("Pessoa"));
    /// assert_eq!(user.status(), UserStatus::Active);
    /// # Ok(())
    /// # }
    /// ```
    pub fn complete_profile(
        &mut self,
        data: ProfileCompletion,
        email: &str,
        now: DateTime<Utc>,
    ) -> Result<(), InvalidProfile> {
        let term = data.institution_entry_semester.as_bytes();
        if self.status == UserStatus::Inactive
            || data.nickname.trim().is_empty()
            || data.current_semester < 1
            || email.is_empty()
            || term.len() != 6
            || !term[..4].iter().all(u8::is_ascii_digit)
            || term[4] != b'.'
            || !matches!(term[5], b'1' | b'2')
        {
            return Err(InvalidProfile);
        }

        self.nickname = Some(data.nickname.trim().into());
        self.name = optional_text(data.name);
        self.email = Some(email.into());
        self.current_semester = Some(data.current_semester);
        self.institution_entry_semester = Some(data.institution_entry_semester);
        self.status = UserStatus::Active;

        self.profile_completed_at.get_or_insert(now);

        self.updated_at = now;

        Ok(())
    }

    /// Cria a representação incompleta usada pelo provisionamento e pelos testes.
    ///
    /// `id` é a identidade interna escolhida pelo chamador; `firebase_uid` deve
    /// identificar uma conta já autenticada. `now` inicializa criação, atualização
    /// e último login. Os dados de perfil e a data de conclusão começam ausentes.
    ///
    /// Não consulta o Firebase, valida o UID ou grava no banco. O adaptador
    /// PostgreSQL atual obtém a entidade pelo UPSERT e pelo mapeamento SQLx;
    /// este construtor também permite implementar outras portas e preparar fixtures.
    pub fn provisioned(id: Uuid, firebase_uid: String, now: DateTime<Utc>) -> Self {
        Self {
            id,
            firebase_uid,
            name: None,
            nickname: None,
            email: None,
            current_semester: None,
            institution_entry_semester: None,
            status: UserStatus::Incomplete,
            profile_completed_at: None,
            created_at: now,
            updated_at: now,
            last_login_at: Some(now),
        }
    }
}

/// Normaliza um complemento opcional antes de armazená-lo no perfil interno.
///
/// Preserva `None`, apara o texto presente e converte vazio/em branco em `None`.
/// Não infere conteúdo nem decide se o complemento é permitido para a evidência;
/// essa checagem de origem pertence ao caso de uso anterior à alteração da conta.
fn optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> ProfileCompletion {
        ProfileCompletion {
            nickname: " Apelido ".into(),
            current_semester: 4,
            institution_entry_semester: "2023.2".into(),
            ..Default::default()
        }
    }
    #[test]
    fn completion_preserves_identity_and_first_completion_date() {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let mut user = User::provisioned(id, "firebase-uid".into(), now);

        assert_eq!(user.status(), UserStatus::Incomplete);

        user.complete_profile(data(), "pessoal@example.test", now)
            .unwrap();

        assert_eq!(user.id(), id);
        assert_eq!(user.firebase_uid(), "firebase-uid");
        assert_eq!(user.nickname(), Some("Apelido"));
        assert_eq!(user.status(), UserStatus::Active);

        user.complete_profile(data(), "novo@example.test", now + chrono::Duration::days(1))
            .unwrap();

        assert_eq!(user.profile_completed_at(), Some(now));
        assert_eq!(user.email(), Some("novo@example.test"));
    }
    #[test]
    fn invalid_or_disabled_profiles_do_not_mutate_state() {
        for (nickname, semester, entry) in [
            (" ", 4, "2023.1"),
            ("ok", 0, "2023.1"),
            ("ok", -1, "2023.1"),
            ("ok", 2, "2023.3"),
            ("ok", 2, "2023"),
            ("ok", 2, "20231"),
            ("ok", 2, "áááá.1"),
        ] {
            let mut user = User::provisioned(Uuid::new_v4(), "uid".into(), Utc::now());

            assert!(
                user.complete_profile(
                    ProfileCompletion {
                        nickname: nickname.into(),
                        current_semester: semester,
                        institution_entry_semester: entry.into(),
                        ..Default::default()
                    },
                    "email@example.test",
                    Utc::now()
                )
                .is_err()
            );
            assert_eq!(user.status(), UserStatus::Incomplete);
            assert!(user.nickname().is_none());
        }

        let mut user = User::provisioned(Uuid::new_v4(), "uid".into(), Utc::now());

        user.status = UserStatus::Inactive;

        assert!(
            user.complete_profile(data(), "email@example.test", Utc::now())
                .is_err()
        );
        assert_eq!(user.status(), UserStatus::Inactive);
    }
}
