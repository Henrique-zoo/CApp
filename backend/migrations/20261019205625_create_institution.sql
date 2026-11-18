CREATE TYPE catalog_status AS ENUM(
    'active',
    'inactive'
);

CREATE TABLE institution(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    acronym text,
    microsoft_tenant_id uuid NOT NULL,
    microsoft_student_group_id uuid NOT NULL,
    status catalog_status NOT NULL DEFAULT 'active',
    CONSTRAINT institution_name_not_blank CHECK (btrim(name) <> ''),
    CONSTRAINT institution_microsoft_tenant_id_unique UNIQUE (microsoft_tenant_id),
    CONSTRAINT institution_acronym_not_blank CHECK (acronym IS NULL OR btrim(acronym) <> '')
);

