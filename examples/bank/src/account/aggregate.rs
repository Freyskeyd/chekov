use std::time::Duration;

use super::*;
use crate::DefaultApp;

chekov::macros::apply_event!(DefaultApp, Account, AccountUpdated, apply_account_updated);
chekov::macros::apply_event!(DefaultApp, Account, AccountDeleted, apply_account_deleted);
chekov::macros::apply_event!(DefaultApp, Account, AccountOpened, apply_account_open);

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
            state.account_id.unwrap().clone(),
            state.name.clone(),
            cmd.name.clone(),
        )])
    }
}

fn apply_account_open(state: &mut Account, event: &AccountOpened) -> Result<(), ApplyError> {
    state.account_id = Some(event.account_id);
    state.status = AccountStatus::Active;

    Ok(())
}

fn apply_account_updated(state: &mut Account, event: &AccountUpdated) -> Result<(), ApplyError> {
    match event {
        AccountUpdated::NameChanged(_, _, new_name) => {
            state.name = new_name.to_string();
        }
        _ => {}
    }
    Ok(())
}

fn apply_account_deleted(state: &mut Account, _event: &AccountDeleted) -> Result<(), ApplyError> {
    state.status = AccountStatus::Deleted;

    Ok(())
}
