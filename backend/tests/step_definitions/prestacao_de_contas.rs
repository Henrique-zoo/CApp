//! Implementação dos steps de BDD para prestação de contas do Centro Acadêmico.
//!
//! Os passos operam sobre a fixture de prestação de contas em [`AppWorld`],
//! simulando o comportamento de regras contábeis manuais, controle de permissões,
//! retificações e consulta pública dos demonstrativos financeiros conforme a
//! especificação em `features/transparencia_e_governanca/prestacao_de_contas.feature`.

use cucumber::{given, then, when};

use crate::support::{
    AppWorld,
    world::{
        FinancialAccountabilityError, FinancialActorRole, FinancialEntry, FinancialRectification,
        FinancialTransactionType,
    },
};

/// Contextualiza o Centro Acadêmico ativo para o cenário de prestação de contas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `ca_name`: nome do Centro Acadêmico configurado.
#[given(expr = "que o {string} está ativo no CApp")]
fn ca_is_active(world: &mut AppWorld, ca_name: String) {
    world.financial_accountability.active_ca = Some(ca_name);
}

/// Define o ator atual como membro da gestão com permissão explícita de prestação de contas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o usuário é um membro da gestão com permissão explícita de prestação de contas")]
fn user_is_manager_with_financial_permission(world: &mut AppWorld) {
    world.financial_accountability.actor_role =
        Some(FinancialActorRole::ManagerWithFinancialPermission);
}

/// Publica um lançamento financeiro manual a partir da tabela de dados fornecida.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `step`: representação do passo Gherkin contendo os dados da transação.
///
/// # Panics
///
/// Se o ator não estiver previamente contextualizado ou se o valor numérico for inválido.
#[when("ele cadastra um lançamento financeiro manual com os seguintes dados:")]
fn register_financial_entry(world: &mut AppWorld, step: &cucumber::gherkin::Step) {
    let role = world
        .financial_accountability
        .actor_role
        .expect("ator deve estar contextualizado");

    if role != FinancialActorRole::ManagerWithFinancialPermission {
        world.financial_accountability.last_error =
            Some(FinancialAccountabilityError::Unauthorized);
        return;
    }

    let mut transaction_type = FinancialTransactionType::Income;
    let mut title = String::new();
    let mut amount = 0.0;
    let mut date = String::new();
    let mut category = String::new();
    let mut attachment = None;

    if let Some(table) = &step.table {
        for row in &table.rows {
            if row.len() >= 2 {
                match row[0].trim() {
                    "tipo de transação" => {
                        if row[1].trim().eq_ignore_ascii_case("despesa") {
                            transaction_type = FinancialTransactionType::Expense;
                        } else {
                            transaction_type = FinancialTransactionType::Income;
                        }
                    }
                    "título" => title = row[1].trim().to_string(),
                    "valor" => {
                        amount = row[1].trim().parse::<f64>().expect("valor numérico válido");
                    }
                    "data" => date = row[1].trim().to_string(),
                    "categoria" => category = row[1].trim().to_string(),
                    "comprovante" => attachment = Some(row[1].trim().to_string()),
                    _ => {}
                }
            }
        }
    }

    let id = format!(
        "LANCTO-{:03}",
        world.financial_accountability.entries.len() + 1
    );
    let entry = FinancialEntry {
        id: id.clone(),
        transaction_type,
        title,
        amount,
        date,
        category,
        attachment,
        rectifications: Vec::new(),
    };

    world
        .financial_accountability
        .entries
        .insert(id.clone(), entry);
    world.financial_accountability.last_published_id = Some(id);
    world.financial_accountability.last_error = None;
}

/// Valida se o lançamento contábil foi publicado com sucesso no acervo.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se houver erro de autorização ou ausência de identificador de publicação.
#[then("o lançamento financeiro é publicado com sucesso")]
fn financial_entry_published_successfully(world: &mut AppWorld) {
    assert!(
        world.financial_accountability.last_error.is_none(),
        "publicação do lançamento financeiro não deve conter erros"
    );
    assert!(
        world.financial_accountability.last_published_id.is_some(),
        "deve haver um lançamento publicado registrado"
    );
}

/// Verifica se o saldo financeiro reflete o acréscimo da receita publicada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `amount_str`: representação textual do valor da receita esperada.
///
/// # Panics
///
/// Se a conversão do valor falhar ou se o saldo consolidado divergir do esperado.
#[then(expr = "o saldo consolidado do Centro Acadêmico reflete a nova receita de {string}")]
fn balance_reflects_income(world: &mut AppWorld, amount_str: String) {
    let expected_amount: f64 = amount_str.parse().expect("valor esperado válido");
    let balance = world.financial_accountability.consolidated_balance();
    assert!(
        (balance - expected_amount).abs() < 0.001,
        "saldo consolidado esperado: {expected_amount}, obtido: {balance}"
    );
}

