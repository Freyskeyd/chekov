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
    pub status: AccountStatus,
}

impl std::default::Default for Account {
    fn default() -> Self {
        Self {
            account_id: None,
            status: AccountStatus::Initialized,
        }
    }
}

impl Account {
    pub async fn find_all(
        mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Vec<Account>, sqlx::Error> {
        let mut accounts = vec![];
        let recs = sqlx::query(
            r#"
                SELECT account_id
                    FROM accounts
                ORDER BY account_id
            "#,
        )
        .map(|row: PgRow| Account {
            account_id: Some(row.get(0)),
            status: AccountStatus::Initialized,
        })
        .fetch_all(&mut pool)
        .await?;

        Ok(accounts)
    }

    pub async fn create(
        account: &AccountOpened,
        mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Account, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let todo =
            sqlx::query("INSERT INTO accounts (account_id) VALUES ($1) RETURNING account_id")
                .bind(&account.account_id)
                .map(|row: PgRow| Account {
                    account_id: row.get(0),
                    status: AccountStatus::Active,
                })
                .fetch_one(&mut tx)
                .await?;

        tx.commit().await?;
        Ok(todo)
    }

    pub async fn update(
        account: &AccountUpdated,
        mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Account, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let todo = sqlx::query("SELECT * FROM accounts LIMIT 1")
            .map(|row: PgRow| Account {
                account_id: row.get(0),
                status: AccountStatus::Active,
            })
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(todo)
    }
}
