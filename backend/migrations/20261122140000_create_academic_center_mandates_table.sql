-- Migration: 20261122140000_create_academic_center_mandates_table.sql
-- Tabela de mandatos e configuração da gestão do Centro Acadêmico (Issue #296)

CREATE TABLE IF NOT EXISTS academic_center_mandate (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    academic_center_id UUID NOT NULL REFERENCES academic_center(id) ON DELETE RESTRICT,
    created_by_id UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    title TEXT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_academic_center_mandate_dates CHECK (end_date > start_date),
    CONSTRAINT chk_academic_center_mandate_status CHECK (status IN ('active', 'concluded', 'archived')),
    CONSTRAINT chk_academic_center_mandate_timestamps CHECK (updated_at >= created_at),
    CONSTRAINT chk_academic_center_mandate_title_not_empty CHECK (title IS NULL OR length(trim(title)) > 0)
);

CREATE INDEX IF NOT EXISTS idx_academic_center_mandate_academic_center_id
    ON academic_center_mandate (academic_center_id);

CREATE INDEX IF NOT EXISTS idx_academic_center_mandate_created_by_id
    ON academic_center_mandate (created_by_id);

CREATE INDEX IF NOT EXISTS idx_academic_center_mandate_status
    ON academic_center_mandate (status);

-- Garante no nível relacional que um Centro Acadêmico possua no máximo um mandato ativo simultaneamente
CREATE UNIQUE INDEX IF NOT EXISTS idx_academic_center_mandate_single_active
    ON academic_center_mandate (academic_center_id)
    WHERE status = 'active';

-- Visão para compatibilidade com consultas e referências no plural
CREATE OR REPLACE VIEW academic_center_mandates AS
    SELECT * FROM academic_center_mandate;
