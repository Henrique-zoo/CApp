//! Publica links e PDFs e recupera apenas conteúdos publicados no contexto.

use chrono::Utc;
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

use crate::modules::materials::domain::entities::{
    StudyMaterial, StudyMaterialContent, StudyMaterialStatus,
};

use super::ports::{
    MaterialAccount, MaterialCenter, MaterialDiscipline, MaterialFile, MaterialRepository,
    MaterialStorage, Publication, PublishedMaterial,
};

/// Limite de bytes do PDF, antes do encapsulamento multipart.
pub const MAX_PDF_BYTES: usize = 10 * 1024 * 1024;

/// Falhas públicas dos casos de uso.
#[derive(Debug, thiserror::Error)]
pub enum MaterialError {
    /// Entrada inválida.
    #[error("Dados de material inválidos.")]
    Invalid,
    /// URL fora da política de links externos.
    #[error("Endereço externo inválido.")]
    InvalidLink,
    /// Disciplina ausente no catálogo do CA.
    #[error("Disciplina incompatível com o Centro Acadêmico.")]
    InvalidDiscipline,
    /// PDF fora dos formatos aceitos.
    #[error("O arquivo deve ser um PDF válido.")]
    InvalidPdf,
    /// PDF acima do limite.
    #[error("O PDF ultrapassa o limite de 10 MiB.")]
    TooLarge,
    /// Conta sem acesso ou capacidade de publicação.
    #[error("Acesso a materiais não permitido neste Centro Acadêmico.")]
    Forbidden,
    /// Material indisponível no contexto informado.
    #[error("Material publicado não encontrado neste Centro Acadêmico.")]
    NotFound,
    /// Repetição de chave de publicação com outro conteúdo.
    #[error("Esta chave de publicação já foi usada para outro material.")]
    Conflict,
    /// Dependência temporariamente indisponível.
    #[error("Armazenamento de materiais temporariamente indisponível.")]
    StorageUnavailable,
    /// Erro interno de persistência.
    #[error("Ocorreu uma falha interna ao processar o material.")]
    Persistence,
}

/// Metadados enviados pelo autor.
pub struct MaterialInput {
    /// Título da publicação.
    pub title: String,
    /// Descrição exibida nos detalhes.
    pub description: String,
    /// Disciplina selecionada.
    pub discipline_id: Uuid,
}

/// Um dos conteúdos admitidos para publicação.
pub enum MaterialUpload {
    /// Endereço externo HTTP(S).
    Link(String),
    /// Nome de apresentação e corpo do PDF.
    Pdf(String, Vec<u8>),
}

/// Paginação da consulta de materiais publicados.
pub struct PublishedPage {
    /// Materiais visíveis na página.
    pub items: Vec<PublishedMaterial>,
    /// Próxima posição, se houver outra página.
    pub next_offset: Option<u32>,
}

/// Lista os contextos acessíveis para a conta autenticada.
pub async fn centers(
    repository: &impl MaterialRepository,
    account: MaterialAccount,
) -> Result<Vec<MaterialCenter>, MaterialError> {
    repository.centers(account).await
}

/// Lista disciplinas do CA após autorizar a navegação nesse contexto.
pub async fn disciplines(
    repository: &impl MaterialRepository,
    account: MaterialAccount,
    ca_id: Uuid,
) -> Result<Vec<MaterialDiscipline>, MaterialError> {
    repository.center(account, ca_id).await?;
    repository.disciplines(ca_id).await
}

/// Lista somente materiais publicados no CA, com filtro opcional de disciplina.
pub async fn list(
    repository: &impl MaterialRepository,
    account: MaterialAccount,
    ca_id: Uuid,
    discipline_id: Option<Uuid>,
    limit: u32,
    offset: u32,
) -> Result<PublishedPage, MaterialError> {
    if !(1..=100).contains(&limit) {
        return Err(MaterialError::Invalid);
    }
    repository.center(account, ca_id).await?;
    if let Some(id) = discipline_id {
        repository.require_discipline(ca_id, id).await?;
    }
    let mut items = repository
        .list(
            ca_id,
            discipline_id,
            i64::from(limit) + 1,
            i64::from(offset),
        )
        .await?;
    let next_offset = if items.len() > limit as usize {
        items.pop();
        offset.checked_add(limit)
    } else {
        None
    };
    Ok(PublishedPage { items, next_offset })
}

