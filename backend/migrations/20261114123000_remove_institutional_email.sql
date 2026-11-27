-- O endereço do Graph serve somente como sugestão transitória no cadastro.
-- O e-mail escolhido pelo usuário continua em app_user.email e no Firebase.
ALTER TABLE institutional_verification DROP COLUMN profile_email;
ALTER TABLE app_user DROP COLUMN institutional_email;