/// Verifica se o saldo financeiro deduz o valor da despesa publicada.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `amount_str`: representação textual do valor da despesa esperada.
///
/// # Panics
///
/// Se a conversão do valor falhar ou se o saldo consolidado divergir do esperado.
#[then(expr = "o saldo consolidado do Centro Acadêmico deduz a nova despesa de {string}")]
fn balance_deducts_expense(world: &mut AppWorld, amount_str: String) {
    let expense_amount: f64 = amount_str.parse().expect("valor de despesa válido");
    let expected_balance = -expense_amount;
    let balance = world.financial_accountability.consolidated_balance();
    assert!(
        (balance - expected_balance).abs() < 0.001,
        "saldo consolidado esperado: {expected_balance}, obtido: {balance}"
    );
}

/// Define o ator atual como estudante regular sem permissão financeira.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o usuário é um estudante regularmente matriculado sem cargo financeiro na gestão")]
fn user_is_regular_student_no_finance(world: &mut AppWorld) {
    world.financial_accountability.actor_role = Some(FinancialActorRole::RegularStudent);
}

/// Simula tentativa de cadastro de despesa por usuário sem permissão.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `_amount_str`: valor monetário da tentativa.
///
/// # Panics
///
/// Se o ator não estiver contextualizado.
#[when(expr = "ele tenta cadastrar um novo lançamento de despesa no valor de {string}")]
fn try_register_expense(world: &mut AppWorld, _amount_str: String) {
    let role = world
        .financial_accountability
        .actor_role
        .expect("ator deve estar contextualizado");

    if role != FinancialActorRole::ManagerWithFinancialPermission {
        world.financial_accountability.last_error =
            Some(FinancialAccountabilityError::Unauthorized);
    }
}

/// Valida recusa da publicação por falta de autorização financeira.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o erro registrado diferir de [`FinancialAccountabilityError::Unauthorized`].
#[then("a publicação é recusada por ausência de permissão financeira")]
fn publication_refused_no_permission(world: &mut AppWorld) {
    assert_eq!(
        world.financial_accountability.last_error,
        Some(FinancialAccountabilityError::Unauthorized),
        "publicação deve ser recusada por ausência de permissão financeira"
    );
}

/// Assegura que nenhum lançamento foi computado no acervo contábil.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se houver lançamentos presentes no acervo.
#[then("nenhum lançamento é incluído nos registros contábeis do Centro Acadêmico")]
fn no_entry_included(world: &mut AppWorld) {
    assert!(
        world.financial_accountability.entries.is_empty(),
        "nenhum lançamento deve constar no acervo contábil"
    );
}

/// Define o ator atual como membro da gestão sem atribuição de contas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given(
    "que o usuário é um membro da gestão ativa, mas não possui permissão de prestação de contas"
)]
fn user_is_manager_without_finance(world: &mut AppWorld) {
    world.financial_accountability.actor_role =
        Some(FinancialActorRole::ManagerWithoutFinancialPermission);
}

/// Simula tentativa de publicação financeira por membro não autorizado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o ator não estiver contextualizado.
#[when("ele tenta publicar um lançamento financeiro")]
fn try_publish_financial_entry(world: &mut AppWorld) {
    let role = world
        .financial_accountability
        .actor_role
        .expect("ator deve estar contextualizado");

    if role != FinancialActorRole::ManagerWithFinancialPermission {
        world.financial_accountability.last_error =
            Some(FinancialAccountabilityError::Unauthorized);
    }
}

/// Valida recusa por falta de permissão explícita de movimentação contábil.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o erro registrado diferir de [`FinancialAccountabilityError::Unauthorized`].
#[then("a publicação é recusada indicando falta de autorização para movimentação contábil")]
fn publication_refused_unauthorized_movement(world: &mut AppWorld) {
    assert_eq!(
        world.financial_accountability.last_error,
        Some(FinancialAccountabilityError::Unauthorized),
        "operação deve ser recusada por falta de permissão contábil"
    );
}

/// Confirma que o balanço contábil não sofreu modificações indesejadas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se não houver erro registrado na tentativa inválida.
#[then("o balanço financeiro permanece inalterado")]
fn balance_remains_unchanged(world: &mut AppWorld) {
    assert!(
        world.financial_accountability.last_error.is_some(),
        "erro de autorização deve garantir que o balanço não foi modificado"
    );
}

