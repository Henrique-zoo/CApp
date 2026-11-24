-- Cada autoridade tem uma única designação. O servidor guarda apenas o próprio segredo.
CREATE TABLE election_authority (
    election_id uuid NOT NULL REFERENCES election (election_id) ON DELETE RESTRICT,
    authority_id uuid NOT NULL CHECK (authority_id <> '00000000-0000-0000-0000-000000000000'),
    account_id uuid REFERENCES app_user (id) ON DELETE RESTRICT,
    role text NOT NULL,
    slate_id uuid,
    slate_name text,
    position integer,
    credential jsonb CHECK (jsonb_typeof(credential) = 'object'),
    server_secret bytea CHECK (octet_length(server_secret) = 256),
    PRIMARY KEY (election_id, authority_id),
    UNIQUE (election_id, account_id),
    UNIQUE (election_id, slate_id),
    UNIQUE (election_id, position),
    CHECK (
        (role = 'server' AND account_id IS NULL AND server_secret IS NOT NULL AND credential IS NULL
            AND slate_id IS NULL AND slate_name IS NULL AND position IS NULL)
        OR (role = 'commission' AND account_id IS NOT NULL AND server_secret IS NULL
            AND slate_id IS NULL AND slate_name IS NULL AND position IS NULL)
        OR (role = 'slate' AND account_id IS NOT NULL AND server_secret IS NULL
            AND slate_id IS NOT NULL AND slate_id <> '00000000-0000-0000-0000-000000000000'
            AND slate_name IS NOT NULL AND length(btrim(slate_name)) BETWEEN 1 AND 200
            AND position IS NOT NULL AND position BETWEEN 1 AND 50)
    )
);

CREATE UNIQUE INDEX election_single_commission_idx
    ON election_authority (election_id) WHERE role = 'commission';
CREATE UNIQUE INDEX election_single_server_idx
    ON election_authority (election_id) WHERE role = 'server';

CREATE FUNCTION protect_election_authority() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    current_status text;
BEGIN
    SELECT status INTO current_status FROM election
    WHERE election_id = CASE WHEN TG_OP = 'INSERT' THEN NEW.election_id ELSE OLD.election_id END
    FOR UPDATE;

    IF current_status IS DISTINCT FROM 'preparing' THEN
        RAISE EXCEPTION 'published electoral assignments are immutable' USING ERRCODE = '23514';
    END IF;
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    IF TG_OP = 'UPDATE' AND (OLD.election_id <> NEW.election_id OR OLD.authority_id <> NEW.authority_id) THEN
        RAISE EXCEPTION 'electoral authority identity is immutable' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER protect_election_authority
BEFORE INSERT OR UPDATE OR DELETE ON election_authority
FOR EACH ROW EXECUTE FUNCTION protect_election_authority();

