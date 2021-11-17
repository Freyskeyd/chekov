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
//! An `EventStore` needs a [`Storage`] that can be used to `append` and `read` events from.
//! [`Storage`] is using a `Backend` to talk to the underlying component and an `EventBus` to
//! notify and listen for events.
//!
//! Currently only two `Storage` are available:
//!
//! - [`PostgresStorage`]
//! - [`InMemoryStorage`]
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
//!         .storage(InMemoryStorage::default())
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
//!     .storage(InMemoryStorage::default())
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
//!     let mut event_store = EventStore::builder()
//!         .storage(InMemoryStorage::default())
//!         .build()
//!         .await
//!         .unwrap()
//!         .start();
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
mod event_store;
mod expected_version;
mod read_version;
pub mod storage;
mod stream;
mod subscriptions;

pub use crate::event_store::EventStore;
use error::EventStoreError;
pub use event::Event;
use event::{ParseEventError, RecordedEvent};
use expected_version::ExpectedVersion;
use read_version::ReadVersion;
use storage::{appender::Appender, reader::Reader, Storage};

#[cfg(feature = "inmemory")]
pub use storage::backend::inmemory::InMemoryBackend;
#[cfg(feature = "postgres")]
pub use storage::backend::postgres::PostgresBackend;
#[cfg(feature = "inmemory")]
pub use storage::InMemoryStorage;
#[cfg(feature = "postgres")]
pub use storage::PostgresStorage;

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

pub mod prelude {
    pub use crate::connection::StreamInfo;
    pub use crate::error::EventStoreError;
    pub use crate::event::{Event, RecordedEvent, RecordedEvents, UnsavedEvent};
    pub use crate::expected_version::ExpectedVersion;
    pub use crate::read_version::ReadVersion;
    #[cfg(feature = "inmemory_backend")]
    pub use crate::storage::backend::inmemory::InMemoryBackend;
    #[cfg(feature = "postgres_backend")]
    pub use crate::storage::backend::postgres::PostgresBackend;
    pub use crate::storage::backend::Backend;
    pub use crate::storage::event_bus::EventBus;
    #[cfg(feature = "inmemory_event_bus")]
    pub use crate::storage::event_bus::InMemoryEventBus;
    #[cfg(feature = "postgres_event_bus")]
    pub use crate::storage::event_bus::PostgresEventBus;
    #[cfg(feature = "inmemory")]
    pub use crate::storage::InMemoryStorage;
    #[cfg(feature = "postgres")]
    pub use crate::storage::PostgresStorage;
    pub use crate::storage::{appender::Appender, reader::Reader, Storage, StorageError};
    pub use crate::stream::Stream;
    pub use crate::subscriptions::Subscription;
    pub use crate::subscriptions::SubscriptionNotification;
    pub use crate::subscriptions::SubscriptionOptions;
    pub use crate::subscriptions::Subscriptions;
    pub use crate::subscriptions::SubscriptionsSupervisor;
    pub use crate::EventStore;
}
