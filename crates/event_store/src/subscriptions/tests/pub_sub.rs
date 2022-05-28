use crate::subscriptions::{
    pub_sub::{PubSub, PubSubNotification},
    tests::support::{event::EventFactory, subscriber::SubscriberFactory, EventStoreHelper},
    SubscriptionNotification,
};
use actix::SystemService;
use event_store_core::versions::ExpectedVersion;
use event_store_storage_inmemory::InMemoryStorage;
use test_log::test;
use uuid::Uuid;

#[test(actix::test)]
async fn should_handle_a_notification() {
    let notification = PubSubNotification {
        stream: "stream_name".into(),
        events: Vec::new(),
    };

    let result = PubSub::from_registry().send(notification).await;

    assert!(matches!(result, Ok(())));
}

#[test(actix::test)]
async fn should_only_notify_subscribers() {
    let (tracker, subscriber_addr) = SubscriberFactory::setup();
    let (tracker_2, _) = SubscriberFactory::setup();
    let identity = Uuid::new_v4();

    PubSub::subscribe(subscriber_addr.recipient(), identity.to_string()).await;

    let es = EventStoreHelper::new(InMemoryStorage::default()).await;
    let initial = EventFactory::create_event(0);

    let _ = es
        .append(&identity, ExpectedVersion::Version(0), &[&initial])
        .await;

    let notif_1 = tracker.lock().await.pop_front();
    assert!(
        matches!(notif_1, Some(SubscriptionNotification::PubSubEvents(ref stream, ref events)) if !events.is_empty() && stream.as_ref() == &identity.to_string()),
        "expected SubscriptionNotification::PubSubEvents received: {:?}",
        notif_1
    );

    let notif_2 = tracker_2.lock().await.pop_front();
    assert!(
        matches!(notif_2, None),
        "expected no notifications received: {:?}",
        notif_2
    );
}

#[test(actix::test)]
async fn ignore_events_persisted_before_subscription() {
    let es = EventStoreHelper::new(InMemoryStorage::default()).await;
    let identity = Uuid::new_v4();

    let initial = EventFactory::create_event(0);
    let event = EventFactory::create_event(1);

    let _ = es
        .append(&identity, ExpectedVersion::Version(0), &[&initial])
        .await;

    let (tracker, subscriber_addr) = SubscriberFactory::setup();

    PubSub::subscribe(subscriber_addr.recipient(), identity.to_string()).await;

    let notif_1 = tracker.lock().await.pop_front();
    assert!(
        matches!(notif_1, None),
        "expected no notifications received: {:?}",
        notif_1
    );

    let _ = es
        .append(&identity, ExpectedVersion::Version(1), &[&event])
        .await;

    let notif_2 = tracker.lock().await.pop_front();
    assert!(
        matches!(notif_2, Some(SubscriptionNotification::PubSubEvents(ref stream, ref events)) if !events.is_empty() && stream.as_ref() == &identity.to_string()),
        "expected SubscriptionNotification::PubSubEvents received: {:?}",
        notif_2
    );
}