CREATE FUNCTION protect_election_lifecycle() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.status <> 'preparing' THEN
            RAISE EXCEPTION 'an election starts in preparation' USING ERRCODE = '23514';
        END IF;
        RETURN NEW;
    END IF;

    IF OLD.election_id <> NEW.election_id OR OLD.academic_center_id <> NEW.academic_center_id
       OR OLD.status IN ('closed', 'cancelled')
       OR (OLD.status <> 'preparing' AND OLD.requires_voter_registration <> NEW.requires_voter_registration) THEN
        RAISE EXCEPTION 'frozen election configuration' USING ERRCODE = '23514';
    END IF;

    IF NEW.starts_at <> OLD.starts_at OR NEW.ends_at <> OLD.ends_at THEN
        IF NEW.status <> OLD.status OR NEW.starts_at < OLD.starts_at OR NEW.ends_at <= OLD.ends_at
           OR clock_timestamp() >= OLD.ends_at
           OR (OLD.status = 'published' AND clock_timestamp() >= OLD.starts_at AND NEW.starts_at <> OLD.starts_at) THEN
            RAISE EXCEPTION 'invalid electoral schedule change' USING ERRCODE = '23514';
        END IF;
    END IF;

    IF NEW.status = OLD.status THEN
        RETURN NEW;
    END IF;
    CASE NEW.status
        WHEN 'published' THEN
            IF OLD.status <> 'preparing' OR NOT EXISTS (
                SELECT 1 FROM election_board WHERE election_id = NEW.election_id AND closure IS NULL
            ) THEN
                RAISE EXCEPTION 'publication requires an electoral board' USING ERRCODE = '23514';
            END IF;
        WHEN 'closed' THEN
            IF OLD.status <> 'published' OR clock_timestamp() < OLD.ends_at OR NOT EXISTS (
                SELECT 1 FROM election_board WHERE election_id = NEW.election_id AND closure IS NOT NULL
            ) THEN
                RAISE EXCEPTION 'invalid electoral closure' USING ERRCODE = '23514';
            END IF;
        WHEN 'cancelled' THEN
            IF NEW.cancelled_at >= OLD.ends_at OR NEW.cancelled_at > clock_timestamp()
               OR EXISTS (SELECT 1 FROM election_board WHERE election_id = NEW.election_id AND closure IS NOT NULL)
               OR NOT EXISTS (
                   SELECT 1 FROM election_authority WHERE election_id = NEW.election_id
                       AND role = 'commission' AND account_id = NEW.cancelled_by
               ) THEN
                RAISE EXCEPTION 'invalid electoral cancellation' USING ERRCODE = '23514';
            END IF;
        ELSE
            RAISE EXCEPTION 'invalid election transition' USING ERRCODE = '23514';
    END CASE;
    RETURN NEW;
END;
$$;

CREATE TRIGGER protect_election_lifecycle
BEFORE INSERT OR UPDATE ON election
FOR EACH ROW EXECUTE FUNCTION protect_election_lifecycle();

CREATE TABLE election_schedule_revision (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    election_id uuid NOT NULL REFERENCES election (election_id) ON DELETE RESTRICT,
    changed_by uuid NOT NULL REFERENCES app_user (id) ON DELETE RESTRICT,
    changed_at timestamptz NOT NULL,
    previous_starts_at timestamptz NOT NULL,
    previous_ends_at timestamptz NOT NULL,
    starts_at timestamptz NOT NULL,
    ends_at timestamptz NOT NULL,
    reason text NOT NULL CHECK (length(btrim(reason)) BETWEEN 1 AND 2000),
    CHECK (previous_starts_at < previous_ends_at),
    CHECK (starts_at >= previous_starts_at AND ends_at > previous_ends_at AND starts_at < ends_at)
);

CREATE INDEX election_schedule_revision_election_idx ON election_schedule_revision (election_id, id);

CREATE FUNCTION validate_election_schedule_revision() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM election e JOIN election_authority a USING (election_id)
        WHERE e.election_id = NEW.election_id AND a.account_id = NEW.changed_by AND a.role = 'commission'
          AND e.starts_at = NEW.starts_at AND e.ends_at = NEW.ends_at
          AND e.status IN ('preparing', 'published')
    ) THEN
        RAISE EXCEPTION 'invalid election schedule revision' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER validate_election_schedule_revision
BEFORE INSERT ON election_schedule_revision
FOR EACH ROW EXECUTE FUNCTION validate_election_schedule_revision();

CREATE TRIGGER immutable_election_schedule_revision
BEFORE UPDATE OR DELETE ON election_schedule_revision
FOR EACH ROW EXECUTE FUNCTION reject_election_evidence_mutation();

