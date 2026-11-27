-- Catálogo de disciplinas já validadas para a grade atendida pelo CA.
CREATE TABLE academic_center_discipline (
    ca_id uuid NOT NULL REFERENCES academic_center(id) ON DELETE RESTRICT,
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    name text NOT NULL CHECK (char_length(btrim(name)) BETWEEN 1 AND 200),
    PRIMARY KEY (ca_id, id)
);

-- Concessões explícitas da capacidade Gerenciar materiais neste CA.
CREATE TABLE academic_center_material_publisher (
    ca_id uuid NOT NULL REFERENCES academic_center(id) ON DELETE RESTRICT,
    account_id uuid NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    PRIMARY KEY (ca_id, account_id)
);

CREATE TABLE study_material (
    id uuid PRIMARY KEY,
    ca_id uuid NOT NULL REFERENCES academic_center(id) ON DELETE RESTRICT,
    author_id uuid NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    discipline_id uuid NOT NULL,
    title text NOT NULL CHECK (char_length(btrim(title)) BETWEEN 1 AND 200),
    description text NOT NULL CHECK (char_length(description) <= 5000),
    kind text NOT NULL CHECK (kind IN ('external_link', 'file')),
    external_url text,
    file_name text,
    file_size integer,
    file_sha256 bytea,
    status text NOT NULL CHECK (status IN ('published', 'archived')),
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    request_id uuid NOT NULL,
    FOREIGN KEY (ca_id, discipline_id)
        REFERENCES academic_center_discipline(ca_id, id) ON DELETE RESTRICT,
    UNIQUE (ca_id, author_id, request_id),
    CONSTRAINT study_material_content_matches_kind CHECK (
        (kind = 'external_link' AND external_url IS NOT NULL
            AND char_length(external_url) BETWEEN 1 AND 2048
            AND file_name IS NULL AND file_size IS NULL AND file_sha256 IS NULL)
        OR (kind = 'file' AND external_url IS NULL AND file_name IS NOT NULL
            AND char_length(file_name) BETWEEN 1 AND 255
            AND file_size IS NOT NULL AND file_size BETWEEN 1 AND 10485760
            AND file_sha256 IS NOT NULL AND octet_length(file_sha256) = 32)
    )
);

CREATE INDEX study_material_ca_published_idx
    ON study_material(ca_id, created_at DESC, id DESC) WHERE status = 'published';
CREATE INDEX study_material_discipline_idx
    ON study_material(ca_id, discipline_id, status, created_at DESC, id DESC);
CREATE INDEX study_material_author_idx ON study_material(author_id);
CREATE INDEX academic_center_material_publisher_account_idx
    ON academic_center_material_publisher(account_id);
