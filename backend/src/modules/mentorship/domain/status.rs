//! Estados, papéis e estratégias persistidos pelo apadrinhamento.

/// Ciclo de vida do programa: rascunho, aberto ou encerrado.
///
/// Mapeado ao enum PostgreSQL previsto `mentorship_program_status`, com rótulos
/// `draft`, `open` e `closed`. O derive não cria o tipo no banco; isso fica para
/// a migration futura. A vigência das datas continua sendo conferida à parte.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "mentorship_program_status", rename_all = "lowercase")]
pub enum MentorshipProgramStatus {
    /// Programa ainda em configuração; não aceita inscrição, vínculo nem execução.
    #[default]
    Draft,
    /// Programa disponível para os fluxos habilitados, dentro das datas e regras.
    Open,
    /// Programa encerrado para novas participações, vínculos e execuções.
    Closed,
}

/// Método ativo de formação de sugestões ou pares.
///
/// Mapeado ao enum PostgreSQL previsto `matching_strategy`, com rótulos `manual`,
/// `criteria`, `secrets` e `neural`. O dicionário nomeia esses quatro métodos e
/// não publica outra lista de códigos. A execução futura pode gravar a mesma
/// estratégia no momento em que for solicitada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "matching_strategy", rename_all = "lowercase")]
pub enum MatchingStrategy {
    /// Atribuição ou confirmação manual de pares.
    Manual,
    /// Formação assistida por critérios ponderados.
    Criteria,
    /// Formação assistida por comparação de segredos.
    Secrets,
    /// Formação assistida por modelo neural.
    Neural,
}

/// Papel do participante no programa: veterano/padrinho ou calouro/afilhado.
///
/// Mapeado ao enum PostgreSQL previsto `mentorship_participant_role`, com
/// rótulos `veterano` e `calouro`. Texto vazio, ausente ou diferente desses
/// dois não produz papel — ver [`MentorshipParticipantRole::parse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "mentorship_participant_role", rename_all = "lowercase")]
pub enum MentorshipParticipantRole {
    /// Veterano no papel de padrinho.
    Veterano,
    /// Calouro no papel de afilhado.
    Calouro,
}

impl MentorshipParticipantRole {
    /// Interpreta o rótulo informado na inscrição ou persistido.
    ///
    /// Aceita somente `veterano` e `calouro` após `trim`. `None`, string vazia
    /// (após aparar) ou qualquer outro texto devolve `None`.
    pub fn parse(label: Option<&str>) -> Option<Self> {
        match label?.trim() {
            "veterano" => Some(Self::Veterano),
            "calouro" => Some(Self::Calouro),
            _ => None,
        }
    }

    /// Rótulo estável usado na persistência e nos contratos textuais.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Veterano => "veterano",
            Self::Calouro => "calouro",
        }
    }

    /// Indica se o papel é o de padrinho.
    pub fn is_mentor(self) -> bool {
        matches!(self, Self::Veterano)
    }

    /// Indica se o papel é o de afilhado.
    pub fn is_mentee(self) -> bool {
        matches!(self, Self::Calouro)
    }
}

/// Situação da inscrição no programa: habilitada ou apenas histórica.
///
/// Mapeado ao enum PostgreSQL previsto `mentorship_participant_status`, com
/// rótulos `active` e `inactive`. `inactive` não desativa a conta do usuário
/// nem afeta inscrição em outro programa.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "mentorship_participant_status", rename_all = "lowercase")]
pub enum MentorshipParticipantStatus {
    /// Participante habilitado no programa, conforme papel e elegibilidade.
    #[default]
    Active,
    /// Participante que deixou de estar habilitado; permanece no histórico.
    Inactive,
}

/// Ciclo de vida de uma execução de matching.
///
/// Mapeado ao enum PostgreSQL previsto `matching_run_status`, com rótulos
/// `pending`, `running`, `completed` e `failed`. A execução nasce `pending`,
/// passa a `running` e termina em `completed` ou `failed`. `completed` pode
/// produzir zero sugestões.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "matching_run_status", rename_all = "lowercase")]
pub enum MatchingRunStatus {
    /// Execução criada e aguardando processamento.
    #[default]
    Pending,
    /// Processamento em andamento.
    Running,
    /// Processamento concluído sem falha operacional.
    Completed,
    /// Execução falhou; a recuperação segue a política do fluxo.
    Failed,
}

/// Decisão sobre uma sugestão de par produzida por uma execução.
///
/// Mapeado ao enum PostgreSQL previsto `matching_suggestion_status`, com
/// rótulos `pending`, `accepted`, `rejected` e `expired`. A sugestão nasce
/// `pending`. Recusar ou expirar não cria vínculo; aceitar só ocorre junto com
/// a criação do vínculo correspondente.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "matching_suggestion_status", rename_all = "lowercase")]
pub enum MatchingSuggestionStatus {
    /// Sugestão ainda aguardando uma decisão.
    #[default]
    Pending,
    /// Sugestão aceita e convertida em vínculo após revalidar as regras.
    Accepted,
    /// Sugestão recusada pelo fluxo autorizado.
    Rejected,
    /// Sugestão que perdeu a validade por mudança de contexto ou política.
    Expired,
}

/// Situação do vínculo entre padrinho e afilhado.
///
/// Mapeado ao enum PostgreSQL previsto `mentorship_match_status`, com rótulos
/// `active` e `ended`. Diferente da situação da sugestão que originou o
/// vínculo; atribuição manual pode não ter sugestão.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "mentorship_match_status", rename_all = "lowercase")]
pub enum MentorshipMatchStatus {
    /// Vínculo confirmado e vigente entre padrinho e afilhado.
    #[default]
    Active,
    /// Vínculo encerrado, preservado no histórico.
    Ended,
}
