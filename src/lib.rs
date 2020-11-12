use actix::prelude::*;
pub use chekov_macros as macros;
pub use event_store;
use event_store::prelude::*;
pub use event_store::Event;
use log::trace;

mod aggregate;
pub mod aggregate_registry;
mod command;
mod command_executor;
mod command_handler;
mod error;
mod event_applier;
mod message;
mod registry;
mod router;

pub mod prelude;

pub use aggregate::Aggregate;
pub use command::Command;
use command_executor::CommandExecutor;
use error::CommandExecutorError;
use event_applier::EventApplier;
use message::Dispatch;
use router::Router;

pub struct EventHandlerBuilder<E: EventHandler> {
    handler: E,
    name: String,
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

pub mod event {
    #[async_trait::async_trait]
    pub trait Handler<E: event_store::Event> {
        async fn handle(&self, event: &E) -> Result<(), ()>;
    }
}

struct EventHandlerInstance<E: EventHandler> {
    handler: E,
    subscribtion: Addr<Subscriber>,
}

impl<E: EventHandler> actix::Actor for EventHandlerInstance<E> {
    type Context = Context<Self>;
}

pub struct Chekov<B: event_store::prelude::Storage> {
    pub _event_store: event_store::EventStore<B>,
    _router: Addr<router::Router<B>>,
}

pub struct Subscriber {
    stream: String,
    subscriber: Recipient<message::EventEnvelope>,
}

impl Subscriber {
    fn new(subscriber: Recipient<message::EventEnvelope>, stream: &str) -> Addr<Self> {
        trace!("Create a new Subscriber for {}", stream);
        Subscriber::create(move |ctx| Subscriber {
            stream: stream.to_string(),
            subscriber,
        })
    }
}

impl actix::Actor for Subscriber {
    type Context = Context<Self>;
}

impl<E: EventHandler> actix::Handler<message::EventEnvelope> for EventHandlerInstance<E> {
    type Result = actix::ResponseActFuture<Self, Result<usize, ()>>;
    fn handle(&mut self, _event: message::EventEnvelope, _: &mut Self::Context) -> Self::Result {
        Box::pin(async { Ok(1) }.into_actor(self))
    }
}

impl<B> Chekov<B>
where
    B: event_store::prelude::Storage,
{
    pub async fn dispatch<C: Command>(&self, cmd: C) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        <C as Command>::ExecutorRegistry: actix::Handler<Dispatch<C, B>>,
    {
        self._router
            .send(Dispatch::<C, B> {
                storage: std::marker::PhantomData,
                command: cmd,
            })
            .await?
    }

    pub async fn with_storage(storage: B) -> Self {
        trace!(
            "Creating a new Chekov instance with {}",
            std::any::type_name::<B>()
        );

        let event_store = EventStore::builder()
            .storage(storage)
            .build()
            .await
            .unwrap();

        ::actix::Arbiter::set_item(event_store.duplicate());

        let router = Router {
            _event_store: event_store.duplicate(),
            _before_dispatch: vec![],
        };

        let addr = router.start();

        ::actix::Registry::set(addr.clone());

        Self {
            _event_store: event_store,
            _router: addr,
        }
    }

    pub async fn register_event_handler<E: EventHandler>(&self, handler: EventHandlerBuilder<E>) {
        EventHandlerInstance::create(move |ctx| {
            trace!("Register a new EventHandler instance with {}", handler.name);
            let ctx_address = ctx.address();

            EventHandlerInstance {
                subscribtion: Subscriber::new(ctx_address.recipient(), "$all"),
                handler: handler.handler,
            }
        });
    }
}

#[cfg(test)]
mod test;
