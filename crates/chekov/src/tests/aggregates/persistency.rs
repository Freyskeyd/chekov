use super::support::*;
use crate::event_store::EventStore;
use crate::prelude::*;
use crate::{assert_aggregate_state, assert_aggregate_version};
use event_store::prelude::Reader;
use test_log::test;
use uuid::Uuid;

#[test(actix::test)]
async fn should_persist_pending_events_in_order_applied() -> Result<(), Box<dyn std::error::Error>>
{
    let identifier = Uuid::new_v4();
    start_application().await;
    let _ = start_aggregate(&identifier).await;

    let result = AggregateInstanceRegistry::<ExampleAggregate>::execute::<MyApplication, _>(
        AppendItem(10, identifier),
    )
    .await;

    assert!(result.is_ok());

    let events = result.unwrap();
    assert!(events.len() == 10);

    let reader = Reader::default().stream(identifier)?.limit(100);
    let recorded_events = EventStore::<MyApplication>::with_reader(reader).await??;

    assert!(recorded_events.len() == 10);

    let identifier_as_string = identifier.to_string();

    let slice = recorded_events
        .into_iter()
        .filter(|event| event.stream_uuid == identifier_as_string)
        .map(|event| serde_json::from_value::<usize>(event.data).unwrap())
        .collect::<Vec<usize>>();

    assert!(slice == (1..=10).collect::<Vec<usize>>());

    Ok(())
}

#[test(actix::test)]
async fn should_not_persist_events_when_command_returns_no_events(
) -> Result<(), Box<dyn std::error::Error>> {
    let identifier = Uuid::new_v4();
    start_application().await;
    let _ = start_aggregate(&identifier).await;

    let result = AggregateInstanceRegistry::<ExampleAggregate>::execute::<MyApplication, _>(
        AppendItem(0, identifier),
    )
    .await;

    assert!(result.is_ok());

    let events = result.unwrap();
    assert!(events.is_empty());

    let reader = Reader::default().stream(identifier)?.limit(100);
    let recorded_events = EventStore::<MyApplication>::with_reader(reader).await??;

    assert!(recorded_events.is_empty());

    Ok(())
}

#[test(actix::test)]
async fn should_persist_event_metadata() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement metadata persistency
    Ok(())
}

#[test(actix::test)]
async fn should_reload_persisted_events_when_restarting_aggregate_process(
) -> Result<(), Box<dyn std::error::Error>> {
    let identifier = Uuid::new_v4();
    start_application().await;
    let addr = start_aggregate(&identifier).await;

    let result = AggregateInstanceRegistry::<ExampleAggregate>::execute::<MyApplication, _>(
        AppendItem(10, identifier),
    )
    .await;

    assert!(result.is_ok());

    let events = result.unwrap();
    assert!(events.len() == 10);

    let reader = Reader::default().stream(identifier)?.limit(100);
    let recorded_events = EventStore::<MyApplication>::with_reader(reader).await??;

    assert!(recorded_events.len() == 10);

    let res =
        AggregateInstanceRegistry::<ExampleAggregate>::shutdown_aggregate(identifier.to_string())
            .await;

    assert!(res.is_ok(), "Aggregate couldn't be shutdown");
    assert!(!addr.connected());

    let addr_after = start_aggregate(&identifier).await;

    assert_aggregate_version!(&addr_after, 10);
    assert_aggregate_state!(
        &addr_after,
        ExampleAggregate {
            items: (1..=10).collect(),
            last_index: 9
        }
    );

    Ok(())
}

#[test(actix::test)]
async fn should_reload_persisted_events_in_batches_when_restarting_aggregate_process(
) -> Result<(), Box<dyn std::error::Error>> {
    let identifier = Uuid::new_v4();
    start_application().await;
    let addr = start_aggregate(&identifier).await;

    let result = AggregateInstanceRegistry::<ExampleAggregate>::execute::<MyApplication, _>(
        AppendItem(300, identifier),
    )
    .await;

    assert!(result.is_ok());

    let events = result.unwrap();
    assert!(events.len() == 300);

    let res =
        AggregateInstanceRegistry::<ExampleAggregate>::shutdown_aggregate(identifier.to_string())
            .await;

    assert!(res.is_ok(), "Aggregate couldn't be shutdown");
    assert!(!addr.connected());

    let addr_after = start_aggregate(&identifier).await;

    assert_aggregate_version!(&addr_after, 300);
    assert_aggregate_state!(
        &addr_after,
        ExampleAggregate {
            items: (1..=300).collect(),
            last_index: 299
        }
    );

    Ok(())
}
