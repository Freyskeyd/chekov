use chekov::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::account::*;
use crate::events::*;

#[derive(Debug)]
pub struct DeleteAccount {
    pub account_id: Uuid,
}

#[async_trait::async_trait]
impl Command for DeleteAccount {
    type Event = AccountDeleted;
    type Executor = Account;
    type ExecutorRegistry = AggregateInstanceRegistry<Account>;

    fn identifier(&self) -> ::std::string::String {
        self.account_id.to_string()
    }
    async fn dispatch(&self) -> Result<(), ()> {
        Ok(())
    }
}

#[derive(Clone, Debug, chekov::macros::Command, Serialize, Deserialize)]
#[command(event = "AccountOpened", aggregate = "Account")]
pub struct OpenAccount {
    #[command(identifier)]
    pub account_id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, chekov::macros::Command, Serialize, Deserialize)]
#[command(event = "AccountUpdated", aggregate = "Account")]
pub struct UpdateAccount {
    #[command(identifier)]
    pub account_id: Uuid,
    pub name: String,
}
