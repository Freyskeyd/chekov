use crate::connection::Connection;
use crate::error::EventStoreError;
pub use crate::event::Event;
use actix::prelude::*;
use event_store_core::storage::Storage;
use tracing::{instrument, trace};

mod logic;
mod runtime;

/// An `EventStore` that hold a storage connection
#[derive(Debug, Clone)]
pub struct EventStore<S: Storage> {
    connection: Addr<Connection<S>>,
}

/// Builder use to simplify the `EventStore` creation
#[derive(Debug)]
pub struct EventStoreBuilder<S: Storage> {
    storage: Option<S>,
}

impl<S: Storage> EventStoreBuilder<S> {
    /// Define which storage will be used by this building `EventStore`
    pub fn storage(mut self, storage: S) -> Self {
        self.storage = Some(storage);

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
        match self.storage {
            Some(storage) => {
                let connection = Connection::make(storage).start();
                trace!("Creating EventStore with {} storage", S::storage_name());

                Ok(EventStore { connection })
            }
            None => Err(EventStoreError::NoStorage),
        }
    }
}
