use chekov::prelude::*;
use futures::future::BoxFuture;
use futures::FutureExt;
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{Acquire, PgPool, Row};
use uuid::Uuid;

use crate::commands::*;
use crate::events::gift_card::GiftCardCreated;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum GiftCardState {
    Unknown,
    Created,
}

#[derive(Default, Debug, Clone, chekov::Aggregate, Serialize)]
#[aggregate(identity = "gift_card")]
pub struct GiftCard {
    pub gift_card_id: Option<Uuid>,
    pub name: String,
    pub price: i64,
    pub count: i32,
    pub gift_card_state: GiftCardState,
}

impl Default for GiftCardState {
    fn default() -> Self {
        Self::Unknown
    }
}

impl CommandExecutor<CreateGiftCard> for GiftCard {
    fn execute(cmd: CreateGiftCard, state: &Self) -> ExecutionResult<GiftCardCreated> {
        let events = if state.gift_card_state == GiftCardState::Unknown {
            vec![GiftCardCreated {
                gift_card_id: cmd.gift_card_id,
                name: cmd.name,
                price: cmd.price,
                count: cmd.count as i16,
            }]
        } else {
            vec![]
        };

        Ok(events)
    }
}

#[chekov::applier]
impl EventApplier<GiftCardCreated> for GiftCard {
    fn apply(&mut self, event: &GiftCardCreated) -> Result<(), ApplyError> {
        self.gift_card_state = GiftCardState::Created;
        self.gift_card_id = Some(event.gift_card_id);
        self.name = event.name.clone();
        self.price = event.price;
        self.count = event.count as i32;

        Ok(())
    }
}

#[derive(chekov::EventHandler, Clone)]
pub struct GiftCardProjector {
    pub pool: PgPool,
}

#[chekov::event_handler]
impl chekov::event::Handler<GiftCardCreated> for GiftCardProjector {
    fn handle(&mut self, event: &GiftCardCreated) -> BoxFuture<Result<(), ()>> {
        let event = event.clone();
        let pool = self.pool.acquire();
        async move {
            let p = pool.await.unwrap();
            let _result = GiftCardRepository::create(&event, p).await;

            Ok(())
        }
        .boxed()
    }
}

pub struct GiftCardRepository {}

impl GiftCardRepository {
    // pub async fn find_all(
    //     mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    // ) -> Result<Vec<GiftCard>, sqlx::Error> {
    //     sqlx::query(
    //         r#"
    //             SELECT account_id, name, balance
    //                 FROM accounts
    //             ORDER BY account_id
    //         "#,
    //     )
    //     .map(|row: PgRow| GiftCard {
    //         account_id: Some(row.get(0)),
    //         name: row.get(1),
    //         status: GiftCardStatus::Initialized,
    //         balance: row.get(2),
    //     })
    //     .fetch_all(&mut pool)
    //     .await
    // }

    pub async fn create(
        entity: &GiftCardCreated,
        mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<GiftCard, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let todo = sqlx::query(
            "INSERT INTO gift_cards (gift_card_id, name, price, count) VALUES ($1, $2, $3, $4) RETURNING gift_card_id, name, price, count",
        )
        .bind(&entity.gift_card_id)
        .bind(&entity.name)
        .bind(&entity.price)
        .bind(&entity.count)
        .map(|row: PgRow| GiftCard {
            gift_card_id: row.get(0),
            name: row.get(1),
            price: row.get::<i64, _>(2),
            count: row.get::<i32, _>(3),
            gift_card_state: GiftCardState::Created
        })
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(todo)
    }

    //     pub async fn update(
    //         account_id: &Uuid,
    //         name: &str,
    //         mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    //     ) -> Result<GiftCard, sqlx::Error> {
    //         let mut tx = pool.begin().await?;
    //         let todo = sqlx::query(
    //             "UPDATE accounts SET name = $1 WHERE account_id = $2 RETURNING account_id, name, balance",
    //         )
    //         .bind(name)
    //         .bind(account_id)
    //         .map(|row: PgRow| GiftCard {
    //             account_id: row.get(0),
    //             name: row.get(1),
    //             status: GiftCardStatus::Active,
    //             balance: row.get(2),
    //         })
    //         .fetch_one(&mut tx)
    //         .await?;

    //         tx.commit().await?;
    //         Ok(todo)
    //     }
}
