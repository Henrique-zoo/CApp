-- Registro privado do aceite. O quadro público usa voter_id para identificar o eleitor.
-- O repositório deve inserir a participação na mesma transação que a cédula e o recibo.
CREATE TABLE election_participation (
    election_id uuid NOT NULL CHECK (election_id <> '00000000-0000-0000-0000-000000000000'),
    account_id uuid NOT NULL REFERENCES app_user (id) ON DELETE RESTRICT,
    voter_id uuid NOT NULL CHECK (voter_id <> '00000000-0000-0000-0000-000000000000'),
    submission_id uuid NOT NULL CHECK (submission_id <> '00000000-0000-0000-0000-000000000000'),
    ballot_hash text NOT NULL CHECK (ballot_hash ~ '^[0-9a-f]{64}$'),
    accepted_at timestamptz NOT NULL,
    PRIMARY KEY (election_id, account_id),
    UNIQUE (election_id, voter_id)
);
