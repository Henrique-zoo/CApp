-- Curso é evidência exclusivamente institucional. Complementos declarados não
-- podem sobreviver como alternativa ao curso obtido pelo Graph.
ALTER TABLE app_user DROP COLUMN course_name;

-- Verificações anteriores podiam confirmar o grupo sem obter um curso. Elas
-- continuam válidas para entrada limitada, sem direitos estudantis.
UPDATE institutional_verification
SET student_membership_confirmed = FALSE, course_name = NULL
WHERE NOT student_membership_confirmed
   OR course_name IS NULL
   OR course_name !~ '[^[:space:]]';

ALTER TABLE institutional_verification
    ADD CONSTRAINT verified_student_requires_course CHECK (
        (student_membership_confirmed AND course_name IS NOT NULL
            AND course_name ~ '[^[:space:]]')
        OR (NOT student_membership_confirmed AND course_name IS NULL)
    );

COMMENT ON COLUMN institutional_verification.student_membership_confirmed
    IS 'Grupo estudantil e curso não vazio confirmados pelo Graph';
