use crate::{
    message::{EventEnvelope, ResolveAndApplyMany},
    Event,
};
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

pub trait BrokerMsg: Message<Result = Result<(), ()>> + Send + Clone + 'static {}
impl<M> BrokerMsg for M where M: Message<Result = Result<(), ()>> + Send + Clone + 'static {}

/// Define a struct as an EventHandler
pub trait EventHandler: Sized + std::marker::Unpin + 'static {
    fn builder(self) -> EventHandlerBuilder<Self> {
        EventHandlerBuilder::new(self)
    }

    fn listen<A: Application, M: BrokerMsg + Event + std::fmt::Debug>(
        &self,
        ctx: &mut actix::Context<EventHandlerInstance<A, Self>>,
    ) {
        let broker = crate::subscriber::SubscriberManager::<A>::from_registry();
        let recipient = ctx.address().recipient::<ResolveAndApplyMany>();
        broker.do_send(Subscribe("$all".into(), recipient));
    }

    fn started<A: Application>(&mut self, _ctx: &mut actix::Context<EventHandlerInstance<A, Self>>)
    where
        Self: EventHandler,
    {
    }
}

#[doc(hidden)]
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Subscribe(pub String, pub Recipient<ResolveAndApplyMany>);

/// Deals with the lifetime of a particular EventHandler
pub struct EventHandlerInstance<A: Application, E: EventHandler> {
    _phantom: std::marker::PhantomData<A>,
    pub(crate) _handler: E,
    pub(crate) _name: String,
}

impl<A: Application, E: EventHandler, T: Event + 'static> ::actix::Handler<EventEnvelope<T>>
    for EventHandlerInstance<A, E>
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
        EventHandler::started(&mut self._handler, ctx);
    }
}

impl<A: Application, E: EventHandler> ::actix::Handler<ResolveAndApplyMany>
    for EventHandlerInstance<A, E>
{
    type Result = Result<(), ()>;

    #[tracing::instrument(name = "EventHandlerInstance", skip(self, _msg, _ctx))]
    fn handle(&mut self, _msg: ResolveAndApplyMany, _ctx: &mut Self::Context) -> Self::Result {
        //TODO create Registry event
        //
        Ok(())

        // let fut = async move {
        //     let _ = with_ctx(|actor: &mut Self, _| actor._handler.handle(&msg.event)).await;
        // };

        // ctx.spawn(fut.interop_actor(self).map(|_, _, _| ()));
    }
}
