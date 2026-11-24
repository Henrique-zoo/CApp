-- Migration: 20261008000003_create_user_preference_table.sql
-- Tabela de preferências do usuário para persistência de CA favorito (Issue #44)
-- Conforme § 2.3 do Modelo ER e § 2.1 do Relatório de Features

CREATE TABLE IF NOT EXISTS user_preference (
    user_id UUID PRIMARY KEY,
    favorite_ca_id UUID NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_user_preference_user FOREIGN KEY (user_id) REFERENCES app_user(id) ON DELETE CASCADE,
    CONSTRAINT fk_user_preference_favorite_ca FOREIGN KEY (favorite_ca_id) REFERENCES academic_center(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_user_preference_favorite_ca_id ON user_preference (favorite_ca_id);
