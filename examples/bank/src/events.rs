use actix::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, chekov::macros::Event, Deserialize, Serialize)]
pub struct AccountDeleted {
    pub account_id: Uuid,
}

#[derive(Clone, Message, Debug, chekov::macros::Event, Deserialize, Serialize)]
#[rtype(result = "()")]
pub struct AccountOpened {
    pub account_id: Uuid,
    pub name: String,
}

#[derive(Clone, Message, Debug, chekov::macros::Event, Deserialize, Serialize)]
#[rtype(result = "()")]
pub enum AccountUpdated {
    Deleted,
    Forced { why: String },
    Disabled(String),
}

#[derive(chekov::macros::Event, Debug, Deserialize, Serialize)]
#[event(event_type = "Elixir.Conduit.Accounts.Events.UserRegistered")]
pub struct UserRegistered {
    pub email: String,
    pub hashed_password: String,
    pub user_uuid: Uuid,
    pub username: String,
}

#[derive(chekov::macros::Event, Deserialize, Serialize)]
#[event(event_type = "MoneyMovement")]
pub enum MoneyMovementEvent {
    Deposited {
        account_id: Uuid,
    },
    Withdrawn {
        account_id: Uuid,
    },
    #[event(event_type = "MoneyDeleted")]
    Removed,
    Added(String),
}
