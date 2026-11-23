-- Migration: 20261123140000_create_student_polls_tables.sql
-- Tabelas de domínio para Consultas e Enquetes Estudantis do CA (Issue #299)

CREATE TABLE IF NOT EXISTS student_poll (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    academic_center_id UUID NOT NULL REFERENCES academic_center(id) ON DELETE CASCADE,
    created_by_id UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    target_audience TEXT NOT NULL DEFAULT 'all_students',
    vote_modality TEXT NOT NULL DEFAULT 'identified',
    starts_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ends_at TIMESTAMPTZ NULL,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ NULL,
    closed_by_id UUID NULL REFERENCES app_user(id) ON DELETE SET NULL,
    CONSTRAINT chk_student_poll_title_not_empty CHECK (btrim(title) <> '' AND length(btrim(title)) <= 255),
    CONSTRAINT chk_student_poll_description_length CHECK (length(description) <= 4000),
    CONSTRAINT chk_student_poll_target_audience_not_empty CHECK (btrim(target_audience) <> '' AND length(btrim(target_audience)) <= 100),
    CONSTRAINT chk_student_poll_vote_modality CHECK (vote_modality IN ('identified', 'secret')),
    CONSTRAINT chk_student_poll_status CHECK (status IN ('open', 'closed')),
    CONSTRAINT chk_student_poll_period CHECK (ends_at IS NULL OR ends_at > starts_at),
    CONSTRAINT chk_student_poll_timestamps CHECK (updated_at >= created_at),
    CONSTRAINT chk_student_poll_closed_status CHECK (
        (status = 'closed' AND closed_at IS NOT NULL) OR
        (status = 'open' AND closed_at IS NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_student_poll_academic_center_id ON student_poll (academic_center_id);
CREATE INDEX IF NOT EXISTS idx_student_poll_status ON student_poll (status);
CREATE INDEX IF NOT EXISTS idx_student_poll_starts_at ON student_poll (starts_at);

CREATE TABLE IF NOT EXISTS student_poll_option (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID NOT NULL REFERENCES student_poll(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    option_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_student_poll_option_title_not_empty CHECK (btrim(title) <> '' AND length(btrim(title)) <= 255),
    CONSTRAINT uk_student_poll_option_poll_title UNIQUE (poll_id, title)
);

CREATE INDEX IF NOT EXISTS idx_student_poll_option_poll_id ON student_poll_option (poll_id);

CREATE TABLE IF NOT EXISTS student_poll_vote (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID NOT NULL REFERENCES student_poll(id) ON DELETE CASCADE,
    option_id UUID NOT NULL REFERENCES student_poll_option(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    voted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uk_student_poll_vote_poll_user UNIQUE (poll_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_student_poll_vote_poll_id ON student_poll_vote (poll_id);
CREATE INDEX IF NOT EXISTS idx_student_poll_vote_option_id ON student_poll_vote (option_id);
CREATE INDEX IF NOT EXISTS idx_student_poll_vote_user_id ON student_poll_vote (user_id);
