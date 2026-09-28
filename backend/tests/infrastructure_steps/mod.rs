//! Steps exclusivos das verificações técnicas do backend.
//!
//! [`readiness`] consulta o router e valida a comunicação com PostgreSQL.
//! [`isolation_audit`] verifica que cenários de escrita usam bancos distintos.
//! Estes módulos são registrados somente no alvo `bdd_infrastructure`.

mod isolation_audit;
mod readiness;
