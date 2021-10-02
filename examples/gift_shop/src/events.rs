use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

pub(crate) mod account {
    use super::*;

    #[derive(Clone, chekov::Event, Deserialize, Serialize)]
    pub struct AccountOpened {
        pub account_id: Uuid,
        pub name: String,
        pub balance: i64,
    }

    #[derive(Clone, chekov::Event, Deserialize, Serialize)]
    #[event(event_type = "MoneyMovement")]
    pub enum MoneyMovementEvent {
        Deposited { account_id: Uuid, amount: u64 },
        Withdrawn { account_id: Uuid, amount: u64 },
    }
}

pub(crate) mod gift_card {
    use super::*;

    #[derive(Clone, chekov::Event, Deserialize, Serialize)]
    pub struct GiftCardCreated {
        pub gift_card_id: Uuid,
        pub name: String,
        pub price: i64,
        pub count: i16,
    }

    #[derive(Clone, chekov::Event, Deserialize, Serialize)]
    pub struct GiftCardUsed {
        pub gift_card_id: Uuid,
        pub account_id: Uuid,
    }
}

pub(crate) mod order {
    use std::collections::HashMap;

    use crate::order::Item;

    use super::*;

    #[derive(Clone, chekov::Event, Deserialize, Serialize)]
    pub struct OrderCreated {
        pub order_id: Uuid,
        pub account_id: Uuid,
    }

    #[derive(Clone, chekov::Event, Deserialize, Serialize)]
    pub struct GiftCardAdded {
        pub order_id: Uuid,
        pub gift_card_id: Uuid,
        pub amount: usize,
        pub price: i64,
    }

    #[derive(Clone, chekov::Event, Deserialize, Serialize)]
    pub struct OrderValidated {
        pub order_id: Uuid,
        pub account_id: Uuid,
        pub items: HashMap<Uuid, Item>,
        pub total_price: i64,
    }

    #[derive(Clone, chekov::Event, Deserialize, Serialize)]
    pub struct OrderCanceled {
        pub order_id: Uuid,
        pub account_id: Uuid,
        pub total_price: i64,
    }
}
