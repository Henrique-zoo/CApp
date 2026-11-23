-- Migration: 20261122140000_create_faq_entries_table.sql
-- Tabela de domínio para armazenamento de perguntas e respostas frequentes (FAQ) dos Centros Acadêmicos

CREATE TABLE IF NOT EXISTS faq_entry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    academic_center_id UUID NOT NULL REFERENCES academic_center(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    category TEXT NULL,
    created_by UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    updated_by UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_faq_entry_question_not_empty CHECK (length(trim(question)) > 0 AND length(trim(question)) <= 500),
    CONSTRAINT chk_faq_entry_answer_not_empty CHECK (length(trim(answer)) > 0 AND length(trim(answer)) <= 5000),
    CONSTRAINT chk_faq_entry_category_valid CHECK (category IS NULL OR (length(trim(category)) > 0 AND length(trim(category)) <= 100)),
    CONSTRAINT chk_faq_entry_timestamps CHECK (updated_at >= created_at)
);

CREATE INDEX IF NOT EXISTS idx_faq_entry_academic_center_id ON faq_entry (academic_center_id);
CREATE INDEX IF NOT EXISTS idx_faq_entry_created_by ON faq_entry (created_by);
CREATE INDEX IF NOT EXISTS idx_faq_entry_category ON faq_entry (category);
CREATE INDEX IF NOT EXISTS idx_faq_entry_created_at ON faq_entry (created_at DESC);

-- View para compatibilidade caso consultas utilizem o nome no plural
CREATE OR REPLACE VIEW faq_entries AS SELECT * FROM faq_entry;
