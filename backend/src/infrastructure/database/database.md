# Infraestrutura de banco de dados

O módulo expõe `MIGRATOR`, a fonte de migrations SQLx compartilhada entre o
servidor HTTP e a preparação dos bancos de teste. Cada ambiente continua
responsável pela criação e pelo ciclo de vida de seu pool PostgreSQL.

As migrations de domínio serão adicionadas após a definição do modelo de dados.
O fluxo de criação, aplicação e versionamento está no
[guia de migrations do backend](../../../README.md#migrations).
Os contratos do migrador são documentados em `mod.rs`.
