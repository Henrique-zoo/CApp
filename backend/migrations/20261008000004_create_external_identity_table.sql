-- Migration: 20261008000004_create_external_identity_table.sql
-- Tabela de identidades externas vinculadas aos usuários conforme modelo ER oficial (§ 1. Identidade e vínculo acadêmico)

CREATE TABLE IF NOT EXISTS external_identity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    institution_id UUID NULL,
    firebase_uid TEXT NOT NULL,
    microsoft_subject TEXT NOT NULL,
    last_login_at TIMESTAMPTZ NULL,
    CONSTRAINT fk_external_identity_user FOREIGN KEY (user_id) REFERENCES app_user(id) ON DELETE CASCADE,
    CONSTRAINT uk_external_identity_firebase_uid UNIQUE (firebase_uid),
    CONSTRAINT chk_external_identity_firebase_uid_not_empty CHECK (length(trim(firebase_uid)) > 0),
    CONSTRAINT chk_external_identity_microsoft_subject_not_empty CHECK (length(trim(microsoft_subject)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_external_identity_subject_institution ON external_identity (institution_id, microsoft_subject) WHERE institution_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uk_external_identity_subject_no_inst ON external_identity (microsoft_subject) WHERE institution_id IS NULL;
CREATE INDEX IF NOT EXISTS idx_external_identity_user_id ON external_identity (user_id);
CREATE INDEX IF NOT EXISTS idx_external_identity_firebase_uid ON external_identity (firebase_uid);
CREATE INDEX IF NOT EXISTS idx_external_identity_microsoft_subject ON external_identity (microsoft_subject);
