use super::support::*;
use crate::aggregate::AggregateInstance;
use crate::assert_aggregate_version;
use crate::event_store::EventStore;
use crate::message::Dispatch;
use crate::prelude::*;
use actix::Addr;
use event_store::prelude::Appender;
use std::marker::PhantomData;
use test_log::test;
use uuid::Uuid;

#[test(actix::test)]
async fn should_be_able_to_start() -> Result<(), Box<dyn std::error::Error>> {
    let identifier = Uuid::new_v4();
    start_application().await;
    let instance = start_aggregate(&identifier).await;

    assert_aggregate_version!(instance, 0);

    let _ = instance
        .send(Dispatch::<_, MyApplication> {
            metadatas: CommandMetadatas::default(),
            storage: PhantomData,
            command: AppendItem(1, identifier),
        })
        .await;

    assert_aggregate_version!(instance, 1);

    Ok(())
}

#[actix::test]
async fn should_rebuild_his_state_from_previously_append_events(
) -> Result<(), Box<dyn std::error::Error>> {
    start_application().await;

    let identifier = Uuid::new_v4();
    let _ = EventStore::<MyApplication>::with_appender(
        Appender::default()
            .event(&ItemAppended(1))
            .unwrap()
            .to(&identifier)
            .unwrap(),
    )
    .await;

    let instance = start_aggregate(&identifier).await;

    assert_aggregate_version!(instance, 1);

    let result = instance
        .send(Dispatch::<_, MyApplication> {
            metadatas: CommandMetadatas::default(),
            storage: PhantomData,
            command: AppendItem(1, identifier),
        })
        .await;

    assert!(result.is_ok());

    assert_aggregate_version!(instance, 2);

    Ok(())
}

#[actix::test]
async fn should_can_fetch_existing_state() -> Result<(), Box<dyn std::error::Error>> {
    start_application().await;
    let identifier = Uuid::new_v4();
    let _ = EventStore::<MyApplication>::with_appender(
        Appender::default()
            .event(&MyEvent { id: identifier })
            .unwrap()
            .to(&identifier)
            .unwrap(),
    )
    .await;

    let result = AggregateInstance::<ExampleAggregate>::fetch_existing_state::<MyApplication>(
        identifier.to_string(),
        Uuid::new_v4(),
    )
    .await;

    assert_eq!(result.expect("shouldn't fail").len(), 1);
    Ok(())
}

#[test]
fn can_apply_event() {
    let instance = AggregateInstance {
        inner: ExampleAggregate::default(),
        current_version: 1,
        resolver: ExampleAggregate::get_event_resolver(),
    };

    let result = AggregateInstance::directly_apply(
        &mut instance.create_mutable_state(),
        &MyEvent { id: Uuid::new_v4() },
    );

    assert!(matches!(result, Ok(_)));
}

#[actix::test]
async fn can_recover_from_fail_execution() -> Result<(), Box<dyn std::error::Error>> {
    let instance = AggregateInstance {
        inner: ExampleAggregate::default(),
        current_version: 1,
        resolver: ExampleAggregate::get_event_resolver(),
    };

    let result = AggregateInstance::execute(
        instance.create_mutable_state(),
        Dispatch::<_, MyApplication> {
            storage: PhantomData,
            command: InvalidCommand(Uuid::new_v4()),
            metadatas: CommandMetadatas::default(),
        },
    )
    .await;

    assert!(matches!(result, Err(_)));

    let result = AggregateInstance::execute(
        instance.create_mutable_state(),
        Dispatch::<_, MyApplication> {
            storage: PhantomData,
            command: ValidCommand(Uuid::new_v4()),
            metadatas: CommandMetadatas::default(),
        },
    )
    .await;

    assert!(matches!(result, Ok(_)));
    Ok(())
}

#[test]
fn can_duplicate_state() {
    let instance = AggregateInstance {
        inner: ExampleAggregate::default(),
        current_version: 0,
        resolver: ExampleAggregate::get_event_resolver(),
    };

    let _: ExampleAggregate = instance.create_mutable_state();
}

#[test(actix::test)]
async fn can_execute_a_command() {
    let instance = AggregateInstance {
        inner: ExampleAggregate::default(),
        current_version: 0,
        resolver: ExampleAggregate::get_event_resolver(),
    };
    let id = Uuid::new_v4();

    let result: Result<_, CommandExecutorError> = AggregateInstance::execute(
        instance.create_mutable_state(),
        Dispatch::<_, MyApplication> {
            storage: PhantomData,
            command: ValidCommand(id),
            metadatas: CommandMetadatas::default(),
        },
    )
    .await
    .map(|(value, _)| value);

    let expected = vec![MyEvent { id: id }];

    assert!(matches!(result, Ok(events) if events == expected));
}

#[allow(dead_code)]
async fn start_context(identity: &Uuid) -> Addr<AggregateInstance<ExampleAggregate>> {
    start_application().await;
    start_aggregate(identity).await
}

async fn start_application() {
    MyApplication::with_default()
        .storage(event_store::storage::InMemoryStorage::initiate())
        .launch()
        .await;
}

async fn start_aggregate(identity: &Uuid) -> Addr<AggregateInstance<ExampleAggregate>> {
    let correlation_id = Uuid::new_v4();

    AggregateInstance::<ExampleAggregate>::new::<MyApplication>(
        identity.to_string(),
        correlation_id,
    )
    .await
    .unwrap()
}
