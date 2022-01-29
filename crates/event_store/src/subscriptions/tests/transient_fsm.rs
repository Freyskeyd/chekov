use actix::Actor;
use std::{collections::VecDeque, sync::Arc};

use crate::{
    prelude::ExpectedVersion,
    subscriptions::{
        fsm::{InternalFSMState, SubscriptionFSM},
        tests::{InnerSub, MyEvent},
        SubscriptionNotification, SubscriptionOptions,
    },
    EventStore, InMemoryStorage,
};

use test_log::test;
use tokio::sync::Mutex;
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

    let tracker: Arc<Mutex<VecDeque<SubscriptionNotification>>> =
        Arc::new(Mutex::new(VecDeque::new()));

    let addr = InnerSub {
        reference: Arc::clone(&tracker),
    }
    .start();

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