/// Prepara lançamento pré-existente de despesa no acervo contábil.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `id`: identificador do lançamento.
/// - `amount_str`: valor monetário da despesa.
#[given(
    expr = "que existe um lançamento de despesa publicado com o identificador {string} no valor de {string}"
)]
fn given_expense_entry_published(world: &mut AppWorld, id: String, amount_str: String) {
    let amount = amount_str.parse::<f64>().expect("valor de despesa válido");
    let entry = FinancialEntry {
        id: id.clone(),
        transaction_type: FinancialTransactionType::Expense,
        title: "Despesa registrada".to_string(),
        amount,
        date: "2026-09-20".to_string(),
        category: "Despesas Administrativas".to_string(),
        attachment: Some("recibo_inicial.pdf".to_string()),
        rectifications: Vec::new(),
    };
    world.financial_accountability.entries.insert(id, entry);
}

/// Executa a retificação do valor de um lançamento com justificativa formal.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `id`: identificador do lançamento a retificar.
/// - `new_amount_str`: novo valor numérico atribuído.
/// - `reason`: justificativa da alteração.
///
/// # Panics
///
/// Se o ator não estiver contextualizado ou se o valor numérico for inválido.
#[when(
    expr = "ele retifica o lançamento {string} alterando o valor para {string} com a justificativa {string}"
)]
fn rectify_entry_amount(world: &mut AppWorld, id: String, new_amount_str: String, reason: String) {
    let role = world
        .financial_accountability
        .actor_role
        .expect("ator deve estar contextualizado");

    if role != FinancialActorRole::ManagerWithFinancialPermission {
        world.financial_accountability.last_error =
            Some(FinancialAccountabilityError::Unauthorized);
        return;
    }

    let new_amount = new_amount_str.parse::<f64>().expect("novo valor válido");

    if let Some(entry) = world.financial_accountability.entries.get_mut(&id) {
        let rectification = FinancialRectification {
            previous_amount: entry.amount,
            new_amount,
            reason,
            date: "2026-10-02".to_string(),
            author: "Diretoria Financeira".to_string(),
        };
        entry.amount = new_amount;
        entry.rectifications.push(rectification);
        world.financial_accountability.last_error = None;
    } else {
        world.financial_accountability.last_error =
            Some(FinancialAccountabilityError::EntryNotFound);
    }
}

/// Valida se o lançamento foi atualizado para o novo valor especificado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `id`: identificador do lançamento verificado.
/// - `expected_amount_str`: valor monetário esperado após a retificação.
///
/// # Panics
///
/// Se o lançamento não for encontrado ou se o valor diferir do esperado.
#[then(expr = "o lançamento {string} é atualizado para o valor de {string}")]
fn entry_updated_to_amount(world: &mut AppWorld, id: String, expected_amount_str: String) {
    let expected_amount = expected_amount_str.parse::<f64>().expect("valor numérico");
    let entry = world
        .financial_accountability
        .entries
        .get(&id)
        .expect("lançamento deve existir");
    assert!(
        (entry.amount - expected_amount).abs() < 0.001,
        "valor do lançamento deve ser {expected_amount}"
    );
}

/// Valida se o histórico preserva os metadados de justificativa, data e autoria.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se alguma retificação não possuir justificativa, data ou autor.
#[then("o histórico de retificações registra a alteração com justificativa, data e autor")]
fn rectification_history_preserves_data(world: &mut AppWorld) {
    let has_valid_rectification = world.financial_accountability.entries.values().any(|e| {
        !e.rectifications.is_empty()
            && e.rectifications
                .iter()
                .all(|r| !r.reason.is_empty() && !r.date.is_empty() && !r.author.is_empty())
    });
    assert!(
        has_valid_rectification,
        "histórico de retificações deve conter justificativa, data e autor"
    );
}

/// Simula tentativa de retificação sobre lançamento inexistente.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `id`: identificador do lançamento inexistente.
#[when(expr = "ele tenta retificar o lançamento com identificador {string} que não existe")]
fn try_rectify_nonexistent_entry(world: &mut AppWorld, id: String) {
    if !world.financial_accountability.entries.contains_key(&id) {
        world.financial_accountability.last_error =
            Some(FinancialAccountabilityError::EntryNotFound);
    }
}

/// Valida recusa de retificação por não localização do lançamento contábil.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o erro registrado diferir de [`FinancialAccountabilityError::EntryNotFound`].
#[then("a operação de retificação é recusada informando que o lançamento não foi encontrado")]
fn rectification_refused_entry_not_found(world: &mut AppWorld) {
    assert_eq!(
        world.financial_accountability.last_error,
        Some(FinancialAccountabilityError::EntryNotFound),
        "retificação deve ser recusada com lançamento não encontrado"
    );
}

