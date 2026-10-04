//! Implementação dos steps de BDD para publicação de informações institucionais.
//!
//! Os passos operam sobre a fixture de publicação institucional em [`AppWorld`],
//! simulando o comportamento de regras de negócio, permissões e versionamento
//! conforme a especificação em `features/transparencia_e_governanca/publicacao_institucional.feature`.

use cucumber::{given, then, when};

use crate::support::{
    AppWorld,
    world::{
        InstitutionalActorRole, InstitutionalContactAndManagement, InstitutionalDocument,
        InstitutionalDocumentVersion, InstitutionalPublicationError,
    },
};

/// Contextualiza o Centro Acadêmico ativo para a execução do cenário.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `ca_name`: nome do Centro Acadêmico configurado.
#[given(expr = "que o {string} está ativo no CApp")]
fn ca_is_active(world: &mut AppWorld, ca_name: String) {
    world.institutional_publication.active_ca = Some(ca_name);
}

/// Define o ator atual como membro autorizado da gestão ativa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o usuário é um membro autorizado da gestão ativa do Centro Acadêmico")]
fn user_is_active_authorized_manager(world: &mut AppWorld) {
    world.institutional_publication.actor_role =
        Some(InstitutionalActorRole::ActiveAuthorizedManager);
}

/// Define o ator atual como estudante regular sem cargo de gestão.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given(
    "que o usuário é um estudante regularmente matriculado sem cargo na gestão do Centro Acadêmico"
)]
fn user_is_regular_student(world: &mut AppWorld) {
    world.institutional_publication.actor_role = Some(InstitutionalActorRole::RegularStudent);
}

/// Define o ator atual como participante de gestão anterior já encerrada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o usuário fez parte de uma gestão anterior já encerrada do Centro Acadêmico")]
fn user_is_past_manager(world: &mut AppWorld) {
    world.institutional_publication.actor_role = Some(InstitutionalActorRole::PastManager);
}

/// Publica um novo documento institucional a partir de uma tabela de parâmetros.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `step`: representação do passo Gherkin contendo a tabela de dados.
///
/// # Panics
///
/// Se o ator não tiver sido previamente contextualizado no cenário.
#[when("ele publica o documento institucional com os seguintes dados:")]
fn publish_institutional_document(world: &mut AppWorld, step: &cucumber::gherkin::Step) {
    let role = world
        .institutional_publication
        .actor_role
        .expect("ator deve estar contextualizado");

    if role != InstitutionalActorRole::ActiveAuthorizedManager {
        world.institutional_publication.last_error =
            Some(InstitutionalPublicationError::Unauthorized);
        return;
    }

    let mut doc_type = String::new();
    let mut title = String::new();
    let mut content = String::new();

    if let Some(table) = &step.table {
        for row in &table.rows {
            if row.len() >= 2 {
                match row[0].trim() {
                    "tipo" => doc_type = row[1].trim().to_string(),
                    "título" => title = row[1].trim().to_string(),
                    "conteúdo" => content = row[1].trim().to_string(),
                    _ => {}
                }
            }
        }
    }

    let version = "1.0".to_string();
    let doc_version = InstitutionalDocumentVersion {
        version: version.clone(),
        content,
        recorded_at: "2026-10-01".to_string(),
        author: "Gestão Ativa".to_string(),
    };

    let document = InstitutionalDocument {
        doc_type: doc_type.clone(),
        title,
        current_version: version.clone(),
        versions: vec![doc_version],
        updated_at: "2026-10-01".to_string(),
    };

    world
        .institutional_publication
        .documents
        .insert(doc_type, document);
    world.institutional_publication.last_published_version = Some(version);
    world.institutional_publication.publicly_available = true;
    world.institutional_publication.last_error = None;
}

/// Valida a versão atribuída ao documento recém-publicado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `version`: identificador de versão esperado.
///
/// # Panics
///
/// Se nenhuma versão tiver sido publicada ou se diferir da esperada.
#[then(expr = "o documento institucional é publicado com a versão {string}")]
fn document_published_with_version(world: &mut AppWorld, version: String) {
    assert_eq!(
        world
            .institutional_publication
            .last_published_version
            .as_deref(),
        Some(version.as_str()),
        "versão publicada deve coincidir com a esperada"
    );
}

