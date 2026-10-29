//! Definição canônica das permissões de acesso do sistema CApp.
//!
//! Mapeia todas as capacidades administrativas discriminadas no Relatório de
//! Features (§ 2.4, § 2.7, § 2.10) e nas especificações de negócio da UnB.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::scope::Scope;

/// Erros decorrentes de conversão ou validação de permissões.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PermissionError {
    /// Identificador textual de permissão não reconhecido.
    #[error("Permissão desconhecida: '{0}'.")]
    UnknownPermission(String),
}

/// Permissões formais do sistema CApp.
///
/// Cada permissão expressa uma capacidade específica vinculada a um [`Scope`].
/// Nas funcionalidades de fluxo editorial com proposição e homologação (notícias e eventos),
/// a posse de permissão de homologação implica automaticamente a permissão de publicação direta,
/// conforme preconizado no Relatório de Features § 2.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    // ==========================================
    // Escopo: Gestão do Centro Acadêmico
    // ==========================================
    /// Criar, alterar e excluir cargos da gestão, bem como vincular e desvincular integrantes.
    ManageMembers,

    /// Enviar documentação comprobatória para solicitar selo oficial do CA.
    RequestCaVerification,

    /// Criar e publicar notícias institucionais diretamente, sem exigir homologação prévia.
    PublishNewsDirect,

    /// Avaliar, aprovar ou rejeitar propostas de notícias enviadas pela comunidade ou integrantes.
    ApproveNews,

    /// Compartilhar notícias com outros Centros Acadêmicos parceiros.
    ShareNews,

    /// Criar, divulgar e gerenciar eventos oficiais diretamente, sem homologação prévia.
    PublishEventsDirect,

    /// Avaliar, homologar ou rejeitar eventos propostos pela comunidade discente.
    ApproveEvents,

    /// Responder a demandas estudantis, atualizar status e encaminhar demandas institucionais.
    ManageDemands,

    /// Publicar apostilas, provas anteriores, PDFs e links organizados por disciplinas.
    ManageMaterials,

    /// Nomear e destituir integrantes da comissão eleitoral independente.
    ManageElectoralCommission,

    /// Cadastrar e atualizar perguntas frequentes e respostas oficiais do CA.
    ManageFaq,

    /// Organizar o programa de apadrinhamento de calouros e configurar regras de pareamento.
    ManageMentorship,

    /// Publicar relatórios contábeis, receitas, despesas e demonstrativos financeiros do CA.
    PublishFinancialStatements,

    /// Criar enquetes e consultas deliberativas para os estudantes do curso.
    CreateStudentPolls,

    /// Registrar ocorrências individuais e advertências a estudantes no âmbito do CA.
    RegisterStudentStrikes,

    /// Registrar advertências ou ocorrências formais recebidas pelo próprio Centro Acadêmico.
    RegisterCaStrikes,

    /// Configurar os critérios, gradações e regras de aplicação de strikes do CA.
    ConfigureStrikeRules,

    // ==========================================
    // Escopo: Comissão Eleitoral
    // ==========================================
    /// Configurar calendários, regras e parâmetros do pleito eleitoral do CA.
    ConfigureElection,

    /// Homologar, deferir e gerenciar chapas concorrentes à gestão.
    ManageSlates,

    /// Conduzir a contagem dos votos e lavrar a ata oficial de apuração eleitoral.
    TallyVotes,

    // ==========================================
    // Escopo: Administração Geral da Plataforma
    // ==========================================
    /// Gestão institucional global e sustentação de integridade da plataforma.
    AdministerPlatform,

    /// Avaliar documentação submetida por Centros Acadêmicos e emitir selo de verificação.
    VerifyAcademicCenter,
}

impl Permission {
    /// Retorna o identificador textual padronizado da permissão.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ManageMembers => "management:manage_members",
            Self::RequestCaVerification => "management:request_verification",
            Self::PublishNewsDirect => "news:publish_direct",
            Self::ApproveNews => "news:approve",
            Self::ShareNews => "news:share",
            Self::PublishEventsDirect => "events:publish_direct",
            Self::ApproveEvents => "events:approve",
            Self::ManageDemands => "demands:manage",
            Self::ManageMaterials => "materials:manage",
            Self::ManageElectoralCommission => "elections:manage_commission",
            Self::ManageFaq => "faq:manage",
            Self::ManageMentorship => "mentorship:manage",
            Self::PublishFinancialStatements => "governance:publish_finances",
            Self::CreateStudentPolls => "governance:create_polls",
            Self::RegisterStudentStrikes => "strikes:register_student",
            Self::RegisterCaStrikes => "strikes:register_ca",
            Self::ConfigureStrikeRules => "strikes:configure_rules",

            Self::ConfigureElection => "elections:configure",
            Self::ManageSlates => "elections:manage_slates",
            Self::TallyVotes => "elections:tally_votes",

