use actix::Actor;
use serde_json::json;

use crate::{
    prelude::ExpectedVersion,
    subscriptions::{
        fsm::{InternalFSMState, SubscriptionFSM},
        tests::support::{
            event::{EventFactory, MyEvent},
            subscriber::SubscriberFactory,
        },
        StartFrom, SubscriptionNotification, SubscriptionOptions, Subscriptions,
    },
    EventStore, InMemoryStorage,
};

use test_log::test;
use uuid::Uuid;

#[test(actix::test)]
async fn transient_subscription() {
    let es = EventStore::builder()
        .storage(InMemoryStorage::default())
        .build()
        .await
        .unwrap()
        .start();

    let identity = Uuid::new_v4();
    let _ = crate::append()
        .to(&identity)
        .unwrap()
        .expected_version(ExpectedVersion::AnyVersion)
        .event(&MyEvent {})
        .unwrap()
        .event(&MyEvent {})
        .unwrap()
        .execute(es.clone())
        .await;

    let (_tracker, addr) = SubscriberFactory::setup();
    let mut opts = SubscriptionOptions::default();
    opts.transient = true;
    opts.stream_uuid = identity.to_string();

    let mut fsm = SubscriptionFSM::with_options(&opts, es);

    assert_eq!(fsm.state, InternalFSMState::Initial);
    assert!(matches!(fsm.data.subscriber, None));

    fsm.connect_subscriber(addr.recipient()).await;

    assert_eq!(fsm.state, InternalFSMState::Initial);
    assert!(matches!(fsm.data.subscriber, Some(_)));

    fsm.subscribe().await;

    assert_eq!(fsm.state, InternalFSMState::RequestCatchUp);
    assert_eq!(fsm.data.last_received, 0);
    assert_eq!(fsm.data.last_sent, 0);
    assert_eq!(fsm.data.last_ack, 0);

    fsm.catch_up().await;

    assert_eq!(fsm.state, InternalFSMState::CatchingUp);
    assert_eq!(fsm.data.subscriber.unwrap().in_flight.len(), 2);
}

#[test(actix::test)]
async fn notify_subscribers_after_events_persisted_to_stream() {
    let es = EventStore::builder()
        .storage(InMemoryStorage::default())
        .build()
        .await
        .unwrap()
        .start();

    let identity = Uuid::new_v4();

    let event = EventFactory::create_event(0);
    let (tracker, subscriber_addr) = SubscriberFactory::setup();

    Subscriptions::subscribe_to_stream(
        subscriber_addr.recipient(),
        SubscriptionOptions {
            stream_uuid: identity.to_string(),
            subscription_name: identity.to_string(),
            start_from: StartFrom::Origin,
            transient: true,
        },
        es.clone(),
    )
    .await
    .expect("Unable to subscribe");

    let x = tracker.lock().await.pop_front();
    assert!(matches!(x, Some(SubscriptionNotification::Subscribed)));

    let _ = crate::append()
        .to(&identity)
        .unwrap()
        .expected_version(ExpectedVersion::AnyVersion)
        .event(&event)
        .unwrap()
        .execute(es.clone())
        .await;

    let x = tracker.lock().await.pop_front();
    assert!(matches!(x, Some(SubscriptionNotification::Events(ref events)) if events.len() == 1));

    if let Some(SubscriptionNotification::Events(ref events)) = x {
        assert_eq!(pluck!(events, event_number), [1]);
        assert_eq!(pluck!(events, stream_uuid), [identity.to_string()]);
        assert_eq!(pluck!(events, stream_version), [Some(1)]);
        assert_eq!(pluck!(events, correlation_id), [None]);
        assert_eq!(pluck!(events, causation_id), [None]);
        assert_eq!(pluck!(events, event_type), ["TestEvent".to_string()]);
        assert_eq!(pluck!(events, data), [json!({"event": 0})]);
        assert_eq!(pluck!(events, metadata), [None]);
    }
}
