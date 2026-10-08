-- Fixture determinística para a base-modelo de testes e validação da UnB
-- CAs didáticos necessários para a entrega incremental I01 ("Acesso e meu CA")

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
