# Executar com o isolationtester do PostgreSQL em um banco descartável com as migrations.
# Duas sessões tentam registrar envios diferentes da mesma conta na mesma eleição.
setup
{
    INSERT INTO app_user (id, firebase_uid)
    VALUES ('80000000-0000-4000-8000-000000000001', 'participation-isolation-test');

    CREATE FUNCTION test_insert_participation(operation uuid) RETURNS void
    LANGUAGE SQL AS $$
        INSERT INTO election_participation
            (election_id, account_id, voter_id, submission_id, ballot_hash, accepted_at)
        VALUES ('80000000-0000-4000-8000-000000000002', '80000000-0000-4000-8000-000000000001',
                operation, operation, repeat('a', 64), CURRENT_TIMESTAMP)
        ON CONFLICT (election_id, account_id) DO NOTHING;
    $$;

    CREATE FUNCTION test_assert_winner(expected uuid) RETURNS void
    LANGUAGE plpgsql AS $$
    BEGIN
        IF (SELECT array_agg(submission_id) FROM election_participation
            WHERE election_id = '80000000-0000-4000-8000-000000000002')
            IS DISTINCT FROM ARRAY[expected] THEN
            RAISE EXCEPTION 'O banco deveria conter somente o envio %', expected;
        END IF;
    END;
    $$;
}

teardown
{
    DROP FUNCTION test_insert_participation(uuid), test_assert_winner(uuid);
    DELETE FROM election_participation WHERE election_id = '80000000-0000-4000-8000-000000000002';
    DELETE FROM app_user WHERE id = '80000000-0000-4000-8000-000000000001';
}

session first
step begin_first { BEGIN; }
step record_first { SELECT test_insert_participation('80000000-0000-4000-8000-000000000003'); }
step commit_first { COMMIT; }
step rollback_first { ROLLBACK; }

session second
step record_second { SELECT test_insert_participation('80000000-0000-4000-8000-000000000004'); }
step check_first { SELECT test_assert_winner('80000000-0000-4000-8000-000000000003'); }
step check_second { SELECT test_assert_winner('80000000-0000-4000-8000-000000000004'); }

permutation begin_first record_first record_second commit_first check_first
permutation begin_first record_first record_second rollback_first check_second
