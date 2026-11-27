//! Contrato JSON de entrada e saída da API de materiais.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::materials::application::{
    ports::{MaterialCenter, MaterialDiscipline, PublishedMaterial},
    services::{MaterialInput, MaterialUpload},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct MetadataDto {
    title: String,
    description: String,
    discipline_id: Uuid,
}

impl From<MetadataDto> for MaterialInput {
    fn from(value: MetadataDto) -> Self {
        Self {
            title: value.title,
            description: value.description,
            discipline_id: value.discipline_id,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct LinkRequestDto {
    title: String,
    description: String,
    discipline_id: Uuid,
    content: LinkDto,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum LinkDto {
    ExternalLink { url: String },
}

impl From<LinkRequestDto> for (MaterialInput, MaterialUpload) {
    fn from(value: LinkRequestDto) -> Self {
        let LinkDto::ExternalLink { url } = value.content;
        (
            MaterialInput {
                title: value.title,
                description: value.description,
                discipline_id: value.discipline_id,
            },
            MaterialUpload::Link(url),
        )
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CenterDto {
    id: Uuid,
    name: String,
    can_publish: bool,
    can_upload_pdf: bool,
}

impl CenterDto {
    pub(super) fn new(center: MaterialCenter, has_storage: bool) -> Self {
        Self {
            id: center.id,
            name: center.name,
            can_publish: center.can_publish,
            can_upload_pdf: center.can_publish && has_storage,
        }
    }
}

#[derive(Serialize)]
pub(super) struct DisciplineDto {
    id: Uuid,
    name: String,
}

impl From<MaterialDiscipline> for DisciplineDto {
    fn from(value: MaterialDiscipline) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MaterialDto {
    id: Uuid,
    ca_id: Uuid,
    discipline_id: Uuid,
    title: String,
    description: String,
    created_at: DateTime<Utc>,
    content: ContentDto,
}

#[derive(Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum ContentDto {
    ExternalLink { url: String },
    File { file_name: String, size_bytes: i32 },
}

impl From<PublishedMaterial> for MaterialDto {
    fn from(published: PublishedMaterial) -> Self {
        let material = published.material;
        let content = match published.file {
            Some(file) => ContentDto::File {
                file_name: file.name,
                size_bytes: file.size,
            },
            None => ContentDto::ExternalLink {
                url: material
                    .content()
                    .external_url()
                    .expect("publicação com link ou arquivo")
                    .to_owned(),
            },
        };
        Self {
            id: material.id(),
            ca_id: material.ca_id(),
            discipline_id: material.discipline_id(),
            title: material.title().to_owned(),
            description: material.description().to_owned(),
            created_at: material.created_at(),
            content,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ListQuery {
    pub(super) discipline_id: Option<Uuid>,
    pub(super) limit: Option<u32>,
    #[serde(default)]
    pub(super) offset: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MaterialPage {
    pub(super) items: Vec<MaterialDto>,
    pub(super) next_offset: Option<u32>,
}
