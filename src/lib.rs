//! `Chekov` is a CQRS/ES Framework built on top of Actix actor framework.
//!
//! ## Getting started
//!
//! ### Installation
//!
//! ```toml
//! chekov = "0.1"
//! ```
//!
//! `Chekov` is using [`::event_store`] in background to interact with backend.
//!
//! ### Application
//!
//! `Chekov` is using `Application` as a way to separate usecases. You can defined multiple
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
//!     type Storage = chekov::prelude::PostgresBackend;
//! }
//! ```
//!
//! If you want to know more about [`Application`] check the module documentation.
//!
//! ### Defining our first `Aggregate`
//!
//! One can create an aggregate that will be able to hold a state and execute commands on demand.
//! Each `Aggregate` is unique and can't be duplicated meaning that you can't have the same
//! aggregate instance running twice.
//!
//! ```rust
//! #[derive(chekov::Aggregate, Default, Clone)]
//! #[aggregate(identity = "user")]
//! struct User {
//!     user_id: uuid::Uuid,
//!     name: String
//! }
//!
//! ```
//!
//! If you want to know more about [`aggregate`] check the module documentation.
//!
//! ### Creating our first `Event`
//!
//! An `Aggregate` produces events based on command execution. Defining an `Event` is pretty
//! straightforward.
//!
//! ```rust
//! # use serde::{Deserialize, Serialize};
//! #[derive(Clone, chekov::Event, Deserialize, Serialize)]
//! struct UserCreated{}
//!
//! #[derive(Clone, chekov::Event, Deserialize, Serialize)]
//! enum UserUpdated {
//!     NamedChanged(uuid::Uuid, String, String),
//!     Disabled {
//!         reason: String
//!     }
//! }
//!
//! ```
//!
//! If you want to know more about [`Event`] check the module documentation.
//!
//! ### Defining our first `Command`
//!
//! The purpose of a command is to been executed on an `Aggregate` (or a `CommandHandler` but we
//! will see that later). Each commands need to provide some basic information on how to deal with
//! them. But let's keep that simple for now:
//!
//! ```rust
//! # use chekov::prelude::*;
//! # use chekov::prelude::CommandHandler;
//! # use uuid::Uuid;
//! # use serde::{Deserialize, Serialize};
//! #
//! # #[derive(chekov::Aggregate, Default, Clone)]
//! # #[aggregate(identity = "user")]
//! # struct User {
//! #     user_id: uuid::Uuid,
//! #     name: String
//! # }
//! # #[derive(Clone, chekov::Event, Deserialize, Serialize)]
//! # struct UserCreated{}
//! # #[derive(Clone, chekov::Event, Deserialize, Serialize)]
//! # enum UserUpdated {
//! #     NamedChanged(uuid::Uuid, String, String),
//! #     Disabled {
//! #         reason: String
//! #     }
//! # }
//! # impl<T> CommandExecutor<T> for User where T: Command {
//! #   fn execute(cmd: T, state: &Self) -> Result<Vec<T::Event>, CommandExecutorError> {
//! #       Ok(vec![])
//! #   }
//! # }
//! # impl<T> EventApplier<T> for User where T: chekov::Event {
//! #   fn apply(&mut self, event: &T) -> Result<(), ApplyError> {
//! #       Ok(())
//! #   }
//! # }
//! # #[derive(Default, CommandHandler)]
//! # pub struct UserValidator{}
//! # use futures::future::BoxFuture;
//! # use futures::FutureExt;
//! # impl<T> chekov::command::Handler<T, User> for UserValidator where T: Command {
//! #     fn handle(&mut self, command: T, state: chekov::aggregate::StaticState<User>) -> BoxFuture<'static, ExecutionResult<T::Event>> {
//! #         let events = User::execute(command, &state);
//!
//! #         async { events }.boxed()
//! #     }
//! # }
//! #[derive(Clone, Debug, chekov::Command, Serialize, Deserialize)]
//! #[command(event = "UserCreated", aggregate = "User", handler = "UserValidator")]
//!struct CreateUser {
//!     #[command(identifier)]
//!     pub user_id: Uuid,
//!     pub name: String,
//! }
//!
//! #[derive(Clone, Debug, chekov::Command, Serialize, Deserialize)]
//! #[command(event = "UserUpdated", aggregate = "User")]
//! struct UpdateUser {
//!     #[command(identifier)]
//!     pub user_id: Uuid,
//!     pub name: String,
//! }
//!
//! ```

pub mod aggregate;
pub mod application;
pub mod command;
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
#[doc(inline)]
pub use command::CommandHandler;
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
pub use chekov_macros::command_handler;
pub use chekov_macros::event_handler;
pub use chekov_macros::Aggregate;
pub use chekov_macros::Command;
pub use chekov_macros::CommandHandler;
pub use chekov_macros::Event;
pub use chekov_macros::EventHandler;

#[doc(hidden)]
pub use async_trait;
#[doc(hidden)]
pub use inventory;

#[cfg(test)]
mod tests;
