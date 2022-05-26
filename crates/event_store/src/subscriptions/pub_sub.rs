use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    sync::Arc,
};

use actix::{Actor, Context, Handler, Message, Recipient, Supervised, SystemService};
use event_store_core::event::RecordedEvent;

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
            .send(Register(recipient, stream))
            .await
            .unwrap()
    }
}

#[derive(Message)]
#[rtype("()")]
struct Register(Recipient<SubscriptionNotification>, String);

#[derive(Message)]
#[rtype("()")]
pub(crate) struct PubSubNotification {
    pub(crate) stream: String,
    pub(crate) events: Vec<RecordedEvent>,
}

impl Handler<Register> for PubSub {
    type Result = ();

    fn handle(&mut self, msg: Register, _ctx: &mut Self::Context) -> Self::Result {
        self.listeners.entry(msg.1).or_default().push(msg.0);
    }
}

impl Handler<PubSubNotification> for PubSub {
    type Result = ();

    fn handle(&mut self, msg: PubSubNotification, ctx: &mut Self::Context) -> Self::Result {
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
    use test_log::test;

    #[test(actix::test)]
    async fn should_handle_a_notification() {
        let notification = PubSubNotification {
            stream: "stream_name".into(),
            events: Vec::new(),
        };

        let result = PubSub::from_registry().send(notification).await;

        assert!(matches!(result, Ok(())));
    }
}