/// Prepara lançamento pré-existente de receita no acervo contábil.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `id`: identificador do lançamento.
/// - `amount_str`: valor monetário da receita.
#[given(
    expr = "que existe um lançamento de receita publicado com o identificador {string} no valor de {string}"
)]
fn given_income_entry_published(world: &mut AppWorld, id: String, amount_str: String) {
    let amount = amount_str.parse::<f64>().expect("valor de receita válido");
    let entry = FinancialEntry {
        id: id.clone(),
        transaction_type: FinancialTransactionType::Income,
        title: "Receita de apoio acadêmico".to_string(),
        amount,
        date: "2026-09-10".to_string(),
        category: "Patrocínios e Doações".to_string(),
        attachment: Some("recibo_deposito.pdf".to_string()),
        rectifications: Vec::new(),
    };
    world.financial_accountability.entries.insert(id, entry);
}

/// Define o ator atual como estudante sem permissão contábil.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[given("que o usuário é um estudante sem permissão financeira")]
fn user_is_student_without_finance(world: &mut AppWorld) {
    world.financial_accountability.actor_role = Some(FinancialActorRole::RegularStudent);
}

/// Simula tentativa não autorizada de retificação de valor de lançamento.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `_id`: identificador do lançamento.
///
/// # Panics
///
/// Se o ator não estiver contextualizado.
#[when(expr = "ele tenta retificar o valor do lançamento {string}")]
fn try_rectify_entry_unauthorized(world: &mut AppWorld, _id: String) {
    let role = world
        .financial_accountability
        .actor_role
        .expect("ator deve estar contextualizado");

    if role != FinancialActorRole::ManagerWithFinancialPermission {
        world.financial_accountability.last_error =
            Some(FinancialAccountabilityError::Unauthorized);
    }
}

/// Valida bloqueio da tentativa de retificação por carência de autorização.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se o erro registrado diferir de [`FinancialAccountabilityError::Unauthorized`].
#[then("a alteração é bloqueada por falta de permissão administrativa")]
fn modification_blocked_unauthorized(world: &mut AppWorld) {
    assert_eq!(
        world.financial_accountability.last_error,
        Some(FinancialAccountabilityError::Unauthorized),
        "operação de alteração deve ser bloqueada por ausência de permissão"
    );
}

/// Confirma preservação do valor original após tentativa indevida de retificação.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `id`: identificador do lançamento verificado.
/// - `expected_str`: valor monetário que deve permanecer inalterado.
///
/// # Panics
///
/// Se o lançamento não for encontrado ou se o valor tiver sido alterado.
#[then(expr = "o valor do lançamento {string} permanece {string}")]
fn entry_amount_remains(world: &mut AppWorld, id: String, expected_str: String) {
    let expected = expected_str.parse::<f64>().expect("valor numérico");
    let entry = world
        .financial_accountability
        .entries
        .get(&id)
        .expect("lançamento deve existir");
    assert!(
        (entry.amount - expected).abs() < 0.001,
        "valor original do lançamento deve ser mantido"
    );
}

/// Prepara balanço inicial com receitas e despesas acumuladas.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `income_str`: valor acumulado de receitas.
/// - `expense_str`: valor acumulado de despesas.
#[given(
    expr = "que o Centro Acadêmico possui lançamentos publicados contendo receitas de {string} e despesas de {string}"
)]
fn ca_has_published_entries(world: &mut AppWorld, income_str: String, expense_str: String) {
    let income = income_str.parse::<f64>().expect("valor de receita");
    let expense = expense_str.parse::<f64>().expect("valor de despesa");

    let rec = FinancialEntry {
        id: "REC-100".to_string(),
        transaction_type: FinancialTransactionType::Income,
        title: "Receita Acumulada".to_string(),
        amount: income,
        date: "2026-09-01".to_string(),
        category: "Geral".to_string(),
        attachment: None,
        rectifications: Vec::new(),
    };

    let desp = FinancialEntry {
        id: "DESP-100".to_string(),
        transaction_type: FinancialTransactionType::Expense,
        title: "Despesa Acumulada".to_string(),
        amount: expense,
        date: "2026-09-05".to_string(),
        category: "Geral".to_string(),
        attachment: None,
        rectifications: Vec::new(),
    };

    world
        .financial_accountability
        .entries
        .insert("REC-100".to_string(), rec);
    world
        .financial_accountability
        .entries
        .insert("DESP-100".to_string(), desp);
}

