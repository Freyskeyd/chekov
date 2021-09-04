//! Aggregates produce events based on commands
//!
//! An `Aggregate` is a simple Rust struct which will hold the state of the `Aggregate`.
//! The `Default` trait is mandatory because of the way the `Aggregate` works.
//!
//! An `Aggregate` is a simple struct and a bunch of functions that react to events or commands
//! to alter the state. The state of an aggregate must remain private and accessible only
//! by the aggregate itself. (Not even his parent). This prevent coupling between aggregates
//! which is a bad practice.
//!
//! An `Aggregate` must have a **unique** identity accros all `Aggregate` to perform well. See [`identity`](trait.Aggregate.html#tymethod.identity).
//!
//! An `Aggregate` will start with a blank state (this is why it needs `Default`). It will then use
//! `EventApplier` to receive and apply domain events on his state. You don't need to worry about
//! event number, stream and all the complexity of the event management, all of it are handle by
//! the parent process which is automatically generated. You just have to decide whether you apply
//! the event to the state or not. An `Aggregate` can't apply multiple event at a time meaning that
//! their is no concurrency state alteration.
//!
//! An `Aggregate` will also be the root producer of all events. `Aggregates` generate events based on
//! [`Command`](../trait.Command.html) which are pushed to him. An `Aggregate` can't execute multiple commands at a time.
//!
//! # Examples
//!
//! ```rust
//! # use chekov::prelude::*;
//! # use chekov::application::DefaultEventResolver;
//! # use event_store::prelude::*;
//! # use chekov::macros::*;
//! # use serde::{Deserialize, Serialize};
//! # use uuid::Uuid;
//! # use actix::Message;
//! #
//! # #[derive(Debug, Serialize)]
//! # pub enum AccountStatus {
//! #     Initialized,
//! #     Active,
//! #     Deleted,
//! # }
//! # impl std::default::Default for AccountStatus {
//! #     fn default() -> Self {
//! #        Self::Initialized
//! #     }
//! # }
//! #
//! # #[derive(Default)]
//! # struct DefaultApp {}
//! #
//! # impl chekov::Application for DefaultApp {
//! #     type Storage = PostgresBackend;
//! #     type EventResolver = DefaultEventResolver<Self>;
//! # }
//! #
//! # #[derive(Clone, Message, Debug, chekov::macros::Event, Deserialize, Serialize)]
//! # #[rtype(result = "Result<(), ()>")]
//! # pub struct AccountOpened {
//! #     pub account_id: Uuid,
//! #     pub name: String,
//! # }
//! #
//! # #[derive(Clone, Debug, chekov::macros::Command, Serialize, Deserialize)]
//! # #[command(event = "AccountOpened", aggregate = "Account")]
//! # pub struct OpenAccount {
//! #     #[command(identifier)]
//! #     pub account_id: Uuid,
//! #     pub name: String,
//! # }
//! #
//! #[derive(Debug, Default, Aggregate)]
//! #[aggregate(identity = "account")]
//! pub struct Account {
//!     account_id: Option<uuid::Uuid>,
//!     name: String,
//!     status: AccountStatus
//! }
//!
//! // Executing commands
//! impl CommandExecutor<OpenAccount> for Account {
//!     fn execute(
//!         cmd: OpenAccount,
//!         state: &Self
//!     ) -> Result<Vec<AccountOpened>, CommandExecutorError> {
//!         match state.status {
//!             AccountStatus::Initialized => Ok(vec![AccountOpened {
//!                 account_id: cmd.account_id,
//!                 name: cmd.name,
//!             }]),
//!             _ => Err(CommandExecutorError::Any),
//!         }
//!     }
//! }
//!
//!
//! #[chekov::applier]
//! impl EventApplier<AccountOpened> for Account {
//!     fn apply(&mut self, event: &AccountOpened) -> Result<(), ApplyError> {
//!         self.account_id = Some(event.account_id);
//!         self.status = AccountStatus::Active;
//!
//!         Ok(())
//!     }
//! }
//! ```

mod instance;
mod registry;

use actix::AsyncContext;
use actix::SystemService;
use event_store::prelude::RecordedEvent;
pub use instance::AggregateInstance;

#[doc(hidden)]
pub use registry::AggregateInstanceRegistry;

use crate::message::ResolveAndApplyMany;
use crate::{event::handler::Subscribe, prelude::ApplyError, Application};

#[doc(hidden)]
pub trait EventResolverItem<A: Aggregate> {
    fn get_names(&self) -> &[&'static str];
}

/// Define an Aggregate
///
/// We don't recommend implementing this trait directly. use the `Aggregate` derive macro instead
///
///
/// ```rust
/// # use chekov::prelude::*;
/// # use chekov::macros::*;
///
/// #[derive(Debug, Default, Aggregate)]
/// #[aggregate(identity = "account")]
/// struct Account {
///     account_id: Option<uuid::Uuid>
/// }
/// ```
///
pub trait Aggregate: std::fmt::Debug + Default + std::marker::Unpin + 'static {
    fn apply_recorded_event(&mut self, event: RecordedEvent) -> Result<(), ApplyError>;

    /// Define the identity of this kind of Aggregate.
    ///
    /// The identity is concatenated to the stream_uuid to create the stream_name of this
    /// aggregate.
    ///
    /// Defining the identity as `account` will create streams `account-UUID`.
    fn identity() -> &'static str;

    #[doc(hidden)]
    fn on_start<A: Application>(
        &mut self,
        stream: &str,
        ctx: &actix::Context<AggregateInstance<Self>>,
    ) {
        let broker = crate::subscriber::SubscriberManager::<A>::from_registry();
        let recipient = ctx.address().recipient::<ResolveAndApplyMany>();
        broker.do_send(Subscribe(stream.into(), recipient));
    }
}
