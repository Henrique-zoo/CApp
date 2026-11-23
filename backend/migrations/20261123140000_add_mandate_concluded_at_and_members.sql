-- Migration: 20261123140000_add_mandate_concluded_at_and_members.sql
-- Adiciona concluded_at e tabela de integrantes para preservação de histórico de gestões (Issue #298)

ALTER TABLE academic_center_mandate
    ADD COLUMN IF NOT EXISTS concluded_at TIMESTAMPTZ NULL;

ALTER TABLE academic_center_mandate
    DROP CONSTRAINT IF EXISTS chk_academic_center_mandate_concluded_at;

ALTER TABLE academic_center_mandate
    ADD CONSTRAINT chk_academic_center_mandate_concluded_at
    CHECK (concluded_at IS NULL OR concluded_at >= created_at);

-- Recria visão para incluir a nova coluna
CREATE OR REPLACE VIEW academic_center_mandates AS
    SELECT * FROM academic_center_mandate;

-- Tabela de integrantes da gestão/mandato
CREATE TABLE IF NOT EXISTS academic_center_mandate_member (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mandate_id UUID NOT NULL REFERENCES academic_center_mandate(id) ON DELETE CASCADE,
    user_id UUID NULL REFERENCES app_user(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_mandate_member_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT chk_mandate_member_role_not_empty CHECK (length(trim(role)) > 0),
    CONSTRAINT chk_mandate_member_left_at CHECK (left_at IS NULL OR left_at >= joined_at),
    CONSTRAINT chk_mandate_member_timestamps CHECK (updated_at >= created_at)
);

CREATE INDEX IF NOT EXISTS idx_academic_center_mandate_member_mandate_id
    ON academic_center_mandate_member (mandate_id);

CREATE INDEX IF NOT EXISTS idx_academic_center_mandate_member_user_id
    ON academic_center_mandate_member (user_id);
