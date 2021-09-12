use chekov::prelude::*;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::commands::*;
use crate::events::*;
mod aggregate;
mod projector;
mod repository;
pub use aggregate::*;
pub use projector::*;
pub use repository::*;

#[derive(Debug, Clone, Serialize)]
pub enum AccountStatus {
    Initialized,
    Active,
    Deleted,
}

#[derive(Debug, Clone, chekov::Aggregate, Serialize)]
#[aggregate(identity = "account")]
pub struct Account {
    pub account_id: Option<Uuid>,
    pub name: String,
    pub status: AccountStatus,
}

impl std::default::Default for Account {
    fn default() -> Self {
        Self {
            account_id: None,
            name: String::new(),
            status: AccountStatus::Initialized,
        }
    }
}
