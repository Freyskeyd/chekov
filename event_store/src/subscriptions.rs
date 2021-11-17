use crate::event::RecordedEvent;
use crate::event::RecordedEvents;
use crate::prelude::EventBus;
use actix::Addr;
use actix::Message;
use actix::Recipient;
use std::marker::PhantomData;

mod fsm;
mod state;
mod subscriber;
mod subscription;
mod supervisor;

#[cfg(test)]
mod test;

pub use self::subscription::Subscription;
pub use supervisor::SubscriptionsSupervisor;

///
/// Subscribe to a stream start a subscription in the superviseur
///   The supervisor starts a subscription actor
///     The subscription actor create a new FSM
///   The subscription actor receive a Connect message
///     - the FSM connects the subscriber
///     - the FSM subscribe
///
pub struct Subscriptions<S: EventBus> {
    _phantom: PhantomData<S>,
}

#[derive(Default, Debug, Clone)]
pub struct SubscriptionOptions {
    pub stream_uuid: String,
    pub subscription_name: String,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), ()>")]
pub enum SubscriptionNotification {
    Events(Vec<RecordedEvent>),
    Subscribed,
}

impl<S: EventBus> Subscriptions<S> {
    pub async fn subscribe_to_stream(
        subscriber: Recipient<SubscriptionNotification>,
        options: SubscriptionOptions,
    ) -> Result<Addr<Subscription<S>>, ()> {
        match SubscriptionsSupervisor::<S>::start_subscription(&options).await {
            Ok(subscription) => {
                let _ = Subscription::connect(&subscription, &subscriber, &options).await;
                Ok(subscription)
            },

            Err(_) => {

                Err(())
            }
        }
    }

    pub fn notify_subscribers(events: RecordedEvents) {
        SubscriptionsSupervisor::<S>::notify_subscribers(events);
    }
}