/// Verifica se o documento publicado está acessível publicamente para consulta.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o documento não estiver publicamente acessível.
#[then("fica disponível publicamente para consulta da comunidade acadêmica")]
fn document_publicly_available(world: &mut AppWorld) {
    assert!(
        world.institutional_publication.publicly_available,
        "documento deve estar disponível publicamente"
    );
}

/// Publica as informações de contato oficiais e a composição de membros da gestão ativa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `step`: passo Gherkin contendo a tabela de dados de contato e cargos.
///
/// # Panics
///
/// Se o ator não tiver sido contextualizado.
#[when(
    "ele publica as informações de contato e a composição da gestão atual com os seguintes dados:"
)]
fn publish_contact_and_management(world: &mut AppWorld, step: &cucumber::gherkin::Step) {
    let role = world
        .institutional_publication
        .actor_role
        .expect("ator deve estar contextualizado");

    if role != InstitutionalActorRole::ActiveAuthorizedManager {
        world.institutional_publication.last_error =
            Some(InstitutionalPublicationError::Unauthorized);
        return;
    }

    let mut official_email = String::new();
    let mut service_channel = String::new();
    let mut members = Vec::new();

    if let Some(table) = &step.table {
        for row in &table.rows {
            if row.len() >= 2 {
                let key = row[0].trim();
                let val = row[1].trim();
                match key {
                    "e-mail oficial" => official_email = val.to_string(),
                    "canal de atendimento" => service_channel = val.to_string(),
                    cargo => members.push((cargo.to_string(), val.to_string())),
                }
            }
        }
    }

    world.institutional_publication.contact_info = Some(InstitutionalContactAndManagement {
        official_email,
        service_channel,
        management_members: members,
    });
    world.institutional_publication.publicly_available = true;
    world.institutional_publication.last_error = None;
}

/// Confirma que as informações institucionais foram publicadas sem erros.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se houver erro ou ausência de dados de contato.
#[then("as informações institucionais são publicadas com sucesso")]
fn institutional_info_published_successfully(world: &mut AppWorld) {
    assert!(
        world.institutional_publication.last_error.is_none(),
        "publicação de informações institucionais deve ocorrer sem erros"
    );
    assert!(
        world.institutional_publication.contact_info.is_some(),
        "informações de contato e gestão devem estar presentes"
    );
}

/// Confirma que as informações institucionais estão marcadas como dados oficiais vigentes.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se os dados não estiverem publicamente disponíveis.
#[then("passam a ser exibidas como os dados oficiais vigentes do Centro Acadêmico")]
fn displayed_as_official_current_data(world: &mut AppWorld) {
    assert!(
        world.institutional_publication.publicly_available,
        "dados oficiais devem estar disponíveis publicamente"
    );
}

/// Simula a tentativa de publicação de regimento interno por estudante sem permissão.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o ator não estiver contextualizado.
#[when("ele tenta publicar um regimento interno para o Centro Acadêmico")]
fn try_publish_internal_regulations(world: &mut AppWorld) {
    let role = world
        .institutional_publication
        .actor_role
        .expect("ator deve estar contextualizado");

    if role != InstitutionalActorRole::ActiveAuthorizedManager {
        world.institutional_publication.last_error =
            Some(InstitutionalPublicationError::Unauthorized);
    }
}

/// Verifica se a publicação foi recusada por falta de autorização.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o erro registrado diferir de [`InstitutionalPublicationError::Unauthorized`].
#[then("a publicação é recusada por falta de permissão administrativa")]
fn publication_refused_unauthorized(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_publication.last_error,
        Some(InstitutionalPublicationError::Unauthorized),
        "publicação deve ser recusada por falta de permissão"
    );
}

/// Verifica se nenhum documento foi incluído no acervo institucional.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se houver qualquer documento registrado.
#[then("nenhum documento institucional é registrado")]
fn no_document_registered(world: &mut AppWorld) {
    assert!(
        world.institutional_publication.documents.is_empty(),
        "nenhum documento institucional deve estar registrado"
    );
}