/// Recupera os detalhes de uma publicação no contexto consultado.
pub async fn details(
    repository: &impl MaterialRepository,
    account: MaterialAccount,
    ca_id: Uuid,
    id: Uuid,
) -> Result<PublishedMaterial, MaterialError> {
    repository.center(account, ca_id).await?;
    repository.find(ca_id, id).await
}

/// Verifica a concessão para o CA de destino antes de receber conteúdo.
pub async fn authorize_publication(
    repository: &impl MaterialRepository,
    account: MaterialAccount,
    ca_id: Uuid,
) -> Result<(), MaterialError> {
    if repository.center(account, ca_id).await?.can_publish {
        Ok(())
    } else {
        Err(MaterialError::Forbidden)
    }
}

/// Valida, disponibiliza o conteúdo e publica em nome da conta autenticada.
pub async fn publish(
    repository: &impl MaterialRepository,
    storage: Option<&dyn MaterialStorage>,
    account: MaterialAccount,
    ca_id: Uuid,
    request_id: Uuid,
    input: MaterialInput,
    upload: MaterialUpload,
) -> Result<Publication, MaterialError> {
    authorize_publication(repository, account, ca_id).await?;
    repository
        .require_discipline(ca_id, input.discipline_id)
        .await?;
    let id = Uuid::new_v4();
    let (content, file, bytes) = match upload {
        MaterialUpload::Link(url) => {
            let url = validate_link(&url)?;
            let content =
                StudyMaterialContent::external_link(url).map_err(|_| MaterialError::InvalidLink)?;
            (content, None, None)
        }
        MaterialUpload::Pdf(name, bytes) => {
            let name = name.trim();
            if name.is_empty()
                || name.chars().count() > 255
                || name
                    .chars()
                    .any(|c| c.is_control() || c == '/' || c == '\\')
                || !name.to_ascii_lowercase().ends_with(".pdf")
            {
                return Err(MaterialError::InvalidPdf);
            }
            validate_pdf(&bytes)?;
            let file = MaterialFile {
                name: name.to_owned(),
                size: bytes.len() as i32,
                sha256: Sha256::digest(&bytes).into(),
            };
            (StudyMaterialContent::file(id), Some(file), Some(bytes))
        }
    };

    if input.title.trim().chars().count() > 200
        || input.description.trim().chars().count() > 5000
        || input.title.contains('\0')
        || input.description.contains('\0')
    {
        return Err(MaterialError::Invalid);
    }
    let material = StudyMaterial::new(
        id,
        ca_id,
        input.discipline_id,
        account.id,
        input.title,
        input.description,
        content,
        StudyMaterialStatus::Published,
        Utc::now(),
    )
    .map_err(|_| MaterialError::Invalid)?;

    if let (Some(file), Some(bytes)) = (&file, bytes) {
        storage
            .ok_or(MaterialError::StorageUnavailable)?
            .put(ca_id, file, &bytes)
            .await
            .map_err(storage_error)?;
    }
    repository
        .publish(account, request_id, PublishedMaterial { material, file })
        .await
}

/// Recupera os bytes após verificar contexto e publicação.
pub async fn download(
    repository: &impl MaterialRepository,
    storage: Option<&dyn MaterialStorage>,
    account: MaterialAccount,
    ca_id: Uuid,
    id: Uuid,
) -> Result<(MaterialFile, Vec<u8>), MaterialError> {
    let published = details(repository, account, ca_id, id).await?;
    let file = published.file.ok_or(MaterialError::NotFound)?;
    let bytes = storage
        .ok_or(MaterialError::StorageUnavailable)?
        .read(ca_id, &file)
        .await
        .map_err(storage_error)?;
    Ok((file, bytes))
}

fn storage_error(error: std::io::Error) -> MaterialError {
    tracing::warn!(kind = ?error.kind(), "Falha no armazenamento de PDF");
    MaterialError::StorageUnavailable
}

