use chekov::aggregate::StaticState;
use chekov::prelude::*;
use futures::future::BoxFuture;
use futures::FutureExt;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::account::*;
use crate::events::*;

#[derive(Debug)]
pub struct DeleteAccount {
    pub account_id: Uuid,
}

impl Command for DeleteAccount {
    type Event = AccountDeleted;
    type Executor = Account;
    type ExecutorRegistry = AggregateInstanceRegistry<Self::Executor>;
    type CommandHandler = NoHandler;

    fn identifier(&self) -> ::std::string::String {
        self.account_id.to_string()
    }
}

#[derive(chekov::Command, Serialize, Deserialize)]
#[command(
    event = "AccountOpened",
    aggregate = "Account",
    handler = "AccountValidator"
)]
pub struct OpenAccount {
    #[command(identifier)]
    pub account_id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, chekov::Command, Serialize, Deserialize)]
#[command(event = "AccountUpdated", aggregate = "Account")]
pub struct UpdateAccount {
    #[command(identifier)]
    pub account_id: Uuid,
    pub name: String,
}

#[derive(Default, CommandHandler)]
pub struct AccountValidator {}

#[chekov::command_handler]
impl chekov::command::Handler<OpenAccount, Account> for AccountValidator {
    fn handle(
        &mut self,
        command: OpenAccount,
        state: StaticState<Account>,
    ) -> BoxFuture<'static, Result<Vec<AccountOpened>, CommandExecutorError>> {
        async move {
            if state.status != AccountStatus::Initialized {
                println!("Can't execute becoze state status is {:?}", state.status);
                Err(CommandExecutorError::Any)
            } else {
                println!("Executing !");
                Account::execute(command, &state)
            }
        }
        .boxed()
    }
}