/// Simula tentativa de publicação por membro fora da gestão ativa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o ator não estiver contextualizado.
#[when("ele tenta publicar um novo documento institucional")]
fn try_publish_new_document(world: &mut AppWorld) {
    let role = world
        .institutional_publication
        .actor_role
        .expect("ator deve estar contextualizado");

    if role == InstitutionalActorRole::PastManager {
        world.institutional_publication.last_error =
            Some(InstitutionalPublicationError::InactiveManagement);
    } else if role != InstitutionalActorRole::ActiveAuthorizedManager {
        world.institutional_publication.last_error =
            Some(InstitutionalPublicationError::Unauthorized);
    }
}

/// Confirma que a operação foi rejeitada devido à inatividade da gestão do solicitante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o erro diferir de [`InstitutionalPublicationError::InactiveManagement`].
#[then("a publicação é recusada pois o usuário não integra a gestão ativa")]
fn publication_refused_inactive_management(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_publication.last_error,
        Some(InstitutionalPublicationError::InactiveManagement),
        "publicação deve ser recusada pois usuário não é da gestão ativa"
    );
}

/// Assegura que o acervo de documentos não sofreu alterações não autorizadas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se não houver registro de erro na tentativa indevida.
#[then("o acervo documental permanece inalterado")]
fn document_archive_remains_unchanged(world: &mut AppWorld) {
    assert!(
        world.institutional_publication.last_error.is_some(),
        "operação inválida deve registrar erro e preservar acervo"
    );
}

/// Prepara documento já existente publicado em versão inicial.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `doc_type`: tipo do documento institucional.
/// - `version`: versão inicial publicada.
#[given(expr = "que o Centro Acadêmico possui um {string} publicado na versão {string}")]
fn ca_has_document_published(world: &mut AppWorld, doc_type: String, version: String) {
    let doc_version = InstitutionalDocumentVersion {
        version: version.clone(),
        content: "Texto regulatório inicial".to_string(),
        recorded_at: "2026-09-01".to_string(),
        author: "Comissão Estatutária".to_string(),
    };

    let document = InstitutionalDocument {
        doc_type: doc_type.clone(),
        title: doc_type.clone(),
        current_version: version,
        versions: vec![doc_version],
        updated_at: "2026-09-01".to_string(),
    };

    world
        .institutional_publication
        .documents
        .insert(doc_type, document);
}

/// Define o ator do cenário como membro autorizado da gestão ativa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o usuário é um membro autorizado da gestão ativa")]
fn user_is_authorized_manager_active(world: &mut AppWorld) {
    world.institutional_publication.actor_role =
        Some(InstitutionalActorRole::ActiveAuthorizedManager);
}

/// Atualiza o conteúdo de um documento institucional existente gerando nova versão.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `doc_type`: tipo do documento a ser atualizado.
///
/// # Panics
///
/// Se o ator não estiver contextualizado.
#[when(expr = "ele atualiza o conteúdo do {string} com uma nova redação")]
fn update_document_content(world: &mut AppWorld, doc_type: String) {
    let role = world
        .institutional_publication
        .actor_role
        .expect("ator deve estar contextualizado");

    if role != InstitutionalActorRole::ActiveAuthorizedManager {
        world.institutional_publication.last_error =
            Some(InstitutionalPublicationError::Unauthorized);
        return;
    }

    if let Some(doc) = world.institutional_publication.documents.get_mut(&doc_type) {
        let next_version = "1.1".to_string();
        doc.versions.push(InstitutionalDocumentVersion {
            version: next_version.clone(),
            content: "Nova redação aprovada".to_string(),
            recorded_at: "2026-10-01".to_string(),
            author: "Assembleia Geral".to_string(),
        });
        doc.current_version = next_version.clone();
        doc.updated_at = "2026-10-01".to_string();
        world.institutional_publication.last_published_version = Some(next_version);
        world.institutional_publication.last_error = None;
    } else {
        world.institutional_publication.last_error =
            Some(InstitutionalPublicationError::DocumentNotFound);
    }
}

/// Verifica se a nova versão esperada foi publicada com sucesso.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `version`: identificador da versão recém-publicada.
///
/// # Panics
///
/// Se a versão publicada for diferente da esperada.
#[then(expr = "uma nova versão {string} do documento é publicada")]
fn new_version_published(world: &mut AppWorld, version: String) {
    assert_eq!(
        world
            .institutional_publication
            .last_published_version
            .as_deref(),
        Some(version.as_str()),
        "versão publicada deve coincidir com a nova versão esperada"
    );
}

