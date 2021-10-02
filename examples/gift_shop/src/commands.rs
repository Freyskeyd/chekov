use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::account::*;
use crate::events::*;
use crate::gift_card::*;
use crate::order::*;

#[derive(chekov::Command, Serialize, Deserialize)]
#[command(event = "account::AccountOpened", aggregate = "Account")]
pub struct OpenAccount {
    #[command(identifier)]
    pub account_id: Uuid,
    pub name: String,
}

#[derive(chekov::Command, Serialize, Deserialize)]
#[command(event = "gift_card::GiftCardCreated", aggregate = "GiftCard")]
pub struct CreateGiftCard {
    #[command(identifier)]
    pub gift_card_id: Uuid,
    pub name: String,
    pub price: i64,
    pub count: usize,
}

#[derive(Clone, chekov::Command, Deserialize, Serialize)]
#[command(event = "order::OrderCreated", aggregate = "Order")]
pub struct CreateOrder {
    #[command(identifier)]
    pub order_id: Uuid,
    pub account_id: Uuid,
}

#[derive(Clone, chekov::Command, Deserialize, Serialize)]
#[command(event = "order::GiftCardAdded", aggregate = "Order")]
pub struct AddGiftCardToOrder {
    #[command(identifier)]
    pub order_id: Uuid,
    pub gift_card_id: Uuid,
    pub amount: usize,
    pub price: i64,
}

#[derive(Clone, chekov::Command, Deserialize, Serialize)]
#[command(event = "order::OrderValidated", aggregate = "Order")]
pub struct ValidateOrder {
    #[command(identifier)]
    pub order_id: Uuid,
}

#[derive(Clone, chekov::Command, Deserialize, Serialize)]
#[command(event = "order::OrderCanceled", aggregate = "Order")]
pub struct CancelOrder {
    #[command(identifier)]
    pub order_id: Uuid,
}