CREATE TABLE election_decryption_share (
    election_id uuid NOT NULL REFERENCES election_board (election_id) ON DELETE RESTRICT,
    authority_id uuid NOT NULL,
    digest text NOT NULL CHECK (digest ~ '^[0-9a-f]{64}$'),
    evidence jsonb NOT NULL CHECK (jsonb_typeof(evidence) = 'object'),
    PRIMARY KEY (election_id, authority_id),
    FOREIGN KEY (election_id, authority_id) REFERENCES election_authority (election_id, authority_id) ON DELETE RESTRICT,
    CHECK ((evidence #>> '{payload,kind}') IS NOT DISTINCT FROM 'decryption-share'),
    CHECK ((evidence #>> '{payload,electionId}') IS NOT DISTINCT FROM election_id::text),
    CHECK ((evidence #>> '{payload,authorityId}') IS NOT DISTINCT FROM authority_id::text)
);

CREATE TABLE election_tally_result (
    election_id uuid PRIMARY KEY REFERENCES election_board (election_id) ON DELETE RESTRICT,
    result jsonb NOT NULL CHECK (jsonb_typeof(result) = 'object'),
    status text GENERATED ALWAYS AS (result #>> '{payload,status}') STORED NOT NULL
        CHECK (status IN ('incomplete', 'valid', 'annulled')),
    CHECK ((result #>> '{payload,kind}') IS NOT DISTINCT FROM 'result'),
    CHECK ((result #>> '{payload,electionId}') IS NOT DISTINCT FROM election_id::text)
);

CREATE FUNCTION require_closed_election() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    current_status text;
BEGIN
    SELECT status INTO current_status FROM election WHERE election_id = NEW.election_id FOR UPDATE;
    IF current_status IS DISTINCT FROM 'closed' THEN
        RAISE EXCEPTION 'tally requires a closed election' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE FUNCTION validate_share_manifest() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM election_board b JOIN election_authority a USING (election_id)
        WHERE b.election_id = NEW.election_id AND a.authority_id = NEW.authority_id
          AND (b.manifest #> '{payload,authorities}') @> jsonb_build_array(
              jsonb_build_object('id', a.authority_id, 'role', a.role)
          )
          AND (NEW.evidence #>> '{payload,manifestHash}') = b.manifest_hash
    ) THEN
        RAISE EXCEPTION 'contribution authority or manifest does not match the published election'
            USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER accept_share_after_closure
BEFORE INSERT ON election_decryption_share
FOR EACH ROW EXECUTE FUNCTION require_closed_election();

CREATE TRIGGER validate_share_manifest
BEFORE INSERT ON election_decryption_share
FOR EACH ROW EXECUTE FUNCTION validate_share_manifest();

CREATE TRIGGER immutable_election_decryption_share
BEFORE UPDATE OR DELETE ON election_decryption_share
FOR EACH ROW EXECUTE FUNCTION reject_election_evidence_mutation();

CREATE TRIGGER accept_result_after_closure
BEFORE INSERT OR UPDATE ON election_tally_result
FOR EACH ROW EXECUTE FUNCTION require_closed_election();

CREATE FUNCTION retain_election_tally_result() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' OR OLD.status <> 'incomplete' OR OLD.election_id <> NEW.election_id THEN
        RAISE EXCEPTION 'final electoral result is immutable' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER retain_election_tally_result
BEFORE UPDATE OR DELETE ON election_tally_result
FOR EACH ROW EXECUTE FUNCTION retain_election_tally_result();

-- No commit, a presença do resultado e sua fase acompanham as contribuições aceitas.
CREATE FUNCTION validate_tally_commit() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    expected bigint;
    received bigint;
    result_status text;
BEGIN
    SELECT count(*) INTO expected FROM election_authority WHERE election_id = NEW.election_id;
    SELECT count(*) INTO received FROM election_decryption_share WHERE election_id = NEW.election_id;
    SELECT status INTO result_status FROM election_tally_result WHERE election_id = NEW.election_id;

    IF received = 0 OR result_status IS NULL
       OR (result_status = 'incomplete') <> (received < expected) THEN
        RAISE EXCEPTION 'tally status does not match accepted shares' USING ERRCODE = '23514';
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER validate_share_commit
AFTER INSERT ON election_decryption_share DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION validate_tally_commit();

CREATE CONSTRAINT TRIGGER validate_tally_commit
AFTER INSERT OR UPDATE ON election_tally_result DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION validate_tally_commit();
