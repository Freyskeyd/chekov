use super::support::*;
use crate::assert_aggregate_version;
use crate::event_store::EventStore;
use crate::message::Dispatch;
use crate::prelude::*;
use event_store::prelude::Appender;
use std::marker::PhantomData;
use test_log::test;
use uuid::Uuid;

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
fn can_duplicate_state() {
    let instance = AggregateInstance {
        inner: ExampleAggregate::default(),
        current_version: 0,
        resolver: ExampleAggregate::get_event_resolver(),
        identity: String::new(),
    };

    let _: ExampleAggregate = instance.create_mutable_state();
}