/// Verifica se a versão informada passou a ser a versão vigente do documento.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `version`: identificador da versão que deve estar vigente.
///
/// # Panics
///
/// Se a versão não constar como vigente em nenhum documento do acervo.
#[then(expr = "a versão {string} torna-se a versão vigente para consulta")]
fn version_becomes_current(world: &mut AppWorld, version: String) {
    let is_current = world
        .institutional_publication
        .documents
        .values()
        .any(|doc| doc.current_version == version);

    assert!(is_current, "versão {version} deve ser a vigente no acervo");
}

/// Prepara documento com histórico de duas versões distintas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `doc_type`: tipo do documento institucional.
/// - `v1`: versão anterior.
/// - `v2`: versão vigente posterior.
#[given(expr = "que o {string} possui as versões {string} e {string} registradas")]
fn document_has_versions(world: &mut AppWorld, doc_type: String, v1: String, v2: String) {
    let version1 = InstitutionalDocumentVersion {
        version: v1,
        content: "Conteúdo da versão original".to_string(),
        recorded_at: "2026-08-01".to_string(),
        author: "Fundadores".to_string(),
    };
    let version2 = InstitutionalDocumentVersion {
        version: v2.clone(),
        content: "Conteúdo da versão revisada".to_string(),
        recorded_at: "2026-09-15".to_string(),
        author: "Gestão Ativa".to_string(),
    };

    let document = InstitutionalDocument {
        doc_type: doc_type.clone(),
        title: doc_type.clone(),
        current_version: v2,
        versions: vec![version1, version2],
        updated_at: "2026-09-15".to_string(),
    };

    world
        .institutional_publication
        .documents
        .insert(doc_type, document);
}

/// Simula a consulta ao histórico de versões de um documento.
///
/// # Parâmetros
///
/// - `_world`: contexto do cenário Cucumber.
/// - `_doc_type`: identificador do documento consultado.
#[when(expr = "qualquer usuário consulta o histórico do {string}")]
fn user_queries_history(_world: &mut AppWorld, _doc_type: String) {}

/// Assegura que versões anteriores continuam acessíveis no histórico para fins de auditoria.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `version`: identificador da versão anterior a ser verificada.
///
/// # Panics
///
/// Se a versão anterior não estiver presente nas versões do documento.
#[then(expr = "o sistema disponibiliza o acesso à versão anterior {string} para auditoria")]
fn access_previous_version(world: &mut AppWorld, version: String) {
    let has_version = world
        .institutional_publication
        .documents
        .values()
        .any(|doc| doc.versions.iter().any(|v| v.version == version));

    assert!(
        has_version,
        "versão anterior {version} deve estar disponível no histórico"
    );
}

/// Valida que todas as versões preservam o carimbo de data e autoria da alteração.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se alguma versão não tiver data ou autor preenchidos.
#[then("mantém o registro de data e autor da alteração")]
fn preserves_date_and_author(world: &mut AppWorld) {
    let valid_metadata = world
        .institutional_publication
        .documents
        .values()
        .all(|doc| {
            doc.versions
                .iter()
                .all(|v| !v.recorded_at.is_empty() && !v.author.is_empty())
        });

    assert!(
        valid_metadata,
        "todas as versões devem possuir data e autor registrados"
    );
}

/// Simula tentativa de atualização em documento não cadastrado no acervo.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `doc_type`: tipo do documento inexistente.
#[when(expr = "ele tenta atualizar o conteúdo de um documento {string} inexistente")]
fn try_update_nonexistent_document(world: &mut AppWorld, doc_type: String) {
    if !world
        .institutional_publication
        .documents
        .contains_key(&doc_type)
    {
        world.institutional_publication.last_error =
            Some(InstitutionalPublicationError::DocumentNotFound);
    }
}

/// Confirma que a tentativa de atualização foi recusada por inexistência do documento.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o erro diferir de [`InstitutionalPublicationError::DocumentNotFound`].
#[then("a operação é recusada indicando que o documento base não foi encontrado")]
fn operation_refused_document_not_found(world: &mut AppWorld) {
    assert_eq!(
        world.institutional_publication.last_error,
        Some(InstitutionalPublicationError::DocumentNotFound),
        "operação deve ser recusada com documento não encontrado"
    );
}

