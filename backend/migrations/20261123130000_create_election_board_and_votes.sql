-- A eleição guarda a configuração vigente e é o bloqueio comum de todas as operações.
CREATE TABLE election (
    election_id uuid PRIMARY KEY CHECK (election_id <> '00000000-0000-0000-0000-000000000000'),
    academic_center_id uuid NOT NULL REFERENCES academic_center (id) ON DELETE RESTRICT,
    starts_at timestamptz NOT NULL,
    ends_at timestamptz NOT NULL CHECK (ends_at > starts_at),
    requires_voter_registration boolean NOT NULL DEFAULT false,
    status text NOT NULL DEFAULT 'preparing'
        CHECK (status IN ('preparing', 'published', 'closed', 'cancelled')),
    cancelled_at timestamptz,
    cancelled_by uuid REFERENCES app_user (id) ON DELETE RESTRICT,
    cancellation_reason text,
    CHECK (
        (status = 'cancelled' AND cancelled_at IS NOT NULL AND cancelled_by IS NOT NULL
            AND cancellation_reason IS NOT NULL AND length(btrim(cancellation_reason)) BETWEEN 1 AND 2000)
        OR (status <> 'cancelled' AND cancelled_at IS NULL AND cancelled_by IS NULL
            AND cancellation_reason IS NULL)
    )
);

CREATE INDEX election_academic_center_idx ON election (academic_center_id);
CREATE INDEX election_deadline_idx ON election (ends_at, election_id) WHERE status = 'published';

