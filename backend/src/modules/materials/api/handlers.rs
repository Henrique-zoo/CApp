//! Adapta parâmetros e conteúdo HTTP aos casos de uso do módulo.

use axum::{
    Json,
    body::{Bytes, to_bytes},
    extract::{FromRequest, Multipart, Path, Query, Request, State, multipart::Field},
    http::{
        HeaderMap, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE, LOCATION},
    },
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    api::{
        error::{ApiError, ApiResult},
        state::AppState,
    },
    modules::{
        identity::api::extractors::AuthenticatedAccount,
        materials::application::{
            ports::MaterialAccount,
            services::{self, MAX_PDF_BYTES, MaterialError, MaterialInput, MaterialUpload},
        },
    },
};

use super::dto::{
    CenterDto, DisciplineDto, LinkRequestDto, ListQuery, MaterialDto, MaterialPage, MetadataDto,
};

pub(super) const METADATA_LIMIT: usize = 96 * 1024;

pub(super) async fn centers(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
) -> ApiResult<Json<Vec<CenterDto>>> {
    let centers = services::centers(&state.pg_database, actor(account)).await?;
    Ok(Json(
        centers
            .into_iter()
            .map(|center| CenterDto::new(center, state.material_storage.is_some()))
            .collect(),
    ))
}

