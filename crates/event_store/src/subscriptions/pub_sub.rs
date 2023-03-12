// TODO: PubSubNotification needs to be filterable by subscribtion (need to store a closure (RecordedEvent -> boolean))
use std::{collections::HashMap, sync::Arc};

use super::SubscriptionNotification;
use actix::{Actor, Context, Handler, MailboxError, Message, Recipient, Supervised, SystemService};
use event_store_core::event::RecordedEvent;
use tracing::trace;

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

    pub async fn has_subscriber_for(stream: String) -> Result<usize, MailboxError> {
        Self::from_registry()
            .send(GetSubscriberCountForStream(stream))
            .await
    }
}

#[derive(Message)]
#[rtype("()")]
struct Subscribe(Recipient<SubscriptionNotification>, String);

#[derive(Message)]
#[rtype("usize")]
struct GetSubscriberCountForStream(String);

#[derive(Message)]
#[rtype("()")]
pub struct PubSubNotification {
    pub(crate) stream: String,
    pub(crate) events: Vec<RecordedEvent>,
}

impl Handler<Subscribe> for PubSub {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: &mut Self::Context) -> Self::Result {
        self.listeners.entry(msg.1).or_default().push(msg.0);
    }
}

impl Handler<GetSubscriberCountForStream> for PubSub {
    type Result = usize;

    fn handle(
        &mut self,
        msg: GetSubscriberCountForStream,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.listeners.get(&msg.0).map_or(0, Vec::len)
    }
}

impl Handler<PubSubNotification> for PubSub {
    type Result = ();

    fn handle(&mut self, msg: PubSubNotification, _ctx: &mut Self::Context) -> Self::Result {
        trace!("Received PubSubNotification");
        // When receiving a PubSub notification
        // We need to:
        //  - check if someone is listening for the stream OR if there is any $all listener
        //  - convert the Vec<RecordedEvent> into a Vec<Arc<RecordedEvent>>
        //  - Send Async notification to every $all / stream listeners

        let stream: Arc<String> = Arc::new(msg.stream);
        let v: Vec<Arc<RecordedEvent>> = msg.events.into_iter().map(Into::into).collect();

        if let Some(listeners) = self.listeners.get::<str>(&stream) {
            for listener in listeners.iter() {
                // FIX: Deal with failure
                let _ = listener.try_send(SubscriptionNotification::PubSubEvents(
                    stream.clone(),
                    v.clone(),
                ));
            }
        }

        // TODO: Group the two loops
        // WARNING: Listeners subscribing both to stream_id and $all will receive events 2times
        if let Some(listeners) = self.listeners.get::<str>("$all") {
            for listener in listeners.iter() {
                // FIX: Deal with failure
                let _ = listener.try_send(SubscriptionNotification::PubSubEvents(
                    stream.clone(),
                    v.clone(),
                ));
            }
        }
    }
}
