use chekov::prelude::*;
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{Acquire, PgPool, Row};
use uuid::Uuid;

use crate::commands::*;
use crate::events::*;
mod aggregate;
mod projector;
pub use aggregate::*;
pub use projector::*;

#[derive(Serialize)]
pub enum AccountStatus {
    Initialized,
    Active,
    Deleted,
}

#[derive(chekov::macros::Aggregate, Serialize)]
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

impl Account {
    pub async fn find_all(
        mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Vec<Account>, sqlx::Error> {
        sqlx::query(
            r#"
                SELECT account_id, name
                    FROM accounts
                ORDER BY account_id
            "#,
        )
        .map(|row: PgRow| Account {
            account_id: Some(row.get(0)),
            name: row.get(1),
            status: AccountStatus::Initialized,
        })
        .fetch_all(&mut pool)
        .await
    }

    pub async fn create(
        account: &AccountOpened,
        mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Account, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let todo = sqlx::query(
            "INSERT INTO accounts (account_id, name) VALUES ($1, $2) RETURNING account_id, name",
        )
        .bind(&account.account_id)
        .bind(&account.name)
        .map(|row: PgRow| Account {
            account_id: row.get(0),
            name: row.get(1),
            status: AccountStatus::Active,
        })
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(todo)
    }

    pub async fn update(
        account_id: &Uuid,
        name: &str,
        mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Account, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let todo = sqlx::query(
            "UPDATE accounts SET name = $1 WHERE account_id = $2 RETURNING account_id, name",
        )
        .bind(name)
        .bind(account_id)
        .map(|row: PgRow| Account {
            account_id: row.get(0),
            name: row.get(1),
            status: AccountStatus::Active,
        })
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(todo)
    }
}
