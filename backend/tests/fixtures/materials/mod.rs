//! Dados reutilizados nas suítes de materiais.

use std::fmt::Write as _;

use backend::modules::materials::application::ports::MaterialAccount;
use sqlx::PgPool;
use uuid::Uuid;

pub struct Fixture {
    pub ca: [Uuid; 2],
    pub discipline: [Uuid; 2],
    pub author: MaterialAccount,
    pub students: [MaterialAccount; 2],
}

impl Fixture {
    pub async fn new(pool: &PgPool, names: [&str; 2], disciplines: [&str; 2]) -> Self {
        let institution = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let group = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO institution (id, name, acronym, microsoft_tenant_id, microsoft_student_group_id, status)
             VALUES ($1, 'Instituição de teste', 'TEST', $2, $3, 'active')",
        ).bind(institution).bind(tenant).bind(group).execute(pool).await.unwrap();

        let ca = [Uuid::new_v4(), Uuid::new_v4()];
        let discipline = [Uuid::new_v4(), Uuid::new_v4()];
        for index in 0..2 {
            sqlx::query(
                "INSERT INTO academic_center (id, institution_id, name) VALUES ($1, $2, $3)",
            )
            .bind(ca[index])
            .bind(institution)
            .bind(names[index])
            .execute(pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO academic_center_discipline (ca_id, id, name) VALUES ($1, $2, $3)",
            )
            .bind(ca[index])
            .bind(discipline[index])
            .bind(disciplines[index])
            .execute(pool)
            .await
            .unwrap();
        }

        let author = account(pool, institution, tenant, group).await;
        let students = [
            account(pool, institution, tenant, group).await,
            account(pool, institution, tenant, group).await,
        ];
        Self {
            ca,
            discipline,
            author,
            students,
        }
    }

    pub async fn grant(&self, pool: &PgPool, ca: Uuid) {
        sqlx::query(
            "INSERT INTO academic_center_material_publisher (ca_id, account_id) VALUES ($1, $2)",
        )
        .bind(ca)
        .bind(self.author.id)
        .execute(pool)
        .await
        .unwrap();
    }
}

async fn account(pool: &PgPool, institution: Uuid, tenant: Uuid, group: Uuid) -> MaterialAccount {
    let id = Uuid::new_v4();
    let object = Uuid::new_v4();
    sqlx::query("INSERT INTO app_user (id, firebase_uid) VALUES ($1, $2)")
        .bind(id)
        .bind(id.to_string())
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO institutional_verification (user_id, microsoft_object_id, institution_id,
            microsoft_tenant_id, microsoft_student_group_id, student_membership_confirmed,
            checked_at, expires_at, course_name)
         VALUES ($1, $2, $3, $4, $5, TRUE, now() - INTERVAL '1 minute', now() + INTERVAL '1 day', 'Engenharia')",
    ).bind(id).bind(object).bind(institution).bind(tenant).bind(group).execute(pool).await.unwrap();
    MaterialAccount {
        id,
        microsoft_object_id: Some(object),
    }
}

pub fn pdf(size: Option<usize>) -> Vec<u8> {
    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for object in [
        "<< /Type /Catalog /Pages 2 0 R >>",
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] >>",
    ] {
        offsets.push(pdf.len());
        writeln!(pdf, "{} 0 obj\n{object}\nendobj", offsets.len()).unwrap();
    }
    let xref = pdf.len();
    pdf.push_str("xref\n0 4\n0000000000 65535 f \n");
    for offset in offsets {
        writeln!(pdf, "{offset:010} 00000 n ").unwrap();
    }
    writeln!(
        pdf,
        "trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF"
    )
    .unwrap();
    let mut bytes = pdf.into_bytes();
    if let Some(size) = size {
        assert!(size >= bytes.len());
        bytes.resize(size, b'\n');
    }
    bytes
}
