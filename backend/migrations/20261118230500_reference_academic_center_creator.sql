-- created_by_id nasceu na tabela do CA antes de app_user existir.
-- A referência entra aqui, com a coluna ainda opcional: o recorte I01 persiste CAs sem autor.
ALTER TABLE academic_center
    ADD CONSTRAINT academic_center_created_by_id_fkey
    FOREIGN KEY (created_by_id) REFERENCES app_user (id) ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS idx_academic_center_created_by_id
    ON academic_center (created_by_id);
