-- Fixture determinística para a base-modelo de testes e validação da UnB
-- CAs didáticos e dados de identidade necessários para a entrega incremental I01 ("Acesso e meu CA")

INSERT INTO academic_center (
    id,
    name,
    acronym,
    description,
    status,
    created_at,
    updated_at
) VALUES
(
    '018f0000-0000-7000-8000-000000000001',
    'Centro Acadêmico de Engenharia de Redes',
    'CAER',
    'Centro Acadêmico de Engenharia de Redes da Universidade de Brasília',
    'active',
    TIMESTAMPTZ '2026-01-01 00:00:00+00',
    TIMESTAMPTZ '2026-01-01 00:00:00+00'
),
(
    '018f0000-0000-7000-8000-000000000002',
    'Centro Acadêmico de Ciência da Computação',
    'CACC',
    'Centro Acadêmico de Ciência da Computação da Universidade de Brasília',
    'active',
    TIMESTAMPTZ '2026-01-01 00:00:00+00',
    TIMESTAMPTZ '2026-01-01 00:00:00+00'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO app_user (
    id,
    display_name,
    institutional_email,
    status,
    created_at,
    updated_at
) VALUES
(
    '018f0000-0000-7000-8000-000000000011',
    'Estudante Veterano UnB',
    '231012345@aluno.unb.br',
    'active',
    TIMESTAMPTZ '2026-01-01 00:00:00+00',
    TIMESTAMPTZ '2026-01-01 00:00:00+00'
),
(
    '018f0000-0000-7000-8000-000000000012',
    'Estudante Veterano 2 UnB',
    '241098765@aluno.unb.br',
    'active',
    TIMESTAMPTZ '2026-01-01 00:00:00+00',
    TIMESTAMPTZ '2026-01-01 00:00:00+00'
),
(
    '018f0000-0000-7000-8000-000000000013',
    'Estudante Calouro UnB',
    '262001234@aluno.unb.br',
    'active',
    TIMESTAMPTZ '2026-01-01 00:00:00+00',
    TIMESTAMPTZ '2026-01-01 00:00:00+00'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_preference (
    user_id,
    favorite_ca_id,
    updated_at
) VALUES
(
    '018f0000-0000-7000-8000-000000000011',
    '018f0000-0000-7000-8000-000000000001',
    TIMESTAMPTZ '2026-01-01 00:00:00+00'
),
(
    '018f0000-0000-7000-8000-000000000013',
    '018f0000-0000-7000-8000-000000000002',
    TIMESTAMPTZ '2026-01-01 00:00:00+00'
)
ON CONFLICT (user_id) DO NOTHING;