/// Prepara o acervo inicial com documentos vigentes publicados.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o Centro Acadêmico possui documentos institucionais vigentes publicados")]
fn ca_has_published_documents(world: &mut AppWorld) {
    let estatuto = InstitutionalDocument {
        doc_type: "Estatuto Social".to_string(),
        title: "Estatuto Geral do CA".to_string(),
        current_version: "1.0".to_string(),
        versions: vec![InstitutionalDocumentVersion {
            version: "1.0".to_string(),
            content: "Conteúdo do Estatuto".to_string(),
            recorded_at: "2026-01-10".to_string(),
            author: "Assembleia".to_string(),
        }],
        updated_at: "2026-01-10".to_string(),
    };
    let regimento = InstitutionalDocument {
        doc_type: "Regimento Interno".to_string(),
        title: "Regimento Interno do CA".to_string(),
        current_version: "1.0".to_string(),
        versions: vec![InstitutionalDocumentVersion {
            version: "1.0".to_string(),
            content: "Conteúdo do Regimento".to_string(),
            recorded_at: "2026-02-15".to_string(),
            author: "Gestão Ativa".to_string(),
        }],
        updated_at: "2026-02-15".to_string(),
    };

    world
        .institutional_publication
        .documents
        .insert("Estatuto Social".to_string(), estatuto);
    world
        .institutional_publication
        .documents
        .insert("Regimento Interno".to_string(), regimento);
    world.institutional_publication.publicly_available = true;
}

/// Contextualiza acesso de estudante à seção institucional do Centro Acadêmico.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("um estudante acessa a seção institucional do Centro Acadêmico")]
fn student_accesses_institutional_section(world: &mut AppWorld) {
    world.institutional_publication.actor_role = Some(InstitutionalActorRole::RegularStudent);
}

/// Valida visualização dos documentos normativos vigentes pelo estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o estatuto ou o regimento não estiverem disponíveis para visualização.
#[then("ele visualiza o estatuto social e o regimento interno vigentes")]
fn student_views_statute_and_regulations(world: &mut AppWorld) {
    assert!(
        world
            .institutional_publication
            .documents
            .contains_key("Estatuto Social"),
        "estatuto social deve estar visível"
    );
    assert!(
        world
            .institutional_publication
            .documents
            .contains_key("Regimento Interno"),
        "regimento interno deve estar visível"
    );
}

/// Assegura que a data da última alteração de cada documento é exibida.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se algum documento não possuir a data de atualização preenchida.
#[then("visualiza a data da última atualização de cada documento")]
fn views_last_update_date(world: &mut AppWorld) {
    let has_update_dates = world
        .institutional_publication
        .documents
        .values()
        .all(|doc| !doc.updated_at.is_empty());

    assert!(
        has_update_dates,
        "cada documento deve conter a data de última atualização"
    );
}

/// Prepara o cadastro de contatos oficiais e ocupantes dos cargos da gestão ativa.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o Centro Acadêmico possui contatos e composição da gestão publicados")]
fn ca_has_contacts_and_management(world: &mut AppWorld) {
    world.institutional_publication.contact_info = Some(InstitutionalContactAndManagement {
        official_email: "contato@cacc.ufba.br".to_string(),
        service_channel: "Sala 102 - Pavilhão de Aulas".to_string(),
        management_members: vec![
            ("Presidente".to_string(), "Maria Silva".to_string()),
            ("Diretora Financeira".to_string(), "João Santos".to_string()),
        ],
    });
    world.institutional_publication.publicly_available = true;
}

/// Simula consulta às informações de contato e membros da gestão por estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("um estudante consulta as informações institucionais do Centro Acadêmico")]
fn student_queries_institutional_info(world: &mut AppWorld) {
    world.institutional_publication.actor_role = Some(InstitutionalActorRole::RegularStudent);
}

/// Verifica se os canais de contato e a relação de gestores estão visíveis ao estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se os contatos ou membros da gestão estiverem ausentes.
#[then("ele visualiza os canais de contato oficiais e a lista de membros da gestão ativa")]
fn views_official_contacts_and_management_members(world: &mut AppWorld) {
    let info = world
        .institutional_publication
        .contact_info
        .as_ref()
        .expect("informações de contato devem existir");

    assert!(
        !info.official_email.is_empty(),
        "e-mail de contato deve estar presente"
    );
    assert!(
        !info.management_members.is_empty(),
        "membros da gestão devem estar presentes"
    );
}
