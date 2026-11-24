-- Fixture determinística para a base-modelo de testes e validação da UnB
-- CAs didáticos necessários para a entrega incremental I01 ("Acesso e meu CA")

INSERT INTO institution (
    id,
    name,
    acronym,
    microsoft_tenant_id,
    microsoft_student_group_id,
    status
) VALUES (
    '018f0000-0000-7000-8000-000000000000',
    'Universidade de Brasília',
    'UnB',
    '018f0000-0000-7000-8000-000000000010',
    '018f0000-0000-7000-8000-000000000020',
    'active'
) ON CONFLICT (microsoft_tenant_id) DO NOTHING;

INSERT INTO academic_center (
    id,
    institution_id,
    name,
    acronym,
    description,
    status,
    created_at,
    updated_at
) VALUES
(
    '018f0000-0000-7000-8000-000000000001',
    '018f0000-0000-7000-8000-000000000000',
    'Centro Acadêmico de Engenharia de Redes',
    'CAER',
    'Centro Acadêmico de Engenharia de Redes da Universidade de Brasília',
    'active',
    TIMESTAMPTZ '2026-01-01 00:00:00+00',
    TIMESTAMPTZ '2026-01-01 00:00:00+00'
),
(
    '018f0000-0000-7000-8000-000000000002',
    '018f0000-0000-7000-8000-000000000000',
    'Centro Acadêmico de Ciência da Computação',
    'CACC',
    'Centro Acadêmico de Ciência da Computação da Universidade de Brasília',
    'active',
    TIMESTAMPTZ '2026-01-01 00:00:00+00',
    TIMESTAMPTZ '2026-01-01 00:00:00+00'
)
ON CONFLICT (id) DO NOTHING;

-- Contas e preferências usadas nos cenários didáticos do CA favorito.
INSERT INTO app_user (
    id, firebase_uid, name, nickname, email, current_semester,
    institution_entry_semester, status, profile_completed_at
) VALUES
    ('018f0000-0000-7000-8000-000000000011', 'capp-bdd-veterano',
     'Estudante Veterano UnB', 'Veterano', '231012345@aluno.unb.br',
     5, '2023.1', 'active', CURRENT_TIMESTAMP),
    ('018f0000-0000-7000-8000-000000000012', 'capp-bdd-veterano-2',
     'Estudante Veterano Secundário UnB', 'Veterano 2', '241098765@aluno.unb.br',
     4, '2024.1', 'active', CURRENT_TIMESTAMP),
    ('018f0000-0000-7000-8000-000000000013', 'capp-bdd-calouro',
     'Estudante Calouro UnB', 'Calouro', '262001234@aluno.unb.br',
     1, '2026.2', 'active', CURRENT_TIMESTAMP)
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_preference (user_id, favorite_ca_id) VALUES
    ('018f0000-0000-7000-8000-000000000011', '018f0000-0000-7000-8000-000000000001'),
    ('018f0000-0000-7000-8000-000000000013', '018f0000-0000-7000-8000-000000000002')
ON CONFLICT (user_id) DO NOTHING;
