use crate::{
    aggregate::AggregateInstance, assert_aggregate_state, assert_aggregate_version,
    message::ResolveAndApply, Aggregate,
};

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
            .expect(&format!(
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
        stream_version: None,
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

    addr.send(ResolveAndApply(event)).await?;

    assert_aggregate_version!(&addr, 1);
    assert_aggregate_state!(
        &addr,
        ExampleAggregate {
            items: vec![1],
            last_index: 0
        }
    );

    Ok(())
    // pid = Registration.whereis_name(DefaultApp, {DefaultApp, BankAccount, account_number})
    // events = EventStore.stream_forward(DefaultApp, account_number) |> Enum.to_list()
    //
    // # send already seen events multiple times, they should be ignored
    // send(pid, {:events, events})
    // send(pid, {:events, events})
    // send(pid, {:events, events})
    // send(pid, {:events, events})
    //
    // assert Aggregate.aggregate_version(DefaultApp, BankAccount, account_number) == 2
    //
    // assert Aggregate.aggregate_state(DefaultApp, BankAccount, account_number) == %BankAccount{
    //          account_number: account_number,
    //          balance: 1_500,
    //          state: :active
    //        }
}

#[test(actix::test)]
async fn should_stop_aggregate_process_when_unexpected_event_received() {
    // pid = Registration.whereis_name(DefaultApp, {DefaultApp, BankAccount, account_number})
    // ref = Process.monitor(pid)
    //
    // events =
    //   EventStore.stream_forward(DefaultApp, account_number)
    //   |> Enum.to_list()
    //   |> Enum.map(fn recorded_event ->
    //     # specify invalid stream version
    //     %EventStore.RecordedEvent{
    //       recorded_event
    //       | stream_version: 999
    //     }
    //   end)
    //
    // # send invalid events, should stop the aggregate process
    // send(pid, {:events, events})
    //
    // assert_receive {:DOWN, ^ref, :process, _, :unexpected_event_received}
}
