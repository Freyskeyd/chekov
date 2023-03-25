use crate::{assert_aggregate_state, assert_aggregate_version, message::ResolveAndApply};

use super::support::*;
use event_store::{prelude::RecordedEvent, Event, PubSub};
use test_log::test;
use uuid::Uuid;

#[test(actix::test)]
async fn aggregate_should_starts_a_pubsub_subscription() {
    let identity = Uuid::new_v4();
    let _ = start_context(&identity).await;

    assert_eq!(
        1,
        PubSub::has_subscriber_for(identity.to_string())
            .await
            .unwrap_or_else(|_| panic!(
                "Failed to fetch the subscriber list for the stream {}",
                identity
            ))
    );
}

/// Append event directly to aggregate stream
#[test(actix::test)]
async fn should_notify_aggregate_and_mutate_its_state() -> Result<(), Box<dyn std::error::Error>> {
    let identity = Uuid::new_v4();
    let addr = start_context(&identity).await;

    assert_aggregate_version!(&addr, 0);
    assert_aggregate_state!(
        &addr,
        ExampleAggregate {
            items: vec![],
            last_index: 0
        }
    );

    Ok(())
}

#[test(actix::test)]
async fn should_ignore_already_seen_events() -> Result<(), Box<dyn std::error::Error>> {
    let identity = Uuid::new_v4();
    let addr = start_context(&identity).await;

    let base = ItemAppended(1);
    let event = RecordedEvent {
        event_number: 1,
        event_uuid: Uuid::new_v4(),
        stream_uuid: identity.to_string(),
        stream_version: Some(1),
        causation_id: None,
        correlation_id: None,
        event_type: base.event_type().to_string(),
        data: serde_json::to_value(base).unwrap(),
        metadata: None,
        created_at: chrono::offset::Utc::now(),
    };

    assert_aggregate_version!(&addr, 0);
    assert_aggregate_state!(
        &addr,
        ExampleAggregate {
            items: vec![],
            last_index: 0
        }
    );

    let _ = addr.send(ResolveAndApply(event.clone())).await?;

    assert_aggregate_version!(&addr, 1);
    assert_aggregate_state!(
        &addr,
        ExampleAggregate {
            items: vec![1],
            last_index: 0
        }
    );

    let _ = addr.send(ResolveAndApply(event.clone())).await?;

    assert_aggregate_version!(&addr, 1);
    assert_aggregate_state!(
        &addr,
        ExampleAggregate {
            items: vec![1],
            last_index: 0
        }
    );

    let second = ItemAppended(2);
    let event2 = RecordedEvent {
        event_number: 2,
        event_uuid: Uuid::new_v4(),
        stream_uuid: identity.to_string(),
        stream_version: Some(2),
        causation_id: None,
        correlation_id: None,
        event_type: second.event_type().to_string(),
        data: serde_json::to_value(second).unwrap(),
        metadata: None,
        created_at: chrono::offset::Utc::now(),
    };

    assert!(addr.send(ResolveAndApply(event2.clone())).await?.is_ok());

    assert_aggregate_version!(&addr, 2);
    assert_aggregate_state!(
        &addr,
        ExampleAggregate {
            items: vec![1, 2],
            last_index: 1
        }
    );

    assert!(addr.send(ResolveAndApply(event)).await?.is_ok());

    assert_aggregate_version!(&addr, 2);
    assert_aggregate_state!(
        &addr,
        ExampleAggregate {
            items: vec![1, 2],
            last_index: 1
        }
    );

    Ok(())
}

#[test(actix::test)]
async fn should_stop_aggregate_process_when_unexpected_event_received(
) -> Result<(), Box<dyn std::error::Error>> {
    let identity = Uuid::new_v4();
    let addr = start_context(&identity).await;

    let base = ItemAppended(1);
    let event = RecordedEvent {
        event_number: 999,
        event_uuid: Uuid::new_v4(),
        stream_uuid: identity.to_string(),
        stream_version: Some(999),
        causation_id: None,
        correlation_id: None,
        event_type: base.event_type().to_string(),
        data: serde_json::to_value(base).unwrap(),
        metadata: None,
        created_at: chrono::offset::Utc::now(),
    };

    assert_aggregate_version!(&addr, 0);
    assert_aggregate_state!(
        &addr,
        ExampleAggregate {
            items: vec![],
            last_index: 0
        }
    );

    assert!(addr.send(ResolveAndApply(event.clone())).await?.is_err());

    assert!(!addr.connected());

    Ok(())
}
