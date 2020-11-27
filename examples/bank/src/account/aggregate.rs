use chekov::prelude::*;

use super::*;

impl Aggregate for Account {
    fn identity() -> &'static str {
        "account"
    }
}

impl CommandExecutor<DeleteAccount> for Account {
    fn execute(cmd: DeleteAccount, _: &Self) -> Result<Vec<AccountDeleted>, CommandExecutorError> {
        Ok(vec![AccountDeleted {
            account_id: cmd.account_id,
        }])
    }
}

impl CommandExecutor<OpenAccount> for Account {
    fn execute(cmd: OpenAccount, state: &Self) -> Result<Vec<AccountOpened>, CommandExecutorError> {
        match state.status {
            AccountStatus::Initialized => Ok(vec![AccountOpened {
                account_id: cmd.account_id,
                name: cmd.name,
            }]),
            _ => Err(CommandExecutorError::Any),
        }
    }
}

impl CommandExecutor<UpdateAccount> for Account {
    fn execute(_cmd: UpdateAccount, _: &Self) -> Result<Vec<AccountUpdated>, CommandExecutorError> {
        Ok(vec![
            AccountUpdated::Deleted,
            AccountUpdated::Forced { why: "duno".into() },
        ])
    }
}

impl EventApplier<AccountOpened> for Account {
    fn apply(&mut self, event: &AccountOpened) -> Result<(), ApplyError> {
        self.account_id = Some(event.account_id);
        self.status = AccountStatus::Active;

        Ok(())
    }
}

impl EventApplier<AccountUpdated> for Account {
    fn apply(&mut self, _event: &AccountUpdated) -> Result<(), ApplyError> {
        Ok(())
    }
}

impl EventApplier<AccountDeleted> for Account {
    fn apply(&mut self, _event: &AccountDeleted) -> Result<(), ApplyError> {
        self.status = AccountStatus::Deleted;

        Ok(())
    }
}
