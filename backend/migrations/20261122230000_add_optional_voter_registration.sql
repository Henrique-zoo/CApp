-- O vínculo é cadastral do CA e pode abranger vários cursos da mesma instituição.
-- Os nomes correspondem à evidência acadêmica sincronizada do diretório.
CREATE TABLE academic_center_course (
    academic_center_id uuid NOT NULL REFERENCES academic_center (id) ON DELETE RESTRICT,
    course_name text NOT NULL CHECK (course_name = btrim(course_name) AND course_name <> ''),
    PRIMARY KEY (academic_center_id, course_name)
);

ALTER TABLE election_board
    ADD COLUMN requires_voter_registration boolean NOT NULL DEFAULT false;

CREATE TABLE election_voter_registration (
    election_id uuid NOT NULL REFERENCES election_board (election_id) ON DELETE RESTRICT,
    account_id uuid NOT NULL REFERENCES app_user (id) ON DELETE RESTRICT,
    public_key bytea NOT NULL CHECK (octet_length(public_key) BETWEEN 1 AND 768),
    registered_at timestamptz NOT NULL,
    PRIMARY KEY (election_id, account_id)
);

CREATE TRIGGER immutable_voter_registration BEFORE UPDATE OR DELETE ON election_voter_registration
FOR EACH ROW EXECUTE FUNCTION reject_election_evidence_mutation();

CREATE TRIGGER immutable_voter_registration_policy
BEFORE UPDATE OF requires_voter_registration ON election_board
FOR EACH ROW
WHEN (OLD.requires_voter_registration IS DISTINCT FROM NEW.requires_voter_registration)
EXECUTE FUNCTION reject_election_evidence_mutation();
