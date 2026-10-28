-- Migration: 20261028000001_create_events_tables.sql
-- Tabelas de domínio para Eventos de Centros Acadêmicos e Inscrições (§ 4.1 do relatório de features)

CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    academic_center_id UUID NOT NULL,
    author_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    location TEXT NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    requires_registration BOOLEAN NOT NULL DEFAULT TRUE,
    max_capacity INTEGER NULL,
    confirmed_registrations INTEGER NOT NULL DEFAULT 0,
    registration_deadline TIMESTAMPTZ NULL,
    published_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_events_title_not_empty CHECK (length(trim(title)) > 0),
    CONSTRAINT chk_events_description_not_empty CHECK (length(trim(description)) > 0),
    CONSTRAINT chk_events_location_not_empty CHECK (length(trim(location)) > 0),
    CONSTRAINT chk_events_period CHECK (ends_at > starts_at),
    CONSTRAINT chk_events_status CHECK (
        status IN (
            'draft',
            'pending_approval',
            'approved',
            'rejected',
            'published',
            'cancelled',
            'rascunho',
            'pendente',
            'aprovado',
            'rejeitado',
            'publicado',
            'cancelado'
        )
    ),
    CONSTRAINT chk_events_max_capacity CHECK (max_capacity IS NULL OR max_capacity > 0),
    CONSTRAINT chk_events_confirmed_registrations CHECK (
        confirmed_registrations >= 0
        AND (max_capacity IS NULL OR confirmed_registrations <= max_capacity)
    ),
    CONSTRAINT chk_events_registration_deadline CHECK (
        registration_deadline IS NULL OR registration_deadline <= ends_at
    ),
    CONSTRAINT chk_events_timestamps CHECK (updated_at >= created_at)
);

CREATE INDEX IF NOT EXISTS idx_events_academic_center_status ON events (academic_center_id, status);
CREATE INDEX IF NOT EXISTS idx_events_starts_at ON events (starts_at);

CREATE TABLE IF NOT EXISTS event_registrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    participant_id UUID NOT NULL,
    status TEXT NOT NULL DEFAULT 'confirmed',
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cancelled_at TIMESTAMPTZ NULL,
    CONSTRAINT uk_event_registrations_event_participant UNIQUE (event_id, participant_id),
    CONSTRAINT chk_event_registrations_status CHECK (
        status IN (
            'confirmed',
            'cancelled',
            'waitlisted',
            'confirmada',
            'cancelada',
            'fila_espera'
        )
    ),
    CONSTRAINT chk_event_registrations_cancelled_at CHECK (
        (status IN ('cancelled', 'cancelada') AND cancelled_at IS NOT NULL)
        OR (status NOT IN ('cancelled', 'cancelada') AND cancelled_at IS NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_event_registrations_event_id ON event_registrations (event_id);
CREATE INDEX IF NOT EXISTS idx_event_registrations_participant_id ON event_registrations (participant_id);

CREATE TABLE IF NOT EXISTS ca_event_permissions (
    academic_center_id UUID NOT NULL,
    user_id UUID NOT NULL,
    permission TEXT NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (academic_center_id, user_id, permission),
    CONSTRAINT chk_ca_event_permissions_not_empty CHECK (length(trim(permission)) > 0)
);

CREATE INDEX IF NOT EXISTS idx_ca_event_permissions_lookup
    ON ca_event_permissions (academic_center_id, user_id);
