#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::toplevel_ref_arg)]
#![allow(clippy::similar_names)]
#![allow(dead_code)]
//! # `EventStore`
//!
//! The `EventStore` will allow you to deal with every aspects of the event sourcing part of Chekov.
//!
//! An `EventStore` needs a [`Storage`] that can be used to `append` and `reead` events from.
//! [`Storage`] is using a `Backend` to talk to the underlying component.
//!
//! Currently only two `Backend` are available:
//!
//! - [`PostgresBackend`]
//! - [`InMemoryBackend`]
//!
//!
//! ## Construct the `EventStore`
//!
//! An `EventStore` is an actor that receive messages to interact with the storage. To create an
//! `EventStore` you need to provide a valid struct that implement [`Storage`].
//!
//! ```rust
//!
//! use event_store::prelude::*;
//! use actix::Actor;
//!
//! #[actix::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>>{
//!     let addr: actix::Addr<EventStore<_>> = EventStore::builder()
//!         .storage(InMemoryBackend::default())
//!         .build()
//!         .await?
//!         .start();
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Event
//!
//! The [`Event`] trait can be used on `struct` and `enum` to define type that can be serialize and
//! append/read to and from the eventstore.
//!
//!
//! ```rust
//! use event_store::prelude::*;
//! use uuid::Uuid;
//! use serde::{Deserialize, Serialize};
//! use std::convert::TryFrom;
//!
//! #[derive(Deserialize, Serialize)]
//! struct MyEvent {
//!     account_id: Uuid
//! }
//!
//! impl Event for MyEvent {
//!     // This method is used by the system to define a human readble representation of the Event.
//!     // For enum, each variant must be a unique `str`
//!     fn event_type(&self) -> &'static str {
//!         "MyEvent"
//!     }
//!
//!     // Returns every human readable name that this type can be decoded to.
//!     fn all_event_types() -> Vec<&'static str> {
//!         vec!["MyEvent"]
//!     }
//! }
//!
//! impl TryFrom<RecordedEvent> for MyEvent {
//!      type Error = ();
//!      fn try_from(e: RecordedEvent) -> Result<Self, ()> {
//!        serde_json::from_value(e.data).map_err(|_| ())
//!      }
//! }

//! ```
//!
//! ## Appending an event
//!
//! [`Event`] can be append by using the fluent API exposed at the root level of the `event_store` crate:
//!
//! ```rust
//! use event_store::prelude::*;
//! use uuid::Uuid;
//! # use std::convert::TryFrom;
//! use actix::Actor;
//! #
//! # #[derive(serde::Deserialize, serde::Serialize)]
//! # struct MyEvent{
//! #     account_id: Uuid
//! # }
//! # impl Event for MyEvent {
//! #     fn event_type(&self) -> &'static str { "MyEvent" }
//! #     fn all_event_types() -> Vec<&'static str> { vec!["MyEvent"] }
//! # }
//! # impl TryFrom<RecordedEvent> for MyEvent {
//! #     type Error = ();
//! #      fn try_from(e: RecordedEvent) -> Result<Self, ()> {
//! #        serde_json::from_value(e.data).map_err(|_| ())
//! #      }
//! # }
//!
//! #[actix::main]
//! async fn reading() -> Result<(), Box<dyn std::error::Error>>{
//! let event_store = EventStore::builder()
//!     .storage(InMemoryBackend::default())
//!     .build()
//!     .await?
//!     .start();
//!
//! let stream_uuid = Uuid::new_v4().to_string();
//! let my_event = MyEvent { account_id: Uuid::new_v4() };
//!
//! event_store::append()
//!   .event(&my_event)?
//!   .to(&stream_uuid)?
//!   .execute(event_store)
//!   .await;
//!
//!   Ok(())
//! }
//! ```
//!
//! ## Reading from stream
//!
//! A `Stream` can be read with the fluent API exposed at the root level of the `event_store` crate:
//!
//! ```rust
//! use event_store::prelude::*;
//! use uuid::Uuid;
//! use actix::Actor;
//!
//! #[actix::main]
//! async fn reading() -> Result<(), Box<dyn std::error::Error>>{
//!     let mut event_store = EventStore::builder().storage(InMemoryBackend::default()).build().await.unwrap().start();
//!
//!     let stream_uuid = Uuid::new_v4().to_string();
//!
//!     event_store::read()
//!         .stream(&stream_uuid)?
//!         .from(ReadVersion::Origin)
//!         .limit(10)
//!         .execute(event_store)
//!         .await;
//!     Ok(())
//! }
//! ```

mod connection;
mod error;
mod event;
mod expected_version;
mod read_version;
mod storage;
mod stream;
mod subscriptions;

use actix::prelude::*;
use connection::{Append, Connection, CreateStream, Read, StreamInfo};
use error::EventStoreError;
pub use event::Event;
use event::{ParseEventError, RecordedEvent};
use expected_version::ExpectedVersion;
use tracing::{debug, info, instrument, trace, warn};

use read_version::ReadVersion;
use std::borrow::Cow;
pub use storage::inmemory::InMemoryBackend;
pub use storage::postgres::PostgresBackend;
use storage::{appender::Appender, reader::Reader, Storage};
use tracing_futures::Instrument;
use uuid::Uuid;

/// An `EventStore` that hold a storage connection
#[derive(Clone)]
pub struct EventStore<S: Storage> {
    connection: Addr<Connection<S>>,
}

impl<S: Storage> std::default::Default for EventStore<S> {
    fn default() -> Self {
        unimplemented!()
    }
}

impl<S> ::actix::Supervised for EventStore<S> where S: 'static + Storage + std::marker::Unpin {}
impl<S> ::actix::Actor for EventStore<S>
where
    S: 'static + Storage + std::marker::Unpin,
{
    type Context = ::actix::Context<Self>;
}

