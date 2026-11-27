\set ON_ERROR_STOP on
-- Configuração da demonstração com três contas Firebase já provisionadas.
-- Passar election_id, center_id, commission_id, slate_a_account_id,
-- slate_b_account_id, starts_at e ends_at via variáveis de psql.
BEGIN;

INSERT INTO election
    (election_id, academic_center_id, starts_at, ends_at)
VALUES (:'election_id', :'center_id', :'starts_at', :'ends_at');

INSERT INTO election_authority
    (election_id, authority_id, account_id, role, slate_id, slate_name, position)
VALUES
    (:'election_id', gen_random_uuid(), :'commission_id', 'commission', NULL, NULL, NULL),
    (:'election_id', gen_random_uuid(), :'slate_a_account_id', 'slate', gen_random_uuid(), 'Aurora', 1),
    (:'election_id', gen_random_uuid(), :'slate_b_account_id', 'slate', gen_random_uuid(), 'Horizonte', 2);

COMMIT;