pub(super) async fn disciplines(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    path: Result<Path<Uuid>, axum::extract::rejection::PathRejection>,
) -> ApiResult<Json<Vec<DisciplineDto>>> {
    let Path(ca_id) = path.map_err(|_| ApiError::bad_request())?;
    Ok(Json(
        services::disciplines(&state.pg_database, actor(account), ca_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

pub(super) async fn list(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    path: Result<Path<Uuid>, axum::extract::rejection::PathRejection>,
    query: Result<Query<ListQuery>, axum::extract::rejection::QueryRejection>,
) -> ApiResult<Json<MaterialPage>> {
    let Path(ca_id) = path.map_err(|_| ApiError::bad_request())?;
    let Query(query) = query.map_err(|_| ApiError::bad_request())?;
    let page = services::list(
        &state.pg_database,
        actor(account),
        ca_id,
        query.discipline_id,
        query.limit.unwrap_or(30),
        query.offset,
    )
    .await?;
    Ok(Json(MaterialPage {
        items: page.items.into_iter().map(Into::into).collect(),
        next_offset: page.next_offset,
    }))
}

pub(super) async fn get(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    path: Result<Path<(Uuid, Uuid)>, axum::extract::rejection::PathRejection>,
) -> ApiResult<Json<MaterialDto>> {
    let Path((ca_id, id)) = path.map_err(|_| ApiError::bad_request())?;
    Ok(Json(
        services::details(&state.pg_database, actor(account), ca_id, id)
            .await?
            .into(),
    ))
}

pub(super) async fn download(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    path: Result<Path<(Uuid, Uuid)>, axum::extract::rejection::PathRejection>,
) -> ApiResult<impl IntoResponse> {
    let Path((ca_id, id)) = path.map_err(|_| ApiError::bad_request())?;
    let (_, bytes) = services::download(
        &state.pg_database,
        state.material_storage.as_deref(),
        actor(account),
        ca_id,
        id,
    )
    .await?;
    Ok((
        [
            (CONTENT_TYPE, "application/pdf"),
            (CONTENT_DISPOSITION, "attachment; filename=\"material.pdf\""),
            (axum::http::header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        Bytes::from(bytes),
    ))
}

pub(super) async fn publish(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    path: Result<Path<Uuid>, axum::extract::rejection::PathRejection>,
    request: Request,
) -> ApiResult<impl IntoResponse> {
    let Path(ca_id) = path.map_err(|_| ApiError::bad_request())?;
    let account = actor(account);
    // Verifique a permissão antes de receber o corpo de um upload.
    services::authorize_publication(&state.pg_database, account, ca_id).await?;
    let request_id = idempotency_key(request.headers())?;
    let (input, upload) = match media_type(request.headers()) {
        "application/json" => {
            let bytes = to_bytes(request.into_body(), METADATA_LIMIT)
                .await
                .map_err(|_| ApiError::bad_request())?;
            serde_json::from_slice::<LinkRequestDto>(&bytes)
                .map_err(|_| ApiError::bad_request())?
                .into()
        }
        "multipart/form-data" => {
            let form = Multipart::from_request(request, &state)
                .await
                .map_err(|_| ApiError::bad_request())?;
            parse_pdf(form).await?
        }
        _ => {
            return Err(error(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_material_media_type",
                "Use application/json ou multipart/form-data.",
            ));
        }
    };
    let result = services::publish(
        &state.pg_database,
        state.material_storage.as_deref(),
        account,
        ca_id,
        request_id,
        input,
        upload,
    )
    .await?;
    let location = format!(
        "/academic-centers/{ca_id}/materials/{}",
        result.value.material.id()
    );
    let status = if result.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((
        status,
        [(LOCATION, location)],
        Json(MaterialDto::from(result.value)),
    ))
}

fn actor(account: AuthenticatedAccount) -> MaterialAccount {
    MaterialAccount {
        id: account.account_id,
        microsoft_object_id: account.microsoft_object_id,
    }
}

fn media_type(headers: &HeaderMap) -> &str {
    headers
        .get(CONTENT_TYPE)
        .and_then(|header| header.to_str().ok())
        .map_or("", |value| {
            value.split(';').next().unwrap_or_default().trim()
        })
}

fn idempotency_key(headers: &HeaderMap) -> ApiResult<Uuid> {
    let mut keys = headers.get_all("idempotency-key").iter();
    let key = keys
        .next()
        .and_then(|key| key.to_str().ok())
        .and_then(|key| Uuid::parse_str(key).ok());
    match key {
        Some(key) if keys.next().is_none() => Ok(key),
        _ => Err(error(
            StatusCode::BAD_REQUEST,
            "invalid_material_request_id",
            "Informe um Idempotency-Key válido.",
        )),
    }
}

async fn parse_pdf(mut form: Multipart) -> ApiResult<(MaterialInput, MaterialUpload)> {
    let mut input = None;
    let mut pdf = None;
    while let Some(field) = form.next_field().await.map_err(multipart_error)? {
        match field.name() {
            Some("metadata") if input.is_none() => {
                let bytes = read_field(field, METADATA_LIMIT, MaterialError::Invalid).await?;
                input = Some(
                    serde_json::from_slice::<MetadataDto>(&bytes)
                        .map_err(|_| ApiError::bad_request())?
                        .into(),
                );
            }
            Some("file") if pdf.is_none() => {
                let name = field
                    .file_name()
                    .ok_or(MaterialError::InvalidPdf)?
                    .to_owned();
                if !field
                    .content_type()
                    .is_some_and(|mime| mime.eq_ignore_ascii_case("application/pdf"))
                {
                    return Err(MaterialError::InvalidPdf.into());
                }
                let bytes = read_field(field, MAX_PDF_BYTES, MaterialError::TooLarge).await?;
                pdf = Some(MaterialUpload::Pdf(name, bytes));
            }
            _ => return Err(ApiError::bad_request()),
        }
    }
    Ok((
        input.ok_or_else(ApiError::bad_request)?,
        pdf.ok_or_else(ApiError::bad_request)?,
    ))
}

async fn read_field(
    mut field: Field<'_>,
    limit: usize,
    exceeded: MaterialError,
) -> ApiResult<Vec<u8>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field.chunk().await.map_err(multipart_error)? {
        if chunk.len() > limit - bytes.len() {
            return Err(exceeded.into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn multipart_error(error: axum::extract::multipart::MultipartError) -> ApiError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        MaterialError::TooLarge.into()
    } else {
        ApiError::bad_request()
    }
}

fn error(status: StatusCode, code: &'static str, message: &str) -> ApiError {
    ApiError::new(status, code, message, None)
}

impl From<MaterialError> for ApiError {
    fn from(value: MaterialError) -> Self {
        use MaterialError::*;
        let (status, code) = match value {
            Invalid => (StatusCode::BAD_REQUEST, "invalid_material"),
            InvalidLink => (StatusCode::BAD_REQUEST, "invalid_material_link"),
            InvalidDiscipline => (StatusCode::BAD_REQUEST, "invalid_material_discipline"),
            InvalidPdf => (StatusCode::BAD_REQUEST, "invalid_material_pdf"),
            TooLarge => (StatusCode::PAYLOAD_TOO_LARGE, "material_pdf_too_large"),
            Forbidden => (StatusCode::FORBIDDEN, "material_access_denied"),
            NotFound => (StatusCode::NOT_FOUND, "material_not_found"),
            Conflict => (StatusCode::CONFLICT, "material_request_conflict"),
            StorageUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "material_storage_unavailable",
            ),
            Persistence => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "material_persistence_failure",
            ),
        };
        error(status, code, &value.to_string())
    }
}
