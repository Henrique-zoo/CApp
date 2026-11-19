-- author_id / reviewer_id / user_id nasceram nas tabelas de eventos antes de app_user existir.
-- As referências entram aqui, após 20261111210000_provision_internal_users.
ALTER TABLE event
    ADD CONSTRAINT event_author_id_fkey
    FOREIGN KEY (author_id) REFERENCES app_user (id) ON DELETE RESTRICT;

ALTER TABLE event_review
    ADD CONSTRAINT event_review_reviewer_id_fkey
    FOREIGN KEY (reviewer_id) REFERENCES app_user (id) ON DELETE RESTRICT;

ALTER TABLE event_registration
    ADD CONSTRAINT event_registration_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES app_user (id) ON DELETE RESTRICT;
