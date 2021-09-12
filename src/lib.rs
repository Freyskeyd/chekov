//! Chekov is a CQRS/ES Framework built on top of Actix actor framework.
//!
//! ## Getting started
//!
//! ### Application
//!
//! Chekov is using `Application` as a way to separate usecases. You can defined multiple
//! applications on the same runtime which can be useful to synchronise multiple domains.
//!
//! But first, let's define our first `Application`:
//!
//! ```rust
//! #[derive(Default)]
//! struct DefaultApp {}
//!
//! // Application trait is here to preconfigure your chekov runtime.
//! // It tells that you want this `Application` to use a PostgresBackend and resolve the
//! // eventbus's event with the `DefaultEventResolver`.
//! impl chekov::Application for DefaultApp {
//!     type Storage = event_store::prelude::PostgresBackend;
//! }
//! ```

pub mod aggregate;
pub mod application;
mod command;
mod error;
pub mod event;
pub mod event_store;
#[doc(hidden)]
pub mod message;
pub mod prelude;
mod router;
mod subscriber;

#[doc(hidden)]
pub use lazy_static::lazy_static;

pub use ::event_store::prelude::RecordedEvent;
#[doc(inline)]
pub use aggregate::Aggregate;
#[doc(inline)]
pub use application::Application;
#[doc(inline)]
pub use application::ApplicationBuilder;
#[doc(inline)]
pub use command::Command;
use error::CommandExecutorError;
#[doc(inline)]
pub use event::Event;
#[doc(inline)]
pub use event::EventApplier;
#[doc(inline)]
pub use event::EventHandler;
use message::Dispatch;
use router::Router;
pub use subscriber::SubscriberManager;

pub use chekov_macros::applier;
pub use chekov_macros::event_handler;
pub use chekov_macros::Aggregate;
pub use chekov_macros::Command;
pub use chekov_macros::Event;
pub use chekov_macros::EventHandler;

#[doc(hidden)]
pub use async_trait;
#[doc(hidden)]
pub use inventory;

#[cfg(test)]
mod tests;
