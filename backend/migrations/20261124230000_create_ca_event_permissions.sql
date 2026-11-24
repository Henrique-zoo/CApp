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
