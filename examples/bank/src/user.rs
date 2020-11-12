use chekov::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, chekov::macros::Aggregate)]
#[aggregate(identity = "user")]
struct User {
    _user_id: Option<Uuid>,
    _account_id: Option<Uuid>,
}

#[derive(Default, chekov::macros::Aggregate)]
#[aggregate(identity = "user-preferences")]
struct UserPreferences {
    _user_id: Option<Uuid>,
    _prefs: Vec<String>,
}

#[derive(Debug, chekov::macros::Command)]
#[command(event = "UserCreated", aggregate = "User")]
struct CreateUser {
    #[command(identifier)]
    user_id: Uuid,
    account_id: Uuid,
}

#[derive(Debug, chekov::macros::Command)]
#[command(event = "PrefUpdated", aggregate = "UserPreferences")]
struct UpdatePref {
    #[command(identifier)]
    user_id: Uuid,
    account_id: Uuid,
}

#[derive(chekov::macros::Event, Deserialize, Serialize)]
struct UserCreated {
    user_id: Uuid,
    account_id: Uuid,
}

#[derive(chekov::macros::Event, Deserialize, Serialize)]
struct PrefUpdated {
    user_id: Uuid,
}

impl CommandExecutor<CreateUser> for User {
    fn execute(cmd: CreateUser, _state: &Self) -> Result<Vec<UserCreated>, CommandExecutorError> {
        Ok(vec![UserCreated {
            user_id: cmd.user_id,
            account_id: cmd.account_id,
        }])
    }
}

impl EventApplier<UserCreated> for User {
    fn apply(&mut self, _event: &UserCreated) -> Result<(), ApplyError> {
        Ok(())
    }
}

impl CommandExecutor<UpdatePref> for UserPreferences {
    fn execute(cmd: UpdatePref, _state: &Self) -> Result<Vec<PrefUpdated>, CommandExecutorError> {
        Ok(vec![PrefUpdated {
            user_id: cmd.user_id,
        }])
    }
}

impl EventApplier<PrefUpdated> for UserPreferences {
    fn apply(&mut self, _event: &PrefUpdated) -> Result<(), ApplyError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chekov::Command;

    #[test]
    fn validate_user_identity() {
        let a = CreateUser {
            user_id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
        };

        assert_eq!(
            format!(
                "{}-{}",
                <CreateUser as Command>::Executor::identity(),
                a.identifier()
            ),
            format!("user-{}", a.user_id)
        );
    }

    #[test]
    fn validate_user_pref_identity() {
        let a = UpdatePref {
            user_id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
        };

        assert_eq!(
            format!(
                "{}-{}",
                <UpdatePref as Command>::Executor::identity(),
                a.identifier()
            ),
            format!("user-preferences-{}", a.user_id)
        );
    }
}
