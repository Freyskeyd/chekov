use crate::event::RecordedEvent;
use crate::EventStore;
use actix::Addr;
use actix::Message;
use actix::Recipient;
use event_store_core::storage::Storage;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::Arc;
use uuid::Uuid;

mod error;
mod fsm;
pub mod pub_sub;
mod state;
mod subscriber;
mod subscription;
mod supervisor;

#[cfg(test)]
mod tests;

use self::error::SubscriptionError;

pub use self::subscription::Subscription;
pub use supervisor::SubscriptionsSupervisor;

///
/// Subscribe to a stream start a subscription in the supervisor
///   The supervisor starts a subscription actor
///     The subscription actor create a new FSM
///   The subscription actor receive a Connect message
///     - the FSM connects the subscriber
///     - the FSM subscribe
///
pub struct Subscriptions<S: Storage> {
    _phantom: PhantomData<S>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionOptions {
    pub stream_uuid: String,
    pub subscription_name: String,
    pub start_from: StartFrom,
    pub transient: bool,
}

impl Default for SubscriptionOptions {
    fn default() -> Self {
        Self {
            stream_uuid: String::new(),
            subscription_name: Uuid::new_v4().to_string(),
            start_from: StartFrom::default(),
            transient: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum StartFrom {
    Origin,
    Version(i64),
}

impl From<StartFrom> for i64 {
    fn from(s: StartFrom) -> Self {
        match s {
            StartFrom::Origin => 0,
            StartFrom::Version(i) => i,
        }
    }
}
impl Default for StartFrom {
    fn default() -> Self {
        Self::Origin
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), ()>")]
pub enum SubscriptionNotification {
    Events(Arc<Vec<Arc<RecordedEvent>>>),
    OwnedEvents(Cow<'static, Arc<Vec<Arc<RecordedEvent>>>>),
    PubSubEvents(Arc<String>, Vec<Arc<RecordedEvent>>),
    Subscribed,
}

impl<S: Storage> Subscriptions<S> {
    pub async fn subscribe_to_stream(
        subscriber: Recipient<SubscriptionNotification>,
        options: SubscriptionOptions,
        storage: Addr<EventStore<S>>,
    ) -> Result<Addr<Subscription<S>>, SubscriptionError>
    where
        S: Storage,
    {
        let subscription =
            SubscriptionsSupervisor::<S>::start_subscription(&options, storage).await?;
        let _ = Subscription::connect(&subscription, subscriber, &options).await;

        Ok(subscription)
    }

    pub fn notify_subscribers(stream_uuid: &str, events: Arc<Vec<Arc<RecordedEvent>>>) {
        SubscriptionsSupervisor::<S>::notify_subscribers(stream_uuid, events);
    }
}
