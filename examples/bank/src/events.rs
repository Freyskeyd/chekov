use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, chekov::Event, Deserialize, Serialize)]
pub struct AccountDeleted {
    pub account_id: Uuid,
}

#[derive(Clone, chekov::Event, Deserialize, Serialize)]
pub struct AccountOpened {
    pub account_id: Uuid,
    pub name: String,
}

#[derive(Clone, chekov::Event, Deserialize, Serialize)]
pub enum AccountUpdated {
    NameChanged(Uuid, String, String),
    Deleted,
    Forced { why: String },
    Disabled(String),
}

#[derive(Clone, chekov::Event, Deserialize, Serialize)]
#[event(event_type = "Elixir.Conduit.Accounts.Events.UserRegistered")]
pub struct UserRegistered {
    pub email: String,
    pub hashed_password: String,
    pub user_uuid: Uuid,
    pub username: String,
}

#[derive(Clone, chekov::Event, Deserialize, Serialize)]
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
