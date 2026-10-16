//! Classificação de dados, níveis de sensibilidade, bases legais e métodos de descarte.

use serde::{Deserialize, Serialize};

/// Categorias de dados sujeitas a regras de ciclo de vida e retenção no CApp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataCategory {
    /// Dados cadastrais e de perfil de discentes (matrícula, e-mail institucional, nome, curso).
    StudentPersonalData,

    /// Histórico funcional de mandatos e cargos de gestão em Centro Acadêmico (para prestação de contas estatutária).
    ManagementMandate,

    /// Relatórios financeiros, demonstrativos contábeis e prestação de contas de gestões de CA.
    FinancialAccountability,

    /// Registro nominal de comparecimento eleitoral (controle de quórum e prevenção a voto duplo).
    ElectoralTurnout,

    /// Cédula eleitoral contendo voto secreto cifrado (desprovido de qualquer identificador de eleitor).
    ElectoralBallot,

    /// Trilha técnica temporária de auditoria e integridade de urnas eleitorais durante período de impugnação.
    ElectoralAudit,

    /// Sessões de usuário, refresh tokens e credenciais efêmeras de autenticação.
    SessionAndToken,

    /// Logs de auditoria de segurança e registros de acesso a aplicações (Marco Civil da Internet, art. 15).
    AuditLog,

    /// Métricas de telemetria operacional, latência e diagnóstico de desempenho sem dados pessoais.
    Telemetry,
}

impl DataCategory {
    /// Retorna o nome amigável e canônico da categoria de dado.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::StudentPersonalData => "Dados Discentes e Cadastrais",
            Self::ManagementMandate => "Histórico Funcional de Gestão de CA",
            Self::FinancialAccountability => "Prestação de Contas e Finanças",
            Self::ElectoralTurnout => "Comparecimento Eleitoral (Turnout)",
            Self::ElectoralBallot => "Cédulas Eleitorais e Votos Secretos",
            Self::ElectoralAudit => "Trilha Temporária de Auditoria Eleitoral",
            Self::SessionAndToken => "Sessões e Tokens de Autenticação",
            Self::AuditLog => "Logs de Auditoria de Acesso e Segurança",
            Self::Telemetry => "Telemetria e Métricas Operacionais",
        }
    }
}

/// Nível de sensibilidade da informação para fins de controle de acesso e proteção.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SensitivityLevel {
    /// Informação ostensiva acessível a qualquer membro da comunidade (estatutos, editais, chapas).
    Public,

    /// Informação de circulação restrita à comunidade universitária autenticada da UnB.
    Internal,

    /// Dados pessoais que exigem controle de acesso restrito (matrículas, e-mails, inscrições em eventos).
    Restricted,

    /// Dados críticos de alta confidencialidade (segredos, votos secretos, credenciais ativas, tokens).
    CriticalSensitive,
}

/// Base legal que fundamenta o tratamento dos dados no CApp sob a LGPD e o Marco Civil da Internet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegalBasis {
    /// Consentimento específico do titular (LGPD art. 7º, I).
    Consent,

    /// Cumprimento de obrigação legal ou regulatória (LGPD art. 7º, II; Marco Civil da Internet art. 15).
    LegalOrRegulatoryObligation,

    /// Execução de contrato ou procedimentos preliminares (LGPD art. 7º, V; Termos de Uso do CApp).
    ContractExecution,

    /// Exercício regular de direitos em processo judicial ou administrativo (LGPD art. 7º, VI; prestação de contas).
    RegularExerciseOfRights,

    /// Legítimo interesse do controlador para apoio às atividades estudantis (LGPD art. 7º, IX).
    LegitimateInterest,

    /// Cumprimento de dever estatutário de processo eleitoral com proteção ao sigilo do voto (LGPD art. 11, II, "a").
    ElectoralStatutoryDuty,
}

impl LegalBasis {
    /// Descrição sumária do fundamento jurídico aplicável.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Consent => "LGPD Art. 7º, I (Consentimento)",
            Self::LegalOrRegulatoryObligation => {
                "LGPD Art. 7º, II / Marco Civil Art. 15 (Obrigação Legal)"
            }
            Self::ContractExecution => "LGPD Art. 7º, V (Execução de Termos do CApp)",
            Self::RegularExerciseOfRights => {
                "LGPD Art. 7º, VI (Exercício Regular de Direitos e Prestação de Contas)"
            }
            Self::LegitimateInterest => "LGPD Art. 7º, IX (Legítimo Interesse Operacional)",
            Self::ElectoralStatutoryDuty => {
                "LGPD Art. 11, II, 'a' (Dever Estatutário Eleitoral e Sigilo Democrático)"
            }
        }
    }
}

/// Método técnico empregado para o descarte ou expurgo seguro dos dados após o período de retenção.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisposalMethod {
    /// Exclusão física definitiva das linhas no banco de dados relacional e reutilização de blocos.
    PhysicalPurge,

    /// Anonimização irreversível: substituição de dados pessoais por pseudônimos sintéticos perenes.
    IrreversibleAnonymization,

    /// Destruição irreversível de chaves criptográficas de apuração ou backup (Crypto-shredding).
    CryptoShredding,

    /// Pseudonimização controlada com segregação e isolamento de chaves de correspondência.
    Pseudonymization,
}
