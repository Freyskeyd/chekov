use super::{Account, AccountStatus};
use sqlx::postgres::PgRow;
use sqlx::{Acquire, Row};
use uuid::Uuid;

use crate::events::*;

pub struct AccountRepository {}

impl AccountRepository {
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
        .bind(account.account_id)
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
