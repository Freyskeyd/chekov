use crate::message::EventEnvelope;
// use crate::Chekov;
use actix::prelude::*;
use actix_interop::{with_ctx, FutureInterop};
use tracing::trace;

use crate::Application;

pub struct EventHandlerBuilder<E: EventHandler> {
    pub(crate) handler: E,
    pub(crate) name: String,
}

impl<E> EventHandlerBuilder<E>
where
    E: EventHandler,
{
    pub fn new(handler: E) -> Self {
        Self {
            handler,
            name: std::any::type_name::<E>().into(),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.into();

        self
    }

    pub async fn register<A: crate::Application>(self) {
        EventHandlerInstance::<A, E>::from_builder(self);
    }
}

pub trait BrokerMsg: Message<Result = ()> + Send + Clone + 'static {}
impl<M> BrokerMsg for M where M: Message<Result = ()> + Send + Clone + 'static {}

/// Define a struct as an EventHandler
pub trait EventHandler: Sized + std::marker::Unpin + 'static {
    fn builder(self) -> EventHandlerBuilder<Self> {
        EventHandlerBuilder::new(self)
    }
    fn started<A: Application>(&mut self, _ctx: &mut actix::Context<EventHandlerInstance<A, Self>>)
    where
        Self: EventHandler,
    {
    }
    fn listen<A: Application, M: BrokerMsg + event_store::Event + std::fmt::Debug>(
        &self,
        ctx: &mut actix::Context<EventHandlerInstance<A, Self>>,
    ) where
        Self: crate::event::Handler<M>,
    {
        let broker = crate::subscriber::SubscriberManager::<A>::from_registry();
        let recipient = ctx.address().recipient::<EventEnvelope<M>>();
        broker.do_send(Subscribe(
            "$all".into(),
            recipient,
            std::any::type_name::<M>().to_string(),
        ));
    }
}

#[doc(hidden)]
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Subscribe<M: BrokerMsg>(pub String, pub Recipient<M>, pub String);

/// Deals with the lifetime of a particular EventHandler
pub struct EventHandlerInstance<A: Application, E: EventHandler> {
    _phantom: std::marker::PhantomData<A>,
    pub(crate) _handler: E,
    // pub(crate) _subscribtion: Addr<Subscriber>,
    pub(crate) _name: String,
}

impl<A: Application, E: EventHandler, T: event_store::Event + 'static>
    ::actix::Handler<EventEnvelope<T>> for EventHandlerInstance<A, E>
where
    E: crate::event::Handler<T>,
{
    type Result = ();

    #[tracing::instrument(name = "EventHandlerInstance", skip(self, msg, ctx))]
    fn handle(&mut self, msg: EventEnvelope<T>, ctx: &mut Self::Context) -> Self::Result {
        let fut = async move {
            let _ = with_ctx(|actor: &mut Self, _| actor._handler.handle(&msg.event)).await;
        };

        ctx.spawn(fut.interop_actor(self).map(|_, _, _| ()));
    }
}

impl<A: Application, E: EventHandler> EventHandlerInstance<A, E> {
    #[tracing::instrument(name = "EventHandlerInstance", skip(builder))]
    pub fn from_builder(builder: EventHandlerBuilder<E>) -> Addr<Self> {
        Self::create(move |_ctx| {
            trace!("Register a new EventHandler instance with {}", builder.name);

            EventHandlerInstance {
                _phantom: std::marker::PhantomData,
                _handler: builder.handler,
                _name: builder.name,
            }
        })
    }
}
impl<A: Application, E: EventHandler> actix::Actor for EventHandlerInstance<A, E> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self._handler.started(ctx);
    }
}
