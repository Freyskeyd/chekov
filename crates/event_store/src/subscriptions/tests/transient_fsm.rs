use crate::{
    prelude::ExpectedVersion,
    subscriptions::{
        fsm::{InternalFSMState, SubscriptionFSM},
        tests::support::{
            event::{EventFactory, MyEvent},
            subscriber::SubscriberFactory,
            EventStoreHelper,
        },
        StartFrom, SubscriptionNotification, SubscriptionOptions, Subscriptions,
    },
    InMemoryStorage,
};
use serde_json::json;
use test_log::test;
use uuid::Uuid;

#[test(actix::test)]
async fn transient_subscription() {
    let es = EventStoreHelper::new(InMemoryStorage::default()).await;
    let identity = Uuid::new_v4();

    let _ = es
        .append(
            &identity,
            ExpectedVersion::AnyVersion,
            &[&MyEvent {}, &MyEvent {}],
        )
        .await;

    let (tracker, addr) = SubscriberFactory::setup();
    let opts = SubscriptionOptions {
        transient: true,
        stream_uuid: identity.to_string(),
        ..Default::default()
    };

    let mut fsm = SubscriptionFSM::with_options(&opts, es.get_addr());

    assert_eq!(fsm.state, InternalFSMState::Initial);
    assert!(matches!(fsm.data.subscriber, None));

    fsm.connect_subscriber(addr.recipient()).await;

    assert_eq!(fsm.state, InternalFSMState::Initial);
    assert!(matches!(fsm.data.subscriber, Some(_)));

    fsm.subscribe().await;

    let x = tracker.lock().await.pop_front();
    assert!(matches!(x, Some(SubscriptionNotification::Subscribed)));

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
    let es = EventStoreHelper::new(InMemoryStorage::default()).await;
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
        es.get_addr(),
    )
    .await
    .expect("Unable to subscribe");

    let x = tracker.lock().await.pop_front();
    assert!(matches!(x, Some(SubscriptionNotification::Subscribed)));

    let _ = es
        .append(&identity, ExpectedVersion::AnyVersion, &[&event])
        .await;

    let x = tracker.lock().await.pop_front();
    assert!(
        matches!(x, Some(SubscriptionNotification::PubSubEvents(_, ref events)) if events.len() == 1)
    );

    if let Some(SubscriptionNotification::PubSubEvents(stream_uuid, ref events)) = x {
        assert_eq!(stream_uuid.as_ref(), &identity.to_string());
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
