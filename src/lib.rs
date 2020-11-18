use actix::prelude::*;
pub use chekov_macros as macros;
pub use event_store;
pub use event_store::Event;

mod aggregate;
pub mod aggregate_registry;
pub mod application;
mod command;
mod command_executor;
mod command_handler;
mod error;
mod event_applier;
pub mod event_handler;
mod message;
pub mod prelude;
mod registry;
mod router;
mod subscriber;

pub use aggregate::Aggregate;
pub use application::Application;
pub use command::Command;
use command_executor::CommandExecutor;
use error::CommandExecutorError;
use event_applier::EventApplier;
use event_handler::EventHandler;
use event_handler::EventHandlerBuilder;
use message::Dispatch;
use router::Router;
use subscriber::SubscriberManager;

pub mod event {
    #[async_trait::async_trait]
    pub trait Handler<E: event_store::Event> {
        async fn handle(&self, event: &E) -> Result<(), ()>;
    }
}

#[cfg(test)]
mod test;
