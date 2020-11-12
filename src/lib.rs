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

pub struct Chekov<B: event_store::prelude::Storage> {
    pub _event_store: event_store::EventStore<B>,
    _router: Addr<router::Router<B>>,
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
}

#[cfg(test)]
mod test;
