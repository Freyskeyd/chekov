use self::support::subscriber::SubscriberFactory;

use super::{StartFrom, SubscriptionNotification, SubscriptionOptions, Subscriptions};
use crate::{
    prelude::ExpectedVersion, subscriptions::tests::support::event::MyEvent, EventStore,
    InMemoryStorage,
};
use actix::Actor;
use serde_json::json;
use test_log::test;
use uuid::Uuid;

macro_rules! pluck {
    ($i:ident, $n:ident) => {
        $i.iter().map(|event| event.$n.clone()).collect::<Vec<_>>()
    };
}

mod support;
mod transient_fsm;

struct TestContext {}

fn before_all() -> TestContext {
    TestContext {}
}

#[test(actix::test)]
async fn should_receive_subscribed_message_once_subscribed() {
    let es = EventStore::builder()
        .storage(InMemoryStorage::default())
        .build()
        .await
        .unwrap();

    let addr_es = es.start();

    let (tracker, addr) = SubscriberFactory::setup();

    let mut opts = SubscriptionOptions::default();

    opts.transient = true;
    Subscriptions::subscribe_to_stream(addr.recipient(), opts, addr_es.clone())
        .await
        .expect("Unable to subscribe");

    let x = tracker.lock().await.pop_front();
    assert!(matches!(x, Some(SubscriptionNotification::Subscribed)));
}

#[test(actix::test)]
async fn should_subscribe_to_single_stream_from_origin() {
    let es = EventStore::builder()
        .storage(InMemoryStorage::default())
        .build()
        .await
        .unwrap();

    let addr_es = es.start();

    let (tracker, addr) = SubscriberFactory::setup();

    let identity = Uuid::new_v4();
    Subscriptions::subscribe_to_stream(
        addr.recipient(),
        SubscriptionOptions {
            stream_uuid: identity.to_string(),
            subscription_name: identity.to_string(),
            start_from: StartFrom::Origin,
            transient: true,
        },
        addr_es.clone(),
    )
    .await
    .expect("Unable to subscribe");

    let _ = crate::append()
        .to(&identity)
        .unwrap()
        .expected_version(ExpectedVersion::AnyVersion)
        .event(&MyEvent {})
        .unwrap()
        .execute(addr_es.clone())
        .await;

    let x = tracker.lock().await.pop_front();
    assert!(matches!(x, Some(SubscriptionNotification::Subscribed)));

    let x = tracker.lock().await.pop_front();
    assert!(matches!(x, Some(SubscriptionNotification::Events(_))));

    if let Some(SubscriptionNotification::Events(events)) = x {
        assert_eq!(pluck!(events, event_number), [1]);
        assert_eq!(pluck!(events, stream_uuid), [identity.to_string()]);
        assert_eq!(pluck!(events, stream_version), [Some(1)]);
        assert_eq!(pluck!(events, correlation_id), [None]);
        assert_eq!(pluck!(events, causation_id), [None]);
        assert_eq!(pluck!(events, event_type), ["MyEvent".to_string()]);
        assert_eq!(pluck!(events, data), [json!({})]);
        assert_eq!(pluck!(events, metadata), [None]);
    }
}

#[test(actix::test)]
async fn should_subscribe_to_single_stream_from_given_stream_version_should_only_receive_later_events(
) {
    let es = EventStore::builder()
        .storage(InMemoryStorage::default())
        .build()
        .await
        .unwrap();

    let addr_es = es.start();

    let (tracker, addr) = SubscriberFactory::setup();

    let identity = Uuid::new_v4();
    let _ = crate::append()
        .to(&identity)
        .unwrap()
        .expected_version(ExpectedVersion::AnyVersion)
        .event(&MyEvent {})
        .unwrap()
        .event(&MyEvent {})
        .unwrap()
        .execute(addr_es.clone())
        .await;

    Subscriptions::subscribe_to_stream(
        addr.recipient(),
        SubscriptionOptions {
            stream_uuid: identity.to_string(),
            subscription_name: identity.to_string(),
            start_from: StartFrom::Version(1),
            transient: true,
        },
        addr_es.clone(),
    )
    .await
    .expect("Unable to subscribe");

    let x = tracker.lock().await.pop_front();
    assert!(matches!(x, Some(SubscriptionNotification::Subscribed)));

    let x = tracker.lock().await.pop_front();
    assert!(matches!(x, Some(SubscriptionNotification::Events(_))));

    if let Some(SubscriptionNotification::Events(events)) = x {
        assert_eq!(pluck!(events, event_number), [2]);
        assert_eq!(pluck!(events, stream_uuid), [identity.to_string()]);
        assert_eq!(pluck!(events, stream_version), [Some(2)]);
        assert_eq!(pluck!(events, correlation_id), [None]);
        assert_eq!(pluck!(events, causation_id), [None]);
        assert_eq!(pluck!(events, event_type), ["MyEvent".to_string()]);
        assert_eq!(pluck!(events, data), [json!({})]);
        assert_eq!(pluck!(events, metadata), [None]);
    }
}