-- Deriva a ordem diretamente do manifesto assinado, inclusive a posição de Nulo.
CREATE FUNCTION election_manifest_options(manifest jsonb) RETURNS uuid[]
LANGUAGE sql IMMUTABLE STRICT AS $$
    SELECT array_agg((item ->> 'id')::uuid ORDER BY position)
    FROM jsonb_array_elements(manifest #> '{payload,options}') WITH ORDINALITY AS options(item, position)
    HAVING count(*) = count(DISTINCT (item ->> 'id')::uuid);
$$;

CREATE TABLE election_board (
    election_id uuid PRIMARY KEY REFERENCES election (election_id) ON DELETE RESTRICT,
    manifest jsonb NOT NULL CHECK (jsonb_typeof(manifest) = 'object'),
    manifest_hash text NOT NULL CHECK (manifest_hash ~ '^[0-9a-f]{64}$'),
    option_ids uuid[] GENERATED ALWAYS AS (election_manifest_options(manifest)) STORED NOT NULL,
    server_signing_key text GENERATED ALWAYS AS (manifest #>> '{payload,serverSigningKey}') STORED NOT NULL,
    head_hash text NOT NULL CHECK (head_hash ~ '^[0-9a-f]{64}$'),
    voter_count integer NOT NULL DEFAULT 0 CHECK (voter_count BETWEEN 0 AND 100000),
    -- Cada voto acrescenta uma cifra a cada posição, com dois inteiros de 256 bytes.
    accumulator bytea NOT NULL CHECK (octet_length(accumulator) = 512 * cardinality(option_ids)),
    closure jsonb CHECK (jsonb_typeof(closure) = 'object'),
    aggregate jsonb CHECK (jsonb_typeof(aggregate) = 'object'),
    CHECK ((closure IS NULL) = (aggregate IS NULL)),
    CHECK (cardinality(option_ids) BETWEEN 2 AND 51 AND array_position(option_ids, NULL) IS NULL),
    CHECK (option_ids[cardinality(option_ids)] = '00000000-0000-0000-0000-000000000000'),
    CHECK (server_signing_key <> ''),
    CHECK ((manifest #>> '{payload,electionId}') IS NOT DISTINCT FROM election_id::text),
    CHECK (closure IS NULL OR (
        (closure #>> '{payload,kind}') IS NOT DISTINCT FROM 'closure'
        AND (closure #>> '{payload,electionId}') IS NOT DISTINCT FROM election_id::text
        AND (closure #>> '{payload,manifestHash}') IS NOT DISTINCT FROM manifest_hash
        AND (closure #>> '{payload,voterCount}') IS NOT DISTINCT FROM voter_count::text
        AND (closure #>> '{payload,headHash}') IS NOT DISTINCT FROM head_hash
    )),
    CHECK (aggregate IS NULL OR (
        (aggregate #>> '{payload,kind}') IS NOT DISTINCT FROM 'aggregate'
        AND (aggregate #>> '{payload,electionId}') IS NOT DISTINCT FROM election_id::text
        AND (aggregate #>> '{payload,manifestHash}') IS NOT DISTINCT FROM manifest_hash
    ))
);

CREATE TABLE election_vote (
    election_id uuid NOT NULL REFERENCES election_board (election_id) ON DELETE RESTRICT,
    account_id uuid NOT NULL REFERENCES app_user (id) ON DELETE RESTRICT,
    accepted_at timestamptz NOT NULL,
    ballot_hash bytea NOT NULL CHECK (octet_length(ballot_hash) = 32),
    public_key bytea NOT NULL CHECK (octet_length(public_key) BETWEEN 1 AND 768),
    entry_index integer NOT NULL CHECK (entry_index BETWEEN 1 AND 100000),
    previous_hash text GENERATED ALWAYS AS (signed_entry #>> '{payload,previousHash}') STORED
        NOT NULL CHECK (previous_hash ~ '^[0-9a-f]{64}$'),
    entry_hash text NOT NULL CHECK (entry_hash ~ '^[0-9a-f]{64}$'),
    signed_entry jsonb NOT NULL CHECK (jsonb_typeof(signed_entry) = 'object'),
    signed_receipt jsonb NOT NULL CHECK (jsonb_typeof(signed_receipt) = 'object'),
    PRIMARY KEY (election_id, account_id),
    UNIQUE (election_id, entry_index),
    CHECK ((signed_entry #>> '{payload,electionId}') IS NOT DISTINCT FROM election_id::text),
    CHECK ((signed_entry #>> '{payload,ballot,payload,accountId}') IS NOT DISTINCT FROM account_id::text),
    CHECK ((signed_entry #>> '{payload,index}') IS NOT DISTINCT FROM entry_index::text),
    CHECK ((signed_receipt #>> '{payload,electionId}') IS NOT DISTINCT FROM election_id::text),
    CHECK ((signed_receipt #>> '{payload,index}') IS NOT DISTINCT FROM entry_index::text),
    CHECK ((signed_receipt #>> '{payload,ballotHash}') IS NOT DISTINCT FROM encode(ballot_hash, 'hex')),
    CHECK ((signed_receipt #>> '{payload,entryHash}') IS NOT DISTINCT FROM entry_hash)
);

-- Os cursos do CA correspondem à evidência acadêmica sincronizada do diretório.
CREATE TABLE academic_center_course (
    academic_center_id uuid NOT NULL REFERENCES academic_center (id) ON DELETE RESTRICT,
    course_name text NOT NULL CHECK (course_name = btrim(course_name) AND course_name <> ''),
    PRIMARY KEY (academic_center_id, course_name)
);

CREATE TABLE election_voter_registration (
    election_id uuid NOT NULL REFERENCES election_board (election_id) ON DELETE RESTRICT,
    account_id uuid NOT NULL REFERENCES app_user (id) ON DELETE RESTRICT,
    public_key bytea NOT NULL CHECK (octet_length(public_key) BETWEEN 1 AND 768),
    registered_at timestamptz NOT NULL,
    PRIMARY KEY (election_id, account_id)
);

CREATE FUNCTION reject_election_evidence_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'accepted electoral evidence is append-only' USING ERRCODE = '23514';
END;
$$;

CREATE TRIGGER immutable_election_vote
BEFORE UPDATE OR DELETE ON election_vote
FOR EACH ROW EXECUTE FUNCTION reject_election_evidence_mutation();

CREATE TRIGGER immutable_voter_registration
BEFORE UPDATE OR DELETE ON election_voter_registration
FOR EACH ROW EXECUTE FUNCTION reject_election_evidence_mutation();

-- O mesmo bloqueio protege o calendário, o aceite, o cadastro e o encerramento.
CREATE FUNCTION protect_election_board() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    config election%ROWTYPE;
BEGIN
    SELECT * INTO config FROM election WHERE election_id = NEW.election_id FOR UPDATE;
    IF TG_OP = 'INSERT' THEN
        IF config.status IS DISTINCT FROM 'preparing'
           OR (NEW.manifest #>> '{payload,academicCenterId}') IS DISTINCT FROM config.academic_center_id::text
           OR NEW.voter_count <> 0 OR NEW.head_hash <> NEW.manifest_hash OR NEW.closure IS NOT NULL THEN
            RAISE EXCEPTION 'invalid initial electoral board' USING ERRCODE = '23514';
        END IF;
        RETURN NEW;
    END IF;

    IF config.status IS DISTINCT FROM 'published'
       OR OLD.election_id IS DISTINCT FROM NEW.election_id
       OR OLD.manifest IS DISTINCT FROM NEW.manifest OR OLD.manifest_hash <> NEW.manifest_hash
       OR OLD.closure IS NOT NULL THEN
        RAISE EXCEPTION 'frozen electoral board evidence' USING ERRCODE = '23514';
    END IF;

    IF NEW.voter_count <> OLD.voter_count OR NEW.head_hash <> OLD.head_hash
       OR NEW.accumulator <> OLD.accumulator THEN
        IF NEW.voter_count <> OLD.voter_count + 1 OR NEW.closure IS NOT NULL THEN
            RAISE EXCEPTION 'the accumulator advances one accepted vote at a time' USING ERRCODE = '23514';
        END IF;
    END IF;
    IF NEW.closure IS NOT NULL AND clock_timestamp() < config.ends_at THEN
        RAISE EXCEPTION 'the voting period is still open' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER protect_election_board
BEFORE INSERT OR UPDATE ON election_board
FOR EACH ROW EXECUTE FUNCTION protect_election_board();

CREATE TRIGGER retain_election_board
BEFORE DELETE ON election_board
FOR EACH ROW EXECUTE FUNCTION reject_election_evidence_mutation();

CREATE FUNCTION validate_vote_registration_window() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    config election%ROWTYPE;
BEGIN
    SELECT * INTO config FROM election WHERE election_id = NEW.election_id FOR UPDATE;
    IF config.status IS DISTINCT FROM 'published' OR NOT config.requires_voter_registration
       OR NEW.registered_at >= config.starts_at OR NEW.registered_at > clock_timestamp() THEN
        RAISE EXCEPTION 'voter registration is unavailable for this period' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER validate_vote_registration_window
BEFORE INSERT ON election_voter_registration
FOR EACH ROW EXECUTE FUNCTION validate_vote_registration_window();

CREATE FUNCTION validate_vote_window_and_position() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    config election%ROWTYPE;
    board election_board%ROWTYPE;
BEGIN
    SELECT * INTO config FROM election WHERE election_id = NEW.election_id FOR UPDATE;
    SELECT * INTO board FROM election_board WHERE election_id = NEW.election_id;
    IF config.status IS DISTINCT FROM 'published' OR board.closure IS NOT NULL
       OR NEW.accepted_at < config.starts_at OR NEW.accepted_at >= config.ends_at
       OR NEW.accepted_at > clock_timestamp()
       OR NEW.entry_index IS DISTINCT FROM board.voter_count + 1
       OR (NEW.signed_entry #>> '{payload,previousHash}') IS DISTINCT FROM board.head_hash
       OR (NEW.signed_entry #>> '{payload,manifestHash}') IS DISTINCT FROM board.manifest_hash
       OR (NEW.signed_receipt #>> '{payload,manifestHash}') IS DISTINCT FROM board.manifest_hash THEN
        RAISE EXCEPTION 'vote does not match the open board' USING ERRCODE = '23514';
    END IF;
    IF config.requires_voter_registration AND NOT EXISTS (
        SELECT 1 FROM election_voter_registration
        WHERE election_id = NEW.election_id AND account_id = NEW.account_id AND public_key = NEW.public_key
    ) THEN
        RAISE EXCEPTION 'vote key does not match voter registration' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER validate_vote_window_and_position
BEFORE INSERT ON election_vote
FOR EACH ROW EXECUTE FUNCTION validate_vote_window_and_position();

-- No commit, toda linha de voto precisa estar refletida no contador do quadro.
CREATE FUNCTION validate_accepted_vote_commit() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM election_board b WHERE b.election_id = NEW.election_id
        AND (b.voter_count > NEW.entry_index
            OR (b.voter_count = NEW.entry_index AND b.head_hash = NEW.entry_hash))
    ) THEN
        RAISE EXCEPTION 'accepted vote is absent from the board accumulator' USING ERRCODE = '23514';
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER validate_accepted_vote_commit
AFTER INSERT ON election_vote DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION validate_accepted_vote_commit();

CREATE FUNCTION validate_board_update_commit() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'UPDATE' AND NEW.voter_count > OLD.voter_count AND NOT EXISTS (
        SELECT 1 FROM election_vote v
        WHERE v.election_id = NEW.election_id AND v.entry_index = NEW.voter_count AND v.entry_hash = NEW.head_hash
    ) THEN
        RAISE EXCEPTION 'board advance is missing its accepted vote' USING ERRCODE = '23514';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM election e JOIN election_board b USING (election_id)
        WHERE e.election_id = NEW.election_id AND e.status <> 'preparing'
            AND (e.status = 'closed') = (b.closure IS NOT NULL)
    ) THEN
        RAISE EXCEPTION 'election status and board evidence disagree' USING ERRCODE = '23514';
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER validate_board_publication_commit
AFTER INSERT ON election_board DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION validate_board_update_commit();

CREATE CONSTRAINT TRIGGER validate_board_update_commit
AFTER UPDATE ON election_board DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION validate_board_update_commit();