fn validate_link(value: &str) -> Result<&str, MaterialError> {
    let value = value.trim();
    let (scheme, rest) = value.split_once("://").ok_or(MaterialError::InvalidLink)?;
    if value.chars().count() > 2048
        || value
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
        || !(scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https"))
        || rest.starts_with(['/', '?', '#'])
        || rest
            .split(['/', '?', '#'])
            .next()
            .is_some_and(|host| host.contains('@'))
    {
        return Err(MaterialError::InvalidLink);
    }
    let parsed = Url::parse(value).map_err(|_| MaterialError::InvalidLink)?;
    if parsed.host().is_none() || !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(MaterialError::InvalidLink);
    }
    Ok(value)
}

fn validate_pdf(bytes: &[u8]) -> Result<(), MaterialError> {
    if bytes.len() > MAX_PDF_BYTES {
        return Err(MaterialError::TooLarge);
    }
    // Conferência estrutural limitada aos bytes recebidos, sem extrair objetos
    // comprimidos nem executar o conteúdo do documento.
    let valid_header = bytes.starts_with(b"%PDF-")
        && bytes.get(5..9).is_some_and(|version| {
            matches!(version[0], b'1' | b'2')
                && version[1] == b'.'
                && version[2].is_ascii_digit()
                && matches!(version[3], b'\r' | b'\n')
        });
    let end = bytes.trim_ascii_end().strip_suffix(b"%%EOF");
    let xref = end.and_then(|end| {
        let marker = end.windows(9).rposition(|part| part == b"startxref")?;
        let offset = std::str::from_utf8(end.get(marker + 9..)?.trim_ascii()).ok()?;
        if offset.is_empty() || !offset.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        let offset: usize = offset.parse().ok()?;
        (offset < marker).then(|| &end[offset..marker])
    });
    let valid_xref = xref.is_some_and(|part| {
        if part.starts_with(b"xref\n") || part.starts_with(b"xref\r") {
            return part.windows(7).any(|word| word == b"trailer");
        }
        let dictionary = &part[..part.len().min(4096)];
        let mut words = dictionary
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|word| !word.is_empty());
        words
            .next()
            .is_some_and(|word| word.iter().all(u8::is_ascii_digit))
            && words
                .next()
                .is_some_and(|word| word.iter().all(u8::is_ascii_digit))
            && words.next() == Some(b"obj")
            && dictionary.windows(5).any(|word| word == b"/XRef")
    });
    if valid_header && valid_xref {
        Ok(())
    } else {
        Err(MaterialError::InvalidPdf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pdf_with_size(size: usize) -> Vec<u8> {
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let tail = b"xref\n0 1\n0000000000 65535 f \ntrailer\n<< /Size 1 >>\nstartxref\n9\n%%EOF\n";
        pdf.extend_from_slice(tail);
        pdf.resize(size, b'\n');
        pdf
    }

    #[test]
    fn links_need_an_absolute_http_address() {
        assert_eq!(
            validate_link(" https://exemplo.edu.br/a ").unwrap(),
            "https://exemplo.edu.br/a"
        );
        for link in [
            "",
            "sem-url",
            "https:/exemplo.edu",
            "https:///exemplo.edu",
            "https://a b.edu",
            "https://a@b.edu",
            "file:///etc/passwd",
            "javascript:alert(1)",
        ] {
            assert!(
                matches!(validate_link(link), Err(MaterialError::InvalidLink)),
                "{link}"
            );
        }
    }

    #[test]
    fn validates_pdf_structure_and_inclusive_size_limit() {
        assert!(validate_pdf(&pdf_with_size(MAX_PDF_BYTES)).is_ok());
        assert!(matches!(
            validate_pdf(&pdf_with_size(MAX_PDF_BYTES + 1)),
            Err(MaterialError::TooLarge)
        ));
        assert!(matches!(
            validate_pdf(b"%PDF-1.4\nexecutable\n%%EOF"),
            Err(MaterialError::InvalidPdf)
        ));
    }
}
