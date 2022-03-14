use std::{error::Error, fmt};

use super::*;
use crate::events::account::*;
use chekov::event::EventApplier;

#[derive(Debug)]
enum AccountError {
    UnableToCreate,
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountError::UnableToCreate => write!(f, "Can't open account"),
        }
    }
}

impl Error for AccountError {}

impl CommandExecutor<OpenAccount> for Account {
    fn execute(cmd: OpenAccount, state: &Self) -> Result<Vec<AccountOpened>, CommandExecutorError> {
        match state.status {
            AccountStatus::Initialized => Ok(vec![AccountOpened {
                account_id: cmd.account_id,
                name: cmd.name,
                balance: 0,
            }]),
            _ => Err(CommandExecutorError::ExecutionError(Box::new(
                AccountError::UnableToCreate,
            ))),
        }
    }
}

#[chekov::applier]
impl EventApplier<AccountOpened> for Account {
    fn apply(&mut self, event: &AccountOpened) -> Result<(), ApplyError> {
        self.account_id = Some(event.account_id);
        self.status = AccountStatus::Active;

        Ok(())
    }
}
