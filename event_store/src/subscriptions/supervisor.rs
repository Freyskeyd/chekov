use std::marker::PhantomData;

use crate::event::RecordedEvents;
use crate::prelude::EventBus;
use crate::storage::Storage;
use crate::EventStore;

use super::subscription::Subscription;
use super::SubscriptionOptions;
use actix::prelude::*;

#[derive(Default, Debug)]
pub struct SubscriptionsSupervisor<S: Storage> {
    _storage: PhantomData<S>,
    subscriptions: Vec<Addr<Subscription<S>>>,
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
    ) -> Result<Addr<Subscription<S>>, ()> {
        match Self::from_registry()
            .send(CreateSubscription(options.clone(), storage))
            .await
        {
            Ok(v) => Ok(v),
            Err(_) => Err(()),
        }
    }

    pub fn notify_subscribers(events: RecordedEvents) {
        Self::from_registry().do_send(Notify(events));
    }
}

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct Notify(pub(crate) RecordedEvents);

impl<S: Storage> Handler<Notify> for SubscriptionsSupervisor<S> {
    type Result = ();

    fn handle(&mut self, msg: Notify, _ctx: &mut Self::Context) -> Self::Result {
        self.subscriptions
            .iter()
            .for_each(|sub| sub.do_send(msg.clone()));
    }
}

#[derive(Message)]
#[rtype(result = "Addr<Subscription<S>>")]
struct CreateSubscription<S: Storage>(SubscriptionOptions, Addr<EventStore<S>>);

impl<S: Storage> Handler<CreateSubscription<S>> for SubscriptionsSupervisor<S> {
    type Result = MessageResult<CreateSubscription<S>>;

    fn handle(&mut self, msg: CreateSubscription<S>, ctx: &mut Self::Context) -> Self::Result {
        let addr = Subscription::start_with_options(&msg.0, ctx.address(), msg.1);

        self.subscriptions.push(addr.clone());

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
