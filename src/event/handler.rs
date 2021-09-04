use crate::message::ResolveAndApplyMany;
use actix::prelude::*;
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

/// Define a struct as an EventHandler
#[async_trait::async_trait]
pub trait EventHandler: Clone + Sized + std::marker::Unpin + 'static {
    fn builder(self) -> EventHandlerBuilder<Self> {
        EventHandlerBuilder::new(self)
    }

    async fn handle_recorded_event(
        state: &mut Self,
        event: event_store::prelude::RecordedEvent,
    ) -> Result<(), ()>;

    fn listen<A: Application>(&self, ctx: &mut actix::Context<EventHandlerInstance<A, Self>>) {
        let broker = crate::subscriber::SubscriberManager::<A>::from_registry();
        let recipient = ctx.address().recipient::<ResolveAndApplyMany>();
        broker.do_send(Subscribe("$all".into(), recipient));
    }

    fn started<A: Application>(&mut self, ctx: &mut actix::Context<EventHandlerInstance<A, Self>>)
    where
        Self: EventHandler,
    {
        self.listen(ctx);
    }
}

#[doc(hidden)]
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Subscribe(pub String, pub Recipient<ResolveAndApplyMany>);

/// Deals with the lifetime of a particular EventHandler
pub struct EventHandlerInstance<A: Application, E: EventHandler> {
    _phantom: std::marker::PhantomData<A>,
    pub(crate) handler: E,
    pub(crate) _name: String,
}

impl<A: Application, E: EventHandler> EventHandlerInstance<A, E> {
    #[tracing::instrument(name = "EventHandlerInstance", skip(builder))]
    pub fn from_builder(builder: EventHandlerBuilder<E>) -> Addr<Self> {
        Self::create(move |_ctx| {
            trace!("Register a new EventHandler instance with {}", builder.name);

            EventHandlerInstance {
                _phantom: std::marker::PhantomData,
                handler: builder.handler,
                _name: builder.name,
            }
        })
    }
}

impl<A: Application, E: EventHandler> actix::Actor for EventHandlerInstance<A, E> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        EventHandler::started(&mut self.handler, ctx);
    }
}

impl<A: Application, E: EventHandler> ::actix::Handler<ResolveAndApplyMany>
    for EventHandlerInstance<A, E>
{
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    #[tracing::instrument(name = "EventHandlerInstance", skip(self, msg, _ctx))]
    fn handle(&mut self, msg: ResolveAndApplyMany, _ctx: &mut Self::Context) -> Self::Result {
        let mut handler = self.handler.clone();
        let events = msg.0;

        let fut = async move {
            for event in events {
                EventHandler::handle_recorded_event(&mut handler, event).await?;
            }
            Ok(())
        }
        .into_actor(self)
        .map(|_res: Result<(), ()>, _actor, _ctx| Ok(()));

        Box::pin(fut)
    }
}
