-- Migration: 20261028000001_create_events_tables.sql
-- Tabelas e tipos enumerados de domínio para Eventos, Homologação e Inscrições
-- conforme a seção 5 do Modelo Entidade-Relacionamento do CApp.

CREATE TYPE event_status AS ENUM (
    'draft',
    'pending_review',
    'published',
    'rejected',
    'cancelled'
);

CREATE TYPE publication_review_decision AS ENUM (
    'approved',
    'rejected'
);

CREATE TYPE event_registration_status AS ENUM (
    'registered',
    'waitlisted',
    'invited',
    'cancelled'
);

CREATE TABLE event (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    ca_id uuid NOT NULL REFERENCES academic_center(id) ON DELETE RESTRICT,
    author_id uuid NOT NULL,
    title text NOT NULL,
    description text NOT NULL,
    location text NOT NULL,
    starts_at timestamptz NOT NULL,
    ends_at timestamptz NOT NULL,
    capacity integer NULL,
    status event_status NOT NULL DEFAULT 'draft',
    revision integer NOT NULL DEFAULT 1,
    registration_policy jsonb NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT chk_event_title_not_empty CHECK (
        btrim(title) <> '' AND char_length(btrim(title)) <= 200
    ),
    CONSTRAINT chk_event_description_not_empty CHECK (
        btrim(description) <> '' AND char_length(btrim(description)) <= 5000
    ),
    CONSTRAINT chk_event_location_not_empty CHECK (
        btrim(location) <> '' AND char_length(btrim(location)) <= 300
    ),
    CONSTRAINT chk_event_period CHECK (ends_at > starts_at),
    CONSTRAINT chk_event_capacity_positive CHECK (capacity IS NULL OR capacity > 0),
    CONSTRAINT chk_event_revision_positive CHECK (revision >= 1),
    CONSTRAINT chk_event_registration_policy_object CHECK (
        jsonb_typeof(registration_policy) = 'object'
    )
);

CREATE INDEX idx_event_ca_id_status ON event (ca_id, status);
CREATE INDEX idx_event_author_id ON event (author_id);
CREATE INDEX idx_event_starts_at ON event (starts_at);

CREATE TABLE event_review (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id uuid NOT NULL REFERENCES event(id) ON DELETE CASCADE,
    reviewer_id uuid NOT NULL,
    reviewed_revision integer NOT NULL,
    decision publication_review_decision NOT NULL,
    reason text NULL,
    decided_at timestamptz NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_event_review_revision_positive CHECK (reviewed_revision >= 1),
    CONSTRAINT chk_event_review_reason_not_empty CHECK (
        reason IS NULL OR btrim(reason) <> ''
    )
);

CREATE INDEX idx_event_review_event_id ON event_review (event_id);
CREATE INDEX idx_event_review_reviewer_id ON event_review (reviewer_id);

CREATE TABLE event_registration (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id uuid NOT NULL REFERENCES event(id) ON DELETE CASCADE,
    user_id uuid NOT NULL,
    status event_registration_status NOT NULL DEFAULT 'registered',
    queue_sequence bigint NULL,
    registered_at timestamptz NOT NULL DEFAULT NOW(),
    invitation_expires_at timestamptz NULL,
    cancelled_at timestamptz NULL,
    CONSTRAINT uk_event_registration_event_user UNIQUE (event_id, user_id),
    CONSTRAINT uk_event_registration_event_queue_sequence UNIQUE (event_id, queue_sequence),
    CONSTRAINT chk_event_registration_queue_sequence_positive CHECK (
        queue_sequence IS NULL OR queue_sequence >= 1
    ),
    CONSTRAINT chk_event_registration_invitation_expires_at CHECK (
        invitation_expires_at IS NULL OR invitation_expires_at > registered_at
    ),
    CONSTRAINT chk_event_registration_cancelled_at CHECK (
        (status = 'cancelled' AND cancelled_at IS NOT NULL AND cancelled_at >= registered_at)
        OR (status <> 'cancelled' AND cancelled_at IS NULL)
    )
);

CREATE INDEX idx_event_registration_event_id ON event_registration (event_id);
CREATE INDEX idx_event_registration_user_id ON event_registration (user_id);
