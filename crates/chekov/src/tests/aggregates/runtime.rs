use super::support::*;
use crate::assert_aggregate_version;
use crate::message::Dispatch;
use crate::prelude::*;
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
async fn can_recover_from_fail_execution() -> Result<(), Box<dyn std::error::Error>> {
    let instance = AggregateInstance {
        inner: ExampleAggregate::default(),
        current_version: 1,
        resolver: ExampleAggregate::get_event_resolver(),
        identity: String::new(),
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
fn can_apply_event() {
    let instance = AggregateInstance {
        inner: ExampleAggregate::default(),
        current_version: 1,
        identity: String::new(),
        resolver: ExampleAggregate::get_event_resolver(),
    };

    let result = AggregateInstance::directly_apply(
        &mut instance.create_mutable_state(),
        &MyEvent { id: Uuid::new_v4() },
    );

    assert!(matches!(result, Ok(_)));
}

#[test(actix::test)]
async fn can_execute_a_command() {
    let instance = AggregateInstance {
        inner: ExampleAggregate::default(),
        current_version: 0,
        resolver: ExampleAggregate::get_event_resolver(),
        identity: String::new(),
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
