-- Migration: 20261122120000_create_demand_notifications_table.sql
-- Tabela de notificações para acompanhamento de atualizações em demandas.

CREATE TABLE IF NOT EXISTS demand_notification (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    demand_id UUID NOT NULL REFERENCES demand(id) ON DELETE CASCADE,
    recipient_user_id UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    academic_center_id UUID NOT NULL REFERENCES academic_center(id) ON DELETE RESTRICT,
    event_type TEXT NOT NULL,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    read_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_demand_notification_event_type CHECK (
        event_type IN ('status_updated', 'official_response')
    ),
    CONSTRAINT chk_demand_notification_title_not_empty CHECK (
        length(trim(title)) > 0
    ),
    CONSTRAINT chk_demand_notification_message_not_empty CHECK (
        length(trim(message)) > 0
    ),
    CONSTRAINT chk_demand_notification_read_at CHECK (
        read_at IS NULL OR read_at >= created_at
    )
);

CREATE INDEX IF NOT EXISTS idx_demand_notification_recipient_created
    ON demand_notification (recipient_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_demand_notification_demand_id
    ON demand_notification (demand_id);

CREATE INDEX IF NOT EXISTS idx_demand_notification_ca_id
    ON demand_notification (academic_center_id);

CREATE INDEX IF NOT EXISTS idx_demand_notification_recipient_unread
    ON demand_notification (recipient_user_id)
    WHERE read_at IS NULL;
