-- Preserva as migrations anteriores e os UIDs já verificados.
CREATE TYPE user_status AS ENUM ('incomplete', 'active', 'inactive');

CREATE TABLE app_user (
    id UUID PRIMARY KEY,
    firebase_uid VARCHAR(128) NOT NULL UNIQUE CHECK (firebase_uid <> ''),
    name TEXT,
    nickname TEXT,
    email TEXT,
    current_semester INTEGER CHECK (current_semester >= 1),
    institution_entry_semester TEXT CHECK (institution_entry_semester ~ '^[0-9]{4}\.[12]$'),
    status user_status NOT NULL DEFAULT 'incomplete',
    profile_completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_at TIMESTAMPTZ,
    CHECK (status <> 'active' OR (nickname IS NOT NULL AND btrim(nickname) <> ''
        AND email IS NOT NULL AND email <> '' AND current_semester IS NOT NULL
        AND institution_entry_semester IS NOT NULL AND profile_completed_at IS NOT NULL))
);

INSERT INTO app_user (id, firebase_uid)
SELECT gen_random_uuid(), firebase_uid FROM institutional_verification;
ALTER TABLE institutional_verification
    ADD COLUMN user_id UUID REFERENCES app_user(id) ON DELETE CASCADE,
    ADD COLUMN profile_name TEXT,
    ADD COLUMN profile_email TEXT,
    ADD COLUMN course_name TEXT;
UPDATE institutional_verification v SET user_id = u.id
FROM app_user u WHERE u.firebase_uid = v.firebase_uid;
ALTER TABLE institutional_verification
    ALTER COLUMN user_id SET NOT NULL,
    DROP CONSTRAINT institutional_verification_pkey,
    ADD PRIMARY KEY (user_id),
    DROP COLUMN firebase_uid;
