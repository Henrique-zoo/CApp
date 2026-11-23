-- A linha do quadro é o lock comum de voto e encerramento. A participação
-- privada da migration anterior é gravada junto do registro público e recibo.
CREATE TABLE election_board (
    election_id uuid PRIMARY KEY CHECK (election_id <> '00000000-0000-0000-0000-000000000000'),
    manifest jsonb NOT NULL CHECK (jsonb_typeof(manifest) = 'object'),
    calendar_origin jsonb NOT NULL CHECK (jsonb_typeof(calendar_origin) = 'object'),
    manifest_hash text NOT NULL CHECK (manifest_hash ~ '^[0-9a-f]{64}$'),
    schedule_hash text NOT NULL CHECK (schedule_hash ~ '^[0-9a-f]{64}$'),
    starts_at timestamptz NOT NULL,
    ends_at timestamptz NOT NULL CHECK (ends_at > starts_at),
    status text NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
    head_hash text NOT NULL CHECK (head_hash ~ '^[0-9a-f]{64}$'),
    voter_count integer NOT NULL DEFAULT 0 CHECK (voter_count BETWEEN 0 AND 100000),
    accumulator jsonb NOT NULL CHECK (jsonb_typeof(accumulator) = 'array'),
    closure jsonb,
    aggregate jsonb,
    CHECK ((status = 'open' AND closure IS NULL AND aggregate IS NULL)
        OR (status = 'closed' AND closure IS NOT NULL AND aggregate IS NOT NULL))
);

ALTER TABLE election_participation
    ADD CONSTRAINT election_participation_board_fk
    FOREIGN KEY (election_id) REFERENCES election_board(election_id) ON DELETE RESTRICT;

CREATE TABLE election_board_entry (
    election_id uuid NOT NULL REFERENCES election_board(election_id) ON DELETE RESTRICT,
    entry_index integer NOT NULL CHECK (entry_index BETWEEN 1 AND 100000),
    voter_id uuid NOT NULL,
    previous_hash text NOT NULL CHECK (previous_hash ~ '^[0-9a-f]{64}$'),
    entry_hash text NOT NULL CHECK (entry_hash ~ '^[0-9a-f]{64}$'),
    signed_ballot jsonb NOT NULL CHECK (jsonb_typeof(signed_ballot) = 'object'),
    signed_entry jsonb NOT NULL CHECK (jsonb_typeof(signed_entry) = 'object'),
    signed_receipt jsonb NOT NULL CHECK (jsonb_typeof(signed_receipt) = 'object'),
    PRIMARY KEY (election_id, entry_index),
    UNIQUE (election_id, voter_id),
    FOREIGN KEY (election_id, voter_id)
        REFERENCES election_participation(election_id, voter_id) ON DELETE RESTRICT
);

-- Proteção dos caminhos normais de escrita. Administradores do banco podem
-- remover triggers: detectar essa adulteração exige as evidências e âncoras externas.
CREATE FUNCTION reject_election_evidence_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'accepted electoral evidence is append-only'
        USING ERRCODE = '23514';
END;
$$;

CREATE TRIGGER immutable_election_board_entry
BEFORE UPDATE OR DELETE ON election_board_entry
FOR EACH ROW EXECUTE FUNCTION reject_election_evidence_mutation();

CREATE TRIGGER immutable_election_participation
BEFORE UPDATE OR DELETE ON election_participation
FOR EACH ROW EXECUTE FUNCTION reject_election_evidence_mutation();

CREATE FUNCTION protect_election_board_origin() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.election_id IS DISTINCT FROM NEW.election_id
       OR OLD.manifest IS DISTINCT FROM NEW.manifest
       OR OLD.calendar_origin IS DISTINCT FROM NEW.calendar_origin
       OR OLD.manifest_hash IS DISTINCT FROM NEW.manifest_hash
       OR OLD.schedule_hash IS DISTINCT FROM NEW.schedule_hash
       OR OLD.starts_at IS DISTINCT FROM NEW.starts_at
       OR OLD.ends_at IS DISTINCT FROM NEW.ends_at
       OR OLD.status = 'closed' THEN
        RAISE EXCEPTION 'frozen electoral origin or closure is immutable'
            USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER frozen_election_board_origin
BEFORE UPDATE ON election_board
FOR EACH ROW EXECUTE FUNCTION protect_election_board_origin();
