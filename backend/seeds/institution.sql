INSERT INTO institution (
    name,
    acronym,
    microsoft_tenant_id,
    microsoft_student_group_id
)
VALUES (
    :'institution_name',
    :'acronym',
    :'tenant_id'::uuid,
    :'student_group_id'::uuid
)
ON CONFLICT (microsoft_tenant_id) DO NOTHING;