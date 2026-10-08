-- Migration: 20261008000001_create_academic_center_table.sql
-- Tabela de domínio para Centros Acadêmicos conforme modelo ER oficial

CREATE TABLE IF NOT EXISTS academic_center (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    institution_id UUID NULL,
    created_by_id UUID NULL,
    name TEXT NOT NULL,
    acronym TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uk_academic_center_acronym UNIQUE (acronym),
    CONSTRAINT chk_academic_center_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT chk_academic_center_acronym_not_empty CHECK (length(trim(acronym)) > 0),
    CONSTRAINT chk_academic_center_status CHECK (status IN ('draft', 'active', 'archived')),
    CONSTRAINT chk_academic_center_timestamps CHECK (updated_at >= created_at)
);

CREATE INDEX IF NOT EXISTS idx_academic_center_status ON academic_center (status);
CREATE INDEX IF NOT EXISTS idx_academic_center_acronym ON academic_center (acronym);