            Self::AdministerPlatform => "platform:admin",
            Self::VerifyAcademicCenter => "platform:verify_ca",
        }
    }

    /// Retorna a descrição formal legível da permissão conforme o Relatório de Features § 2.4.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::ManageMembers => "Administrar integrantes da gestão",
            Self::RequestCaVerification => "Solicitar a verificação do CA",
            Self::PublishNewsDirect => "Publicar notícias diretamente",
            Self::ApproveNews => "Homologar notícias",
            Self::ShareNews => "Compartilhar notícias",
            Self::PublishEventsDirect => "Publicar e gerenciar eventos diretamente",
            Self::ApproveEvents => "Homologar eventos",
            Self::ManageDemands => "Gerenciar demandas",
            Self::ManageMaterials => "Gerenciar materiais",
            Self::ManageElectoralCommission => "Gerenciar comissão eleitoral",
            Self::ManageFaq => "Gerenciar o FAQ",
            Self::ManageMentorship => "Gerenciar o apadrinhamento",
            Self::PublishFinancialStatements => "Publicar a prestação de contas",
            Self::CreateStudentPolls => "Criar consultas estudantis",
            Self::RegisterStudentStrikes => "Registrar strikes de estudantes",
            Self::RegisterCaStrikes => "Registrar strikes do CA",
            Self::ConfigureStrikeRules => "Configurar as regras de strikes",

            Self::ConfigureElection => "Configurar eleições",
            Self::ManageSlates => "Gerenciar chapas",
            Self::TallyVotes => "Apurar votos",

            Self::AdministerPlatform => "Administrar plataforma",
            Self::VerifyAcademicCenter => "Verificar Centro Acadêmico",
        }
    }

    /// Retorna o escopo ao qual a permissão está subordinada.
    pub const fn scope(&self) -> Scope {
        match self {
            Self::ManageMembers
            | Self::RequestCaVerification
            | Self::PublishNewsDirect
            | Self::ApproveNews
            | Self::ShareNews
            | Self::PublishEventsDirect
            | Self::ApproveEvents
            | Self::ManageDemands
            | Self::ManageMaterials
            | Self::ManageElectoralCommission
            | Self::ManageFaq
            | Self::ManageMentorship
            | Self::PublishFinancialStatements
            | Self::CreateStudentPolls
            | Self::RegisterStudentStrikes
            | Self::RegisterCaStrikes
            | Self::ConfigureStrikeRules => Scope::CaManagement,

            Self::ConfigureElection | Self::ManageSlates | Self::TallyVotes => {
                Scope::ElectoralCommission
            }

            Self::AdministerPlatform | Self::VerifyAcademicCenter => Scope::PlatformAdministration,
        }
    }

    /// Retorna as permissões acessórias implicitamente concedidas por esta permissão.
    ///
    /// # Regra de Homologação vs Publicação Direta (Relatório de Features § 2.4)
    /// Nas funcionalidades editoriais que admitem proposição e homologação:
    /// - [`Permission::ApproveNews`] concede implicitamente [`Permission::PublishNewsDirect`].
    /// - [`Permission::ApproveEvents`] concede implicitamente [`Permission::PublishEventsDirect`].
    /// - Permissões de publicação direta isoladas **NÃO** implicam homologação de terceiros.
    pub fn implied_permissions(&self) -> Vec<Permission> {
        match self {
            Self::ApproveNews => vec![Self::PublishNewsDirect],
            Self::ApproveEvents => vec![Self::PublishEventsDirect],
            _ => Vec::new(),
        }
    }

    /// Verifica se a permissão requerida é atendida por uma permissão concedida.
    ///
    /// Uma permissão requerida é satisfeita se a concedida for idêntica ou se
    /// estiver incluída no conjunto de implicações diretas da concedida.
    pub fn is_satisfied_by(&self, granted: &Permission) -> bool {
        if self == granted {
            return true;
        }
        granted.implied_permissions().contains(self)
    }

    /// Verifica se a permissão requerida é satisfeita por qualquer permissão contida no conjunto.
    pub fn is_satisfied_by_any<'a>(
        &self,
        granted_permissions: impl IntoIterator<Item = &'a Permission>,
    ) -> bool {
        granted_permissions
            .into_iter()
            .any(|granted| self.is_satisfied_by(granted))
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Permission {
    type Err = PermissionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase();
        match normalized.as_str() {
            "management:manage_members"
            | "administrar integrantes da gestão"
            | "administrar integrantes da gestao"
            | "administrar perfis e integrantes da gestão"
            | "administrar perfis e integrantes da gestao" => Ok(Self::ManageMembers),

            "management:request_verification"
            | "solicitar a verificação do ca"
            | "solicitar a verificacao do ca"
            | "solicitar verificação do ca"
            | "solicitar verificacao do ca" => Ok(Self::RequestCaVerification),

            "news:publish_direct"
            | "publicar notícias diretamente"
            | "publicar noticias diretamente" => Ok(Self::PublishNewsDirect),

            "news:approve" | "homologar notícias" | "homologar noticias" => Ok(Self::ApproveNews),

            "news:share" | "compartilhar notícias" | "compartilhar noticias" => {
                Ok(Self::ShareNews)
            }

            "events:publish_direct"
            | "publicar e gerenciar eventos diretamente"
            | "publicar eventos diretamente" => Ok(Self::PublishEventsDirect),

            "events:approve" | "homologar eventos" => Ok(Self::ApproveEvents),

            "demands:manage" | "gerenciar demandas" => Ok(Self::ManageDemands),

            "materials:manage" | "gerenciar materiais" => Ok(Self::ManageMaterials),

            "elections:manage_commission"
            | "gerenciar comissão eleitoral"
            | "gerenciar comissao eleitoral" => Ok(Self::ManageElectoralCommission),

            "faq:manage" | "gerenciar o faq" | "gerenciar faq" => Ok(Self::ManageFaq),

            "mentorship:manage" | "gerenciar o apadrinhamento" | "gerenciar apadrinhamento" => {
                Ok(Self::ManageMentorship)
            }

            "governance:publish_finances"
            | "publicar a prestação de contas"
            | "publicar a prestacao de contas"
            | "publicar prestação de contas"
            | "publicar prestacao de contas" => Ok(Self::PublishFinancialStatements),

            "governance:create_polls" | "criar consultas estudantis" => {
                Ok(Self::CreateStudentPolls)
            }

            "strikes:register_student" | "registrar strikes de estudantes" => {
                Ok(Self::RegisterStudentStrikes)
            }

            "strikes:register_ca" | "registrar strikes do ca" => Ok(Self::RegisterCaStrikes),

            "strikes:configure_rules"
            | "configurar as regras de strikes"
            | "configurar regras de strikes" => Ok(Self::ConfigureStrikeRules),

            "elections:configure"
            | "configurar a eleição"
            | "configurar a eleicao"
            | "configurar eleições"
            | "configurar eleicoes" => Ok(Self::ConfigureElection),

            "elections:manage_slates" | "gerenciar chapas" => Ok(Self::ManageSlates),

            "elections:tally_votes" | "apurar votos" | "apuração" | "apuracao" => {
                Ok(Self::TallyVotes)
            }

            "platform:admin" | "administrar plataforma" => Ok(Self::AdministerPlatform),

            "platform:verify_ca" | "verificar centro acadêmico" | "verificar centro academico" => {
                Ok(Self::VerifyAcademicCenter)
            }

            _ => Err(PermissionError::UnknownPermission(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_permissions_from_identifiers_and_descriptions() {
        assert_eq!(
            "news:publish_direct".parse::<Permission>().unwrap(),
            Permission::PublishNewsDirect
        );
        assert_eq!(
            "Publicar notícias diretamente"
                .parse::<Permission>()
                .unwrap(),
            Permission::PublishNewsDirect
        );
        assert_eq!(
            "Homologar notícias".parse::<Permission>().unwrap(),
            Permission::ApproveNews
        );
        assert_eq!(
            "Administrar integrantes da gestão"
                .parse::<Permission>()
                .unwrap(),
            Permission::ManageMembers
        );
    }

    #[test]
    fn should_enforce_homologation_implies_direct_publication_rule() {
        // Quem tem homologação de notícias satisfaz publicação direta
        assert!(Permission::PublishNewsDirect.is_satisfied_by(&Permission::ApproveNews));

        // Quem tem APENAS publicação direta NÃO satisfaz homologação
        assert!(!Permission::ApproveNews.is_satisfied_by(&Permission::PublishNewsDirect));

        // Quem tem homologação de eventos satisfaz publicação direta de eventos
        assert!(Permission::PublishEventsDirect.is_satisfied_by(&Permission::ApproveEvents));

        // Quem tem APENAS publicação direta de eventos NÃO satisfaz homologação de eventos
        assert!(!Permission::ApproveEvents.is_satisfied_by(&Permission::PublishEventsDirect));

        // Homologação de notícias NÃO implica publicação direta de eventos (domínios segregados)
        assert!(!Permission::PublishEventsDirect.is_satisfied_by(&Permission::ApproveNews));
    }

    #[test]
    fn should_map_permissions_to_correct_scopes() {
        assert_eq!(Permission::ManageMembers.scope(), Scope::CaManagement);
        assert_eq!(Permission::ApproveNews.scope(), Scope::CaManagement);
        assert_eq!(
            Permission::ConfigureElection.scope(),
            Scope::ElectoralCommission
        );
        assert_eq!(
            Permission::AdministerPlatform.scope(),
            Scope::PlatformAdministration
        );
    }
}
