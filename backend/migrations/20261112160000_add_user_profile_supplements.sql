-- Dados declarados pelo usuário para ausências do Graph. Não são evidência
-- institucional e não modificam o vínculo ou a fotografia verificada.
ALTER TABLE app_user
    ADD COLUMN institutional_email TEXT,
    ADD COLUMN course_name TEXT;
COMMENT ON COLUMN app_user.name IS 'Nome declarado somente quando ausente no Graph';
COMMENT ON COLUMN app_user.institutional_email IS 'Contato declarado, sem comprovação de titularidade pelo Graph';
COMMENT ON COLUMN app_user.course_name IS 'Curso declarado, sem comprovação pelo Graph';
