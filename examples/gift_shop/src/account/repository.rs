use super::{Account, AccountStatus};
use sqlx::postgres::PgRow;
use sqlx::{Acquire, Row};

use crate::events::*;

pub struct AccountRepository {}

impl AccountRepository {
    pub async fn create(
        account: &account::AccountOpened,
        mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Account, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let todo = sqlx::query(
            "INSERT INTO accounts (account_id, name) VALUES ($1, $2) RETURNING account_id, name, balance",
        )
        .bind(account.account_id)
        .bind(&account.name)
        .map(|row: PgRow| Account {
            account_id: row.get(0),
            name: row.get(1),
            status: AccountStatus::Active,
            balance: row.get::<i64, _>(2),
        })
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(todo)
    }
}
