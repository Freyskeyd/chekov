use super::*;

use chekov::event::EventApplier;

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
    fn execute(
        cmd: UpdateAccount,
        state: &Self,
    ) -> Result<Vec<AccountUpdated>, CommandExecutorError> {
        Ok(vec![AccountUpdated::NameChanged(
            state.account_id.unwrap(),
            state.name.clone(),
            cmd.name,
        )])
    }
}

#[chekov::applier]
impl EventApplier<AccountOpened> for Account {
    fn apply(&mut self, event: &AccountOpened) -> Result<(), ApplyError> {
        println!("Account open applied");
        self.account_id = Some(event.account_id);
        self.status = AccountStatus::Active;

        Ok(())
    }
}

#[chekov::applier]
impl EventApplier<AccountUpdated> for Account {
    fn apply(&mut self, event: &AccountUpdated) -> Result<(), ApplyError> {
        if let AccountUpdated::NameChanged(_, _, new_name) = event {
            self.name = new_name.to_string();
        }
        Ok(())
    }
}

#[chekov::applier]
impl EventApplier<AccountDeleted> for Account {
    fn apply(&mut self, _: &AccountDeleted) -> Result<(), ApplyError> {
        self.status = AccountStatus::Deleted;

        Ok(())
    }
}
