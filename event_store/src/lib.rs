#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]
#![allow(clippy::module_name_repetitions)]
#![allow(dead_code)]

mod connection;
mod error;
mod event;
mod expected_version;
mod storage;
mod stream;

use actix::{Actor, Addr};
use connection::{Append, Connection, CreateStream, StreamInfo};
use error::EventStoreError;
use event::{Event, ParseEventError, UnsavedEvent};
use expected_version::ExpectedVersion;
use log::{debug, info, trace, warn};
use std::borrow::Cow;
use storage::{appender::Appender, Storage};
use uuid::Uuid;

pub struct EventStore<S: Storage> {
    connection: Addr<Connection<S>>,
}

impl<S> EventStore<S>
where
    S: 'static + Storage + std::marker::Unpin,
{
    #[must_use]
    pub fn builder() -> EventStoreBuilder<S> {
        EventStoreBuilder { storage: None }
    }

    /// # Errors
    pub(crate) async fn create_stream<T: Into<String> + Send>(
        &self,
        stream_uuid: T,
    ) -> Result<Cow<'static, crate::stream::Stream>, EventStoreError> {
        let stream_uuid: String = stream_uuid.into();
        debug!("Creating stream {}", stream_uuid);

        self.connection.send(CreateStream(stream_uuid)).await?
    }

    pub(crate) async fn stream_info<T: Into<String> + Send>(
        &self,
        stream_uuid: T,
    ) -> Result<Cow<'static, crate::stream::Stream>, EventStoreError> {
        let stream_uuid: String = stream_uuid.into();
        debug!("Asking for stream {} infos", stream_uuid);

        self.connection.send(StreamInfo(stream_uuid)).await?
    }

    /// # Errors
    pub async fn append_to_stream<T: Into<String> + Send>(
        &self,
        stream: T,
        expected_version: ExpectedVersion,
        events: Vec<UnsavedEvent>,
    ) -> Result<Vec<Uuid>, EventStoreError> {
        let stream: String = stream.into();

        #[cfg(feature = "verbose")]
        let events_number = {
            info!(
                "Attempting to append {} event(s) to {} with ExpectedVersion::{:?}",
                events.len(),
                stream,
                expected_version
            );

            events.len()
        };

        let res = self
            .connection
            .send(Append {
                #[cfg(feature = "verbose")]
                stream: stream.clone(),
                #[cfg(not(feature = "verbose"))]
                stream,
                expected_version,
                events,
            })
            .await?;

        #[cfg(feature = "verbose")]
        if let Ok(ref events_ids) = res {
            info!("Appended {} event(s) to {}", events_ids.len(), stream)
        } else {
            info!("Failed to append {} event(s) to {}", events_number, stream)
        }

        res
    }
}

#[must_use]
pub fn append() -> Appender {
    Appender::default()
}

pub struct EventStoreBuilder<S: Storage> {
    storage: Option<S>,
}

impl<S> EventStoreBuilder<S>
where
    S: Storage,
{
    pub fn storage(mut self, storage: S) -> Self {
        self.storage = Some(storage);

        self
    }

    /// # Errors
    pub async fn build(self) -> Result<EventStore<S>, ()> {
        if self.storage.is_none() {
            return Err(());
        }

        trace!("Creating EventStore with {} storage", S::storage_name());
        Ok(EventStore {
            connection: Connection::make(self.storage.unwrap()).start(),
        })
    }
}

pub mod prelude {
    pub use crate::event::{Event, UnsavedEvent};
    pub use crate::expected_version::ExpectedVersion;
    pub use crate::storage::{
        inmemory::InMemoryBackend, postgres::PostgresBackend, AppendToStreamError, Storage,
    };
    pub use crate::stream::Stream;
    pub use crate::EventStore;
}
