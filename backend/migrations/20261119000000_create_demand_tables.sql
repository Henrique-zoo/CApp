-- Migration: 20261119000000_create_demand_tables.sql
-- Tabelas de domínio para demandas acadêmicas e histórico de status.

CREATE TABLE IF NOT EXISTS demand (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    academic_center_id UUID NOT NULL REFERENCES academic_center(id) ON DELETE RESTRICT,
    author_id UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    forum TEXT NOT NULL,
    title TEXT NULL,
    description TEXT NOT NULL,
    category TEXT NULL,
    course_name TEXT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_demand_forum CHECK (forum IN ('institutional', 'physical_space')),
    CONSTRAINT chk_demand_status CHECK (status IN ('open', 'in_analysis', 'forwarded_to_coordination', 'completed')),
    CONSTRAINT chk_demand_description_not_empty CHECK (length(trim(description)) > 0),
    CONSTRAINT chk_demand_title_not_empty CHECK (title IS NULL OR length(trim(title)) > 0),
    CONSTRAINT chk_demand_category_not_empty CHECK (category IS NULL OR length(trim(category)) > 0),
    CONSTRAINT chk_demand_course_not_empty CHECK (course_name IS NULL OR length(trim(course_name)) > 0),
    CONSTRAINT chk_demand_timestamps CHECK (updated_at >= created_at),
    CONSTRAINT chk_demand_physical_space_no_coordination CHECK (
        forum <> 'physical_space' OR status <> 'forwarded_to_coordination'
    ),
    CONSTRAINT chk_demand_physical_space_no_course CHECK (
        forum <> 'physical_space' OR course_name IS NULL
    )
);

CREATE INDEX IF NOT EXISTS idx_demand_academic_center_id ON demand (academic_center_id);
CREATE INDEX IF NOT EXISTS idx_demand_author_id ON demand (author_id);
CREATE INDEX IF NOT EXISTS idx_demand_forum ON demand (forum);
CREATE INDEX IF NOT EXISTS idx_demand_status ON demand (status);
CREATE INDEX IF NOT EXISTS idx_demand_ca_forum ON demand (academic_center_id, forum);
CREATE INDEX IF NOT EXISTS idx_demand_created_at ON demand (created_at DESC);

CREATE TABLE IF NOT EXISTS demand_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    demand_id UUID NOT NULL REFERENCES demand(id) ON DELETE CASCADE,
    from_status TEXT NULL,
    to_status TEXT NOT NULL,
    changed_by_id UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    comment TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_demand_status_history_from_status CHECK (
        from_status IS NULL OR from_status IN ('open', 'in_analysis', 'forwarded_to_coordination', 'completed')
    ),
    CONSTRAINT chk_demand_status_history_to_status CHECK (
        to_status IN ('open', 'in_analysis', 'forwarded_to_coordination', 'completed')
    ),
    CONSTRAINT chk_demand_status_history_comment_not_empty CHECK (
        comment IS NULL OR length(trim(comment)) > 0
    )
);

CREATE INDEX IF NOT EXISTS idx_demand_status_history_demand_id ON demand_status_history (demand_id);
CREATE INDEX IF NOT EXISTS idx_demand_status_history_changed_by_id ON demand_status_history (changed_by_id);
CREATE INDEX IF NOT EXISTS idx_demand_status_history_created_at ON demand_status_history (created_at ASC);
