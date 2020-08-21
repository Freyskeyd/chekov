use event_store::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use tokio::stream::StreamExt;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
struct AccountOpened {
    account_id: Uuid,
}

impl Event for AccountOpened {
    fn event_type(&self) -> &'static str {
        "AccountOpened"
    }
}

#[derive(Deserialize, Serialize)]
enum MoneyMovementEvent {
    MoneyDeposited { account_id: Uuid },
    // MoneyWithdrawn { account_id: Uuid },
}

impl Event for MoneyMovementEvent {
    fn event_type(&self) -> &'static str {
        match *self {
            // MoneyMovementEvent::MoneyWithdrawn { .. } => "MoneyMovementEvent::MoneyWithdrawn",
            MoneyMovementEvent::MoneyDeposited { .. } => "MoneyMovementEvent::MoneyDeposited",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct UserRegistered {
    email: String,
    hashed_password: String,
    user_uuid: Uuid,
    username: String,
}

impl Event for UserRegistered {
    fn event_type(&self) -> &'static str {
        "Elixir.Conduit.Accounts.Events.UserRegistered"
    }
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // Configure the storage (PG, InMemory,...)
    let storage = PostgresBackend::with_url("postgresql://postgres:postgres@localhost/event_store")
        .await
        .unwrap();

    let event_store = EventStore::builder()
        .storage(storage)
        .build()
        .await
        .unwrap();

    let account_opened = AccountOpened {
        account_id: Uuid::new_v4(),
    };

    let money_deposited = MoneyMovementEvent::MoneyDeposited {
        account_id: Uuid::new_v4(),
    };

    let uuid = Uuid::new_v4().to_string();

    let result = event_store::append()
        .events(&[&account_opened])?
        .event(&money_deposited)?
        .to(&uuid)?
        .expected_version(ExpectedVersion::AnyVersion)
        .execute(&event_store)
        .await;

    println!("{:?}", result);

    let result = event_store::read()
        .stream(&uuid)?
        .from(ReadVersion::Origin)
        .limit(10)
        .execute(&event_store)
        .await?;

    let _ = result
        .first()
        .unwrap()
        .try_deserialize::<AccountOpened>()
        .unwrap();

    let mut stream = event_store::read()
        .stream(&uuid)?
        .from(ReadVersion::Origin)
        .limit(2)
        .into_stream()
        .chunk_by(1)
        .execute(&event_store)
        .await;

    // while let Some(notification) = stream.try_next().await? {
    while let Some(notification) = stream.try_next().await? {
        println!("[from stream]: {:?}", notification);
    }

    Ok(())
}
