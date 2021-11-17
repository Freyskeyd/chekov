use self::backend::Backend;
use self::event_bus::BoxedStream;
use crate::prelude::EventBus;
use crate::storage::event_bus::EventBusMessage;
use tokio::sync::mpsc;

/// A `Storage` is responsible for storing and managing `Stream` and `Event`for a `Backend`
pub trait Storage: std::fmt::Debug + Default + Send + std::marker::Unpin + 'static {
    type Backend: Backend;
    type EventBus: EventBus;

    fn storage_name() -> &'static str;

    #[doc(hidden)]
    fn direct_channel(&mut self, _notifier: mpsc::UnboundedSender<EventBusMessage>) {}

    fn create_stream(&mut self) -> BoxedStream;
    fn backend(&mut self) -> &mut Self::Backend;
}

#[derive(Debug, PartialEq)]
pub enum StorageError {
    StreamDoesntExists,
    StreamAlreadyExists,
    Unknown,
}

pub mod appender;
pub mod backend;
pub mod event_bus;
#[cfg(feature = "inmemory")]
mod inmemory;
#[cfg(feature = "inmemory")]
pub use inmemory::InMemoryStorage;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::PostgresStorage;

pub mod reader;

#[cfg(test)]
mod test;
