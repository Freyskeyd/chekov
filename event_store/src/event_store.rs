use crate::connection::Connection;
use crate::error::EventStoreError;
pub use crate::event::Event;
use crate::event_bus::EventBus;
use crate::event_bus::EventBusConnection;
use crate::storage::Storage;
use actix::prelude::*;
use tracing::{instrument, trace};

mod logic;
mod runtime;

/// An `EventStore` that hold a storage connection
#[derive(Clone)]
pub struct EventStore<S: Storage> {
    connection: Addr<Connection<S>>,
    event_bus: Addr<EventBusConnection<S>>,
}

/// Builder use to simplify the `EventStore` creation
#[derive(Debug)]
pub struct EventStoreBuilder<S: Storage, E: EventBus> {
    storage: Option<S>,
    event_bus: Option<E>,
}

impl<S: Storage, E: EventBus> EventStoreBuilder<S, E> {
    /// Define which storage will be used by this building `EventStore`
    pub fn storage(mut self, storage: S) -> Self {
        self.storage = Some(storage);

        self
    }

    /// Define which event bus will be used by this building `EventStore`
    pub fn event_bus(mut self, event_bus: E) -> Self {
        self.event_bus = Some(event_bus);

        self
    }

    /// Try to build the previously configured `EventStore`
    ///
    /// # Errors
    ///
    /// For now this method can fail only if you haven't define a `Storage`
    // #[instrument(level = "trace", name = "my_name", skip(self))]
    #[instrument(level = "trace", name = "EventStoreBuilder::build", skip(self))]
    pub async fn build(self) -> Result<EventStore<S>, EventStoreError> {
        if self.storage.is_none() {
            return Err(EventStoreError::NoStorage);
        }
        let connection = Connection::make(self.storage.unwrap()).start();
        let event_bus = EventBusConnection::make(self.event_bus.unwrap(), connection.clone());
        trace!("Creating EventStore with {} storage", S::storage_name());

        Ok(EventStore {
            connection,
            event_bus,
        })
    }
}
