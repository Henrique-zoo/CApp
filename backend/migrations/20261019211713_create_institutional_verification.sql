CREATE TABLE institutional_verification (
    firebase_uid VARCHAR(128) PRIMARY KEY,
    microsoft_object_id UUID NOT NULL,
    institution_id UUID NOT NULL
        REFERENCES institution(id) ON DELETE CASCADE,

    microsoft_tenant_id UUID NOT NULL,
    microsoft_student_group_id UUID NOT NULL,

    student_membership_confirmed BOOLEAN NOT NULL,
    checked_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT institutional_verification_uid_not_empty
        CHECK (firebase_uid <> ''),

    CONSTRAINT institutional_verification_valid_period
        CHECK (expires_at > checked_at)
);

CREATE INDEX institutional_verification_institution_idx
    ON institutional_verification (institution_id);