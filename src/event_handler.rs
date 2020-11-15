use crate::message::EventEnvelope;
use crate::subscriber::Subscriber;
use crate::Chekov;
use actix::prelude::*;

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

    pub async fn register<T: event_store::prelude::Storage>(self, app: &Chekov<T>) {
        app.register_event_handler(self).await
    }
}

pub trait EventHandler: Sized + std::marker::Unpin + 'static {
    fn builder(self) -> EventHandlerBuilder<Self> {
        EventHandlerBuilder::new(self)
    }
}

pub struct EventHandlerInstance<E: EventHandler> {
    pub(crate) _handler: E,
    pub(crate) _subscribtion: Addr<Subscriber>,
}

impl<E: EventHandler> actix::Actor for EventHandlerInstance<E> {
    type Context = Context<Self>;
}

impl<E: EventHandler> actix::Handler<EventEnvelope> for EventHandlerInstance<E> {
    type Result = actix::ResponseActFuture<Self, Result<usize, ()>>;
    fn handle(&mut self, _event: EventEnvelope, _: &mut Self::Context) -> Self::Result {
        Box::pin(async { Ok(1) }.into_actor(self))
    }
}
