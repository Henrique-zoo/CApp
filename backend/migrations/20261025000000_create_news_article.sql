-- Migration para criação da tabela news_article
-- Modelo Entidade-Relacionamento do CApp: mural administrado de notícias

CREATE TABLE IF NOT EXISTS news_article (
    id UUID PRIMARY KEY,
    ca_id UUID NOT NULL,
    author_id UUID NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    status TEXT NOT NULL,
    revision INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL,
    published_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_news_article_ca_id ON news_article (ca_id);
CREATE INDEX IF NOT EXISTS idx_news_article_status ON news_article (status);
