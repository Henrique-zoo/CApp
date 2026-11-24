-- Migration: 20261008000002_create_app_user_table.sql
-- Tabela de domínio para Usuários da aplicação conforme modelo ER oficial

CREATE TABLE IF NOT EXISTS app_user (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name TEXT NOT NULL,
    institutional_email TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uk_app_user_institutional_email UNIQUE (institutional_email),
    CONSTRAINT chk_app_user_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    CONSTRAINT chk_app_user_institutional_email_not_empty CHECK (length(trim(institutional_email)) > 0),
    CONSTRAINT chk_app_user_status CHECK (status IN ('active', 'inactive')),
    CONSTRAINT chk_app_user_timestamps CHECK (updated_at >= created_at)
);

CREATE INDEX IF NOT EXISTS idx_app_user_status ON app_user (status);
CREATE INDEX IF NOT EXISTS idx_app_user_institutional_email ON app_user (institutional_email);
