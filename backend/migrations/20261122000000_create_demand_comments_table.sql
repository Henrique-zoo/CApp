-- Migration: 20261122000000_create_demand_comments_table.sql
-- Tabela de comentários e interações em demandas acadêmicas e de espaço físico.

CREATE TABLE IF NOT EXISTS demand_comment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    demand_id UUID NOT NULL REFERENCES demand(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    author_role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_demand_comment_author_role CHECK (author_role IN ('student', 'ca_management')),
    CONSTRAINT chk_demand_comment_content_not_empty CHECK (length(trim(content)) > 0),
    CONSTRAINT chk_demand_comment_content_max_length CHECK (length(content) <= 2000)
);

CREATE INDEX IF NOT EXISTS idx_demand_comment_demand_id ON demand_comment (demand_id);
CREATE INDEX IF NOT EXISTS idx_demand_comment_author_id ON demand_comment (author_id);
CREATE INDEX IF NOT EXISTS idx_demand_comment_created_at ON demand_comment (demand_id, created_at ASC, id ASC);
