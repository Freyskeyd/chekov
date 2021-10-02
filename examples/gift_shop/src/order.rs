use std::collections::HashMap;

use chekov::prelude::*;
use futures::future::BoxFuture;
use futures::FutureExt;
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::types::Json;
use sqlx::{Acquire, PgPool, Row};
use uuid::Uuid;

use crate::commands::*;
use crate::events::order::{GiftCardAdded, OrderCanceled, OrderCreated, OrderValidated};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum OrderStatus {
    Unknown,
    Created,
    Canceled,
    Paid,
    Validated,
}

impl Default for OrderStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug, Serialize, serde::Deserialize)]
pub struct Item {
    amount: i64,
    price: i64,
}

#[derive(Default, Debug, Clone, chekov::Aggregate, Serialize)]
#[aggregate(identity = "order")]
pub struct Order {
    pub order_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub status: OrderStatus,
    pub items: HashMap<Uuid, Item>,
    pub total_price: i64,
}

impl CommandExecutor<CreateOrder> for Order {
    fn execute(cmd: CreateOrder, state: &Self) -> ExecutionResult<OrderCreated> {
        if state.status != OrderStatus::Unknown {
            return Err(CommandExecutorError::Any);
        }

        Ok(vec![OrderCreated {
            order_id: cmd.order_id,
            account_id: cmd.account_id,
        }])
    }
}

impl CommandExecutor<CancelOrder> for Order {
    fn execute(cmd: CancelOrder, state: &Self) -> ExecutionResult<OrderCanceled> {
        match state.status {
            OrderStatus::Created | OrderStatus::Validated => Ok(vec![OrderCanceled {
                order_id: cmd.order_id,
                account_id: state.account_id.unwrap(),
                total_price: state.total_price,
            }]),
            OrderStatus::Unknown | OrderStatus::Canceled | OrderStatus::Paid => {
                Err(CommandExecutorError::Any)
            }
        }
    }
}

impl CommandExecutor<ValidateOrder> for Order {
    fn execute(cmd: ValidateOrder, state: &Self) -> ExecutionResult<OrderValidated> {
        if state.status == OrderStatus::Created && !state.items.is_empty() {
            return Ok(vec![OrderValidated {
                order_id: cmd.order_id,
                account_id: state.account_id.unwrap(),
                items: state.items.clone(),
                total_price: state.total_price,
            }]);
        }

        Err(CommandExecutorError::Any)
    }
}

impl CommandExecutor<AddGiftCardToOrder> for Order {
    fn execute(cmd: AddGiftCardToOrder, state: &Self) -> ExecutionResult<GiftCardAdded> {
        if state.status == OrderStatus::Created
            && cmd.price > 0
            && cmd.amount > 0
            && !state.items.iter().any(|(id, _)| id == &cmd.gift_card_id)
        {
            return Ok(vec![GiftCardAdded {
                order_id: cmd.order_id,
                gift_card_id: cmd.gift_card_id,
                amount: cmd.amount,
                price: cmd.price,
            }]);
        }

        Err(CommandExecutorError::Any)
    }
}

#[chekov::applier]
impl EventApplier<OrderCreated> for Order {
    fn apply(&mut self, event: &OrderCreated) -> Result<(), chekov::prelude::ApplyError> {
        self.status = OrderStatus::Created;
        self.order_id = Some(event.order_id);
        self.account_id = Some(event.account_id);

        Ok(())
    }
}

#[chekov::applier]
impl EventApplier<OrderCanceled> for Order {
    fn apply(&mut self, _: &OrderCanceled) -> Result<(), ApplyError> {
        self.status = OrderStatus::Canceled;

        Ok(())
    }
}

#[chekov::applier]
impl EventApplier<GiftCardAdded> for Order {
    fn apply(&mut self, event: &GiftCardAdded) -> Result<(), chekov::prelude::ApplyError> {
        self.items.insert(
            event.gift_card_id,
            Item {
                amount: event.amount as i64,
                price: event.price,
            },
        );

        Ok(())
    }
}

#[chekov::applier]
impl EventApplier<OrderValidated> for Order {
    fn apply(&mut self, _: &OrderValidated) -> Result<(), ApplyError> {
        self.status = OrderStatus::Validated;

        Ok(())
    }
}

#[derive(chekov::EventHandler, Clone)]
pub struct OrderProjector {
    pub pool: PgPool,
}

#[chekov::event_handler]
impl chekov::event::Handler<OrderCreated> for OrderProjector {
    fn handle(&mut self, event: &OrderCreated) -> BoxFuture<Result<(), ()>> {
        let event = event.clone();
        let pool = self.pool.acquire();
        async move {
            let p = pool.await.unwrap();
            let _result = OrderRepository::create(&event, p).await;

            Ok(())
        }
        .boxed()
    }
}

#[chekov::event_handler]
impl chekov::event::Handler<GiftCardAdded> for OrderProjector {
    fn handle(&mut self, event: &GiftCardAdded) -> BoxFuture<Result<(), ()>> {
        let event = event.clone();
        let pool = self.pool.acquire();
        async move {
            let p = pool.await.unwrap();
            let result = OrderRepository::add_gift_card(&event, p).await;

            Ok(())
        }
        .boxed()
    }
}

pub struct OrderRepository {}

impl OrderRepository {
    pub async fn create(
        entity: &OrderCreated,
        mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Order, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let todo = sqlx::query(
            "INSERT INTO orders (order_id, account_id) VALUES ($1, $2) RETURNING order_id, account_id, items, total_price",
        )
        .bind(&entity.order_id)
        .bind(&entity.account_id)
        .map(|row: PgRow| Order {
            order_id: row.get(0),
            account_id: row.get(1),
            items: row.get::<Json<HashMap<Uuid, Item>>, _>(2).0,
            total_price: row.get(3),
            status: OrderStatus::Created
        })
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(todo)
    }

    pub async fn add_gift_card(
        entity: &GiftCardAdded,
        mut pool: sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Order, sqlx::Error> {
        let json = format!(
            r#"{{"{}": {{"amount": {}, "price": {}}}}}"#,
            entity.gift_card_id, entity.amount, entity.price
        );

        let mut tx = pool.begin().await?;
        let todo = sqlx::query(
            r#"UPDATE orders SET items = items::jsonb || $1::jsonb WHERE order_id = $2 RETURNING order_id, account_id, items, total_price"#
        )
            .bind(&json)
            .bind(entity.order_id)
            .map(|row: PgRow| Order {
                order_id: row.get(0),
                account_id: row.get(1),
                items: row.get::<Json<HashMap<Uuid, Item>>, _>(2).0,
                total_price: row.get(3),
                status: OrderStatus::Created
            })
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(todo)
    }
}