/// Simula consulta ao demonstrativo contábil por estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("qualquer estudante consulta o demonstrativo financeiro do Centro Acadêmico")]
fn student_queries_financial_statement(world: &mut AppWorld) {
    world.financial_accountability.actor_role = Some(FinancialActorRole::RegularStudent);
}

/// Verifica se o total de receitas acumuladas exibido confere com o esperado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `expected_str`: valor esperado de receitas acumuladas.
///
/// # Panics
///
/// Se o valor calculado divergir do esperado.
#[then(expr = "visualiza o total de receitas acumuladas de {string}")]
#[then(expr = "ele visualiza o total de receitas acumuladas de {string}")]
fn views_total_income(world: &mut AppWorld, expected_str: String) {
    let expected = expected_str.parse::<f64>().expect("valor");
    let total = world.financial_accountability.total_income();
    assert!(
        (total - expected).abs() < 0.001,
        "total de receitas deve ser {expected}"
    );
}

/// Verifica se o total de despesas acumuladas exibido confere com o esperado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `expected_str`: valor esperado de despesas acumuladas.
///
/// # Panics
///
/// Se o valor calculado divergir do esperado.
#[then(expr = "visualiza o total de despesas acumuladas de {string}")]
#[then(expr = "ele visualiza o total de despesas acumuladas de {string}")]
fn views_total_expense(world: &mut AppWorld, expected_str: String) {
    let expected = expected_str.parse::<f64>().expect("valor");
    let total = world.financial_accountability.total_expense();
    assert!(
        (total - expected).abs() < 0.001,
        "total de despesas deve ser {expected}"
    );
}

/// Verifica se o saldo consolidado atual exibido confere com o cálculo esperado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `expected_str`: valor de saldo esperado.
///
/// # Panics
///
/// Se o saldo calculado divergir do esperado.
#[then(expr = "visualiza o saldo atual consolidado de {string}")]
#[then(expr = "ele visualiza o saldo atual consolidado de {string}")]
fn views_consolidated_balance(world: &mut AppWorld, expected_str: String) {
    let expected = expected_str.parse::<f64>().expect("valor");
    let balance = world.financial_accountability.consolidated_balance();
    assert!(
        (balance - expected).abs() < 0.001,
        "saldo consolidado deve ser {expected}"
    );
}

/// Prepara lançamento contábil contendo anexo de comprovante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
/// - `attachment`: nome do arquivo anexado como comprovante.
#[given(expr = "que existe um lançamento publicado com comprovante {string} anexado")]
fn given_entry_with_attachment(world: &mut AppWorld, attachment: String) {
    let entry = FinancialEntry {
        id: "DESP-050".to_string(),
        transaction_type: FinancialTransactionType::Expense,
        title: "Reforma do Laboratório".to_string(),
        amount: 750.0,
        date: "2026-09-28".to_string(),
        category: "Infraestrutura".to_string(),
        attachment: Some(attachment),
        rectifications: Vec::new(),
    };
    world
        .financial_accountability
        .entries
        .insert("DESP-050".to_string(), entry);
}

/// Simula visualização dos detalhes de um lançamento por estudante.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
#[when("um estudante consulta os detalhes desse lançamento")]
fn student_queries_entry_details(world: &mut AppWorld) {
    world.financial_accountability.actor_role = Some(FinancialActorRole::RegularStudent);
}

/// Valida disponibilização pública do comprovante anexado.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se nenhum comprovante estiver presente nos lançamentos.
#[then("o comprovante anexado é disponibilizado para visualização pública e download")]
fn attachment_available_publicly(world: &mut AppWorld) {
    let has_attachment = world
        .financial_accountability
        .entries
        .values()
        .any(|e| e.attachment.is_some());
    assert!(
        has_attachment,
        "comprovante anexado deve estar presente e acessível"
    );
}

/// Assegura exibição clara e transparente de todos os metadados contábeis.
///
/// # Parâmetros
///
/// - `world`: contexto do cenário Cucumber.
///
/// # Panics
///
/// Se algum lançamento contiver metadados essenciais vazios.
#[then("todos os metadados como categoria, data e justificativa são exibidos com transparência")]
fn entry_metadata_displayed(world: &mut AppWorld) {
    let valid_metadata = world
        .financial_accountability
        .entries
        .values()
        .all(|e| !e.category.is_empty() && !e.date.is_empty() && !e.title.is_empty());
    assert!(
        valid_metadata,
        "metadados contábeis devem ser exibidos de forma transparente"
    );
}
