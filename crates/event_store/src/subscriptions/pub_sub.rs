use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    sync::Arc,
};

use actix::{Actor, Context, Handler, Message, Recipient, Supervised, SystemService};
use event_store_core::event::RecordedEvent;
use tracing::trace;

use super::SubscriptionNotification;

#[derive(Default, Debug)]
pub struct PubSub {
    listeners: HashMap<String, Vec<Recipient<SubscriptionNotification>>>,
}

impl Actor for PubSub {
    type Context = Context<Self>;
}

impl SystemService for PubSub {}
impl Supervised for PubSub {}

impl PubSub {
    pub async fn subscribe(recipient: Recipient<SubscriptionNotification>, stream: String) {
        Self::from_registry()
            .send(Subscribe(recipient, stream))
            .await
            .unwrap()
    }
}

#[derive(Message)]
#[rtype("()")]
struct Subscribe(Recipient<SubscriptionNotification>, String);

#[derive(Message)]
#[rtype("()")]
pub(crate) struct PubSubNotification {
    pub(crate) stream: String,
    pub(crate) events: Vec<RecordedEvent>,
}

impl Handler<Subscribe> for PubSub {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: &mut Self::Context) -> Self::Result {
        self.listeners.entry(msg.1).or_default().push(msg.0);
    }
}

impl Handler<PubSubNotification> for PubSub {
    type Result = ();

    fn handle(&mut self, msg: PubSubNotification, ctx: &mut Self::Context) -> Self::Result {
        trace!("Received PubSubNotification");
        // When receiving a PubSub notification
        // We need to:
        //  - check if someone is listening for the stream OR if there is any $all listener
        //  - convert the Vec<RecordedEvent> into a Vec<Arc<RecordedEvent>>
        //  - Send Async notification to every $all / stream listeners
        let _has_listeners = self.listeners.contains_key::<str>(msg.stream.borrow());

        let stream: Arc<String> = Arc::new(msg.stream);
        let v: Vec<Arc<RecordedEvent>> = msg.events.into_iter().map(Into::into).collect();

        if let Some(listeners) = self.listeners.get::<str>(&stream) {
            listeners.iter().for_each(|listener| {
                // FIX: Deal with failure
                let _ = listener.try_send(SubscriptionNotification::PubSubEvents(
                    stream.clone(),
                    v.clone(),
                ));
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subscriptions::tests::support::{
        event::EventFactory, subscriber::SubscriberFactory, EventStoreHelper,
    };
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
            "expected SubscriptionNotification::Subscribed received: {:?}",
            notif_1
        );

        let notif_2 = tracker_2.lock().await.pop_front();
        assert!(
            matches!(notif_2, None),
            "expected no notifications received: {:?}",
            notif_2
        );
    }
}
