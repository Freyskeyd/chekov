use super::*;
use crate::DefaultApp;
use actix::prelude::*;
use actix::Context;

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
    fn execute(_cmd: UpdateAccount, _: &Self) -> Result<Vec<AccountUpdated>, CommandExecutorError> {
        Ok(vec![
            AccountUpdated::Deleted,
            AccountUpdated::Forced { why: "duno".into() },
        ])
    }
}

fn apply_account_open(state: &mut Account, event: &AccountOpened) -> Result<(), ApplyError> {
    println!("Applying AccountOpened");
    state.account_id = Some(event.account_id);
    state.status = AccountStatus::Active;

    Ok(())
}
fn apply_account_updated(_state: &mut Account, _event: &AccountUpdated) -> Result<(), ApplyError> {
    println!("Applying AccountUpdated");
    Ok(())
}

fn apply_account_deleted(state: &mut Account, _event: &AccountDeleted) -> Result<(), ApplyError> {
    println!("Applying AccountDeleted");
    state.status = AccountStatus::Deleted;

    Ok(())
}