impl<S: Storage> Handler<storage::reader::ReadStreamRequest> for EventStore<S> {
    type Result = actix::ResponseActFuture<Self, Result<Vec<RecordedEvent>, EventStoreError>>;

    #[tracing::instrument(name = "EventStore::ReadStreamRequest", skip(self, e, _ctx), fields(correlation_id = %e.correlation_id))]
    fn handle(
        &mut self,
        e: storage::reader::ReadStreamRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let stream: String = e.stream.to_string();

        info!("Attempting to read {} stream event(s)", stream);

        let connection = self.connection.clone();

        let fut = async move {
            match connection
                .send(Read {
                    correlation_id: e.correlation_id,
                    #[cfg(feature = "verbose")]
                    stream: stream.clone(),
                    #[cfg(not(feature = "verbose"))]
                    stream,
                    version: e.version,
                    limit: e.limit,
                })
                .await?
            {
                Ok(events) => {
                    #[cfg(feature = "verbose")]
                    info!("Read {} event(s) to {}", events.len(), stream);
                    Ok(events)
                }
                Err(e) => {
                    #[cfg(feature = "verbose")]
                    info!("Failed to read event(s) from {}", stream);
                    Err(e)
                }
            }
        }
        .instrument(tracing::Span::current())
        .into_actor(self);

        Box::pin(fut)
    }
}

impl<S: Storage> Handler<StreamInfo> for EventStore<S> {
    type Result = actix::ResponseActFuture<
        Self,
        Result<Cow<'static, crate::stream::Stream>, EventStoreError>,
    >;

    #[tracing::instrument(name = "EventStore::StreamInfo", skip(self, e, _ctx), fields(correlation_id = %e.correlation_id))]
    fn handle(&mut self, e: StreamInfo, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Asking for stream {} infos", e.stream_uuid);

        let connection = self.connection.clone();
        let fut = async move { connection.send(e).await? }.into_actor(self);

        Box::pin(fut)
    }
}

impl<S: Storage> Handler<CreateStream> for EventStore<S> {
    type Result = actix::ResponseActFuture<
        Self,
        Result<Cow<'static, crate::stream::Stream>, EventStoreError>,
    >;

    #[tracing::instrument(name = "EventStore::CreateStream", skip(self, e, _ctx), fields(correlation_id = %e.correlation_id))]
    fn handle(&mut self, e: CreateStream, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Creating stream {}", e.stream_uuid);

        let connection = self.connection.clone();
        let fut = async move { connection.send(e).await? }.into_actor(self);

        Box::pin(fut)
    }
}

impl<S: Storage> Handler<storage::appender::AppendToStreamRequest> for EventStore<S> {
    type Result = actix::ResponseActFuture<Self, Result<Vec<Uuid>, EventStoreError>>;

    #[tracing::instrument(name = "EventStore::AppendToStream", skip(self, e, _ctx), fields(correlation_id = %e.correlation_id))]
    fn handle(
        &mut self,
        e: storage::appender::AppendToStreamRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let stream: String = e.stream.to_string();

        #[cfg(feature = "verbose")]
        let events_number = e.events.len();
        info!(
            "Attempting to append {} event(s) to {} with ExpectedVersion::{:?}",
            e.events.len(),
            e.stream,
            e.expected_version
        );

        let connection = self.connection.clone();
        let fut = async move {
            match connection
                .send(Append {
                    correlation_id: e.correlation_id,
                    #[cfg(feature = "verbose")]
                    stream: stream.clone(),
                    #[cfg(not(feature = "verbose"))]
                    stream,
                    expected_version: e.expected_version,
                    events: e.events,
                })
                .await?
            {
                Ok(events) => {
                    #[cfg(feature = "verbose")]
                    info!("Appended {} event(s) to {}", events.len(), stream);
                    Ok(events)
                }
                Err(e) => {
                    #[cfg(feature = "verbose")]
                    info!("Failed to append {} event(s) to {}", events_number, stream);
                    Err(e)
                }
            }
        }
        .instrument(tracing::Span::current())
        .into_actor(self);

        Box::pin(fut)
    }
}

impl<S> EventStore<S>
where
    S: 'static + Storage + std::marker::Unpin,
{
    #[must_use]
    pub fn builder() -> EventStoreBuilder<S> {
        EventStoreBuilder { storage: None }
    }
}

/// Create an `Appender` to append events
#[must_use]
pub fn append() -> Appender {
    Appender::default()
}

/// Create a `Reader` to read a stream
#[must_use]
pub fn read() -> Reader {
    Reader::default()
}

/// Builder use to simplify the `EventStore` creation
#[derive(Debug)]
pub struct EventStoreBuilder<S: Storage> {
    storage: Option<S>,
}

impl<S> EventStoreBuilder<S>
where
    S: Storage,
{
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
        if self.storage.is_none() {
            return Err(EventStoreError::NoStorage);
        }

        trace!("Creating EventStore with {} storage", S::storage_name());
        Ok(EventStore {
            connection: Connection::make(self.storage.unwrap()).start(),
        })
    }
}

pub mod prelude {
    pub use crate::connection::StreamInfo;
    pub use crate::error::EventStoreError;
    pub use crate::event::{Event, RecordedEvent, RecordedEvents, UnsavedEvent};
    pub use crate::expected_version::ExpectedVersion;
    pub use crate::read_version::ReadVersion;
    pub use crate::storage::{
        appender::Appender, inmemory::InMemoryBackend, postgres::PostgresBackend, reader::Reader,
        Storage, StorageError,
    };
    pub use crate::stream::Stream;
    pub use crate::EventStore;
}
