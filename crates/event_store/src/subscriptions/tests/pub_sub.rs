use std::time::Duration;

use crate::{
    prelude::ExpectedVersion,
    subscriptions::{
        fsm::{InternalFSMState, SubscriptionFSM},
        pub_sub::PubSub,
        tests::support::{
            event::{EventFactory, MyEvent},
            subscriber::SubscriberFactory,
            EventStoreHelper,
        },
        StartFrom, SubscriptionNotification, SubscriptionOptions, Subscriptions,
    },
    InMemoryStorage,
};
use actix::clock::sleep;
use serde_json::json;
use test_log::test;
use uuid::Uuid;

#[test(actix::test)]
async fn ignore_events_persisted_before_subscription() {
    let es = EventStoreHelper::new(InMemoryStorage::default()).await;
    let identity = Uuid::new_v4();

    let initial = EventFactory::create_event(0);
    let event = EventFactory::create_event(1);

    let _ = es
        .append(&identity, ExpectedVersion::Version(0), &[&initial])
        .await;

    sleep(Duration::from_millis(100)).await;
    let (tracker, subscriber_addr) = SubscriberFactory::setup();

    PubSub::subscribe(subscriber_addr.recipient(), identity.to_string()).await;

    let x = tracker.lock().await.pop_front();
    println!("{:?}", x);
    assert!(matches!(x, None));

    let _ = es
        .append(&identity, ExpectedVersion::Version(1), &[&event])
        .await;

    let x = tracker.lock().await.pop_front();
    println!("{:?}", x);
    assert!(matches!(x, None));
}
