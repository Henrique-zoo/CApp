-- O manifesto assinado identifica o CA; o curso aceito vem da preparação privada.
CREATE TABLE election_eligibility_course (
    election_id uuid PRIMARY KEY REFERENCES election_board (election_id) ON DELETE RESTRICT,
    academic_center_id uuid NOT NULL REFERENCES academic_center (id) ON DELETE RESTRICT,
    course_name text NOT NULL CHECK (course_name = btrim(course_name) AND course_name <> '')
);

-- O identificador aleatório nasce somente quando uma conta comprova a posse da chave.
CREATE TABLE election_voter_registration (
    election_id uuid NOT NULL REFERENCES election_eligibility_course (election_id) ON DELETE RESTRICT,
    account_id uuid NOT NULL REFERENCES app_user (id) ON DELETE RESTRICT,
    voter_id uuid NOT NULL DEFAULT gen_random_uuid(),
    public_key text NOT NULL CHECK (length(public_key) BETWEEN 1 AND 1024),
    registered_at timestamptz NOT NULL,
    PRIMARY KEY (election_id, account_id),
    UNIQUE (election_id, voter_id)
);

CREATE TRIGGER immutable_election_course BEFORE UPDATE OR DELETE ON election_eligibility_course
FOR EACH ROW EXECUTE FUNCTION reject_election_evidence_mutation();
CREATE TRIGGER immutable_voter_registration BEFORE UPDATE OR DELETE ON election_voter_registration
FOR EACH ROW EXECUTE FUNCTION reject_election_evidence_mutation();
