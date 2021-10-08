use super::{SubscriptionNotification, SubscriptionOptions, Subscriptions};
use crate::{
    event::RecordedEvent,
    prelude::{ExpectedVersion, PostgresEventBus},
    Event, EventStore, InMemoryBackend,
};
use actix::{Actor, Context, Handler, ResponseActFuture, WrapFuture};
use std::{collections::VecDeque, convert::TryFrom, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

struct TestContext {}

fn before_all() -> TestContext {
    TestContext {}
}

struct InnerSub {
    reference: Arc<Mutex<VecDeque<SubscriptionNotification>>>,
}

impl Actor for InnerSub {
    type Context = Context<Self>;
}

#[derive(serde::Serialize)]
struct MyEvent {}
impl Event for MyEvent {
    fn event_type(&self) -> &'static str {
        "MyEvent"
    }

    fn all_event_types() -> Vec<&'static str> {
        vec!["MyEvent"]
    }
}

impl TryFrom<RecordedEvent> for MyEvent {
    type Error = ();

    fn try_from(_: RecordedEvent) -> Result<Self, Self::Error> {
        Ok(MyEvent {})
    }
}

impl Handler<SubscriptionNotification> for InnerSub {
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, msg: SubscriptionNotification, _ctx: &mut Self::Context) -> Self::Result {
        let aquire = Arc::clone(&self.reference);

        Box::pin(
            async move {
                let mut mutex = aquire.lock().await;
                mutex.push_back(msg);
                Ok(())
            }
            .into_actor(self),
        )
    }
}

#[actix::test]
async fn should_receive_subscribed_message_once_subscribed() {
    let tracker: Arc<Mutex<VecDeque<SubscriptionNotification>>> =
        Arc::new(Mutex::new(VecDeque::new()));

    let addr = InnerSub {
        reference: Arc::clone(&tracker),
    }
    .start();

    Subscriptions::<InMemoryBackend>::subscribe_to_stream(
        addr.recipient(),
        SubscriptionOptions::default(),
    )
    .await
    .expect("Unable to subscribe");

    let x = tracker.lock().await.pop_front();
    assert!(matches!(x, Some(SubscriptionNotification::Subscribed)));
}

#[actix::test]
async fn should_subscribe_to_single_stream_from_origin() {
    let es = EventStore::builder()
        .storage(InMemoryBackend::default())
        .event_bus(
            PostgresEventBus::initiate(
                "postgresql://postgres:postgres@localhost/event_store_gift_shop".into(),
            )
            .await
            .unwrap(),
        )
        .build()
        .await
        .unwrap();

    let addr_es = es.start();

    let tracker: Arc<Mutex<VecDeque<SubscriptionNotification>>> =
        Arc::new(Mutex::new(VecDeque::new()));

    let addr = InnerSub {
        reference: Arc::clone(&tracker),
    }
    .start();

    let identity = Uuid::new_v4();
    Subscriptions::<InMemoryBackend>::subscribe_to_stream(
        addr.recipient(),
        SubscriptionOptions {
            stream_uuid: identity.to_string(),
            subscription_name: identity.to_string(),
        },
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

    //     let x = tracker.lock().await.pop_front();
    //     assert!(matches!(x, Some(SubscriptionNotification::Events(_))))
}
