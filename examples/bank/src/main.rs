use event_store::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
struct AccountOpened {
    account_id: Uuid,
}

impl Event for AccountOpened {
    fn event_type(&self) -> &'static str {
        "AccountOpened"
    }
}

#[derive(Serialize)]
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

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // Configure the storage (PG, InMemory,...)
    let storage = PostgresBackend::with_url("postgresql://postgres:postgres@localhost/event_store")
        .await
        .unwrap();
    // Configure the event bus (PG notify, kafka,...)
    // let event_bus = InMemoryEventBus::default();

    let event_store = EventStore::builder()
        .storage(storage)
        // .event_bus(event_bus)
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
    // let uuid2 = Uuid::new_v4().to_string();

    let result = event_store::append()
        .events(&[&account_opened])?
        .event(&money_deposited)?
        .to(&uuid)
        .expected_version(ExpectedVersion::AnyVersion)
        .execute(&event_store)
        .await;

    println!("{:?}", result);

    Ok(())
}
