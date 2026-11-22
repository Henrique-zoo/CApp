//! Consultas de aplicação do módulo de demandas.

pub mod get_demand;
pub mod list_demands;
pub mod list_notifications;

pub use get_demand::GetDemandQuery;
pub use list_demands::ListDemandsQuery;
pub use list_notifications::ListDemandNotificationsQuery;
