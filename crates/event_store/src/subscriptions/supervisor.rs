use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::EventStore;

use super::error::SubscriptionError;
use super::subscription::Subscription;
use super::SubscriptionOptions;
use actix::prelude::*;
use event_store_core::event::RecordedEvent;
use event_store_core::storage::Storage;

#[derive(Default, Debug)]
pub struct SubscriptionsSupervisor<S: Storage> {
    _storage: PhantomData<S>,
    subscriptions: HashMap<String, Vec<Addr<Subscription<S>>>>,
}

impl<S: Storage> Actor for SubscriptionsSupervisor<S> {
    type Context = Context<Self>;
}

impl<S: Storage> Supervised for SubscriptionsSupervisor<S> {}
impl<S: Storage> ArbiterService for SubscriptionsSupervisor<S> {}

impl<S: Storage> SubscriptionsSupervisor<S> {
    pub async fn start_subscription(
        options: &SubscriptionOptions,
        storage: Addr<EventStore<S>>,
    ) -> Result<Addr<Subscription<S>>, SubscriptionError> {
        match Self::from_registry()
            .send(CreateSubscription(options.clone(), storage))
            .await
        {
            Ok(v) => Ok(v),
            Err(_) => Err(SubscriptionError::UnableToStart),
        }
    }

    pub fn notify_subscribers(stream_uuid: &str, events: Vec<RecordedEvent>) {
        Self::from_registry().do_send(Notify(stream_uuid.into(), events));
    }
}

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct Notify(pub(crate) String, pub(crate) Vec<RecordedEvent>);

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct NotifySubscribers(pub(crate) String, pub(crate) Arc<Vec<Arc<RecordedEvent>>>);

impl<S: Storage> Handler<Notify> for SubscriptionsSupervisor<S> {
    type Result = ();

    fn handle(
        &mut self,
        Notify(stream_uuid, events): Notify,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        if let Some(subscriptions) = self.subscriptions.get(&stream_uuid) {
            let events: Arc<Vec<Arc<RecordedEvent>>> =
                Arc::new(events.into_iter().map(|event| Arc::new(event)).collect());

            subscriptions.iter().for_each(|sub| {
                sub.do_send(NotifySubscribers(stream_uuid.clone(), events.clone()))
            });
        }
    }
}

#[derive(Message)]
#[rtype(result = "Addr<Subscription<S>>")]
struct CreateSubscription<S: Storage>(SubscriptionOptions, Addr<EventStore<S>>);

impl<S: Storage> Handler<CreateSubscription<S>> for SubscriptionsSupervisor<S> {
    type Result = MessageResult<CreateSubscription<S>>;

    fn handle(&mut self, msg: CreateSubscription<S>, ctx: &mut Self::Context) -> Self::Result {
        let addr = Subscription::start_with_options(&msg.0, ctx.address(), msg.1);

        self.subscriptions
            .entry(msg.0.stream_uuid)
            .or_default()
            .push(addr.clone());

        MessageResult(addr)
    }
}

#[derive(Message)]
#[rtype("()")]
pub struct Started;

impl<S: Storage> Handler<Started> for SubscriptionsSupervisor<S> {
    type Result = MessageResult<Started>;

    fn handle(&mut self, _: Started, _: &mut Self::Context) -> Self::Result {
        MessageResult(())
    }
}

#[derive(Message)]
#[rtype("()")]
pub struct GoingDown;

impl<S: Storage> Handler<GoingDown> for SubscriptionsSupervisor<S> {
    type Result = MessageResult<GoingDown>;

    fn handle(&mut self, _: GoingDown, _: &mut Self::Context) -> Self::Result {
        MessageResult(())
    }
}

#[derive(Message)]
#[rtype("()")]
pub struct Down;

impl<S: Storage> Handler<Down> for SubscriptionsSupervisor<S> {
    type Result = MessageResult<Down>;

    fn handle(&mut self, _: Down, _: &mut Self::Context) -> Self::Result {
        MessageResult(())
    }
}
