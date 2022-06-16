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
//! # use serde::{Deserialize, Serialize};
//! # use uuid::Uuid;
//! # use actix::Message;
//! #
//! # #[derive(Debug, Clone, Serialize)]
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
//! #     type Storage = PostgresStorage;
//! # }
//! #
//! # #[derive(Clone, Message, Debug, chekov::Event, Deserialize, Serialize)]
//! # #[rtype(result = "Result<(), ()>")]
//! # pub struct AccountOpened {
//! #     pub account_id: Uuid,
//! #     pub name: String,
//! # }
//! #
//! # #[derive(Clone, Debug, chekov::Command, Serialize, Deserialize)]
//! # #[command(event = "AccountOpened", aggregate = "Account")]
//! # pub struct OpenAccount {
//! #     #[command(identifier)]
//! #     pub account_id: Uuid,
//! #     pub name: String,
//! # }
//! #
//! #[derive(Debug, Clone, Default, Aggregate)]
//! #[aggregate(identity = "account")]
//! pub struct Account {
//!     account_id: Option<uuid::Uuid>,
//!     name: String,
//!     status: AccountStatus
//! }
//!
//! // Executing commands
//! impl CommandExecutor<OpenAccount> for Account {
//!     fn execute(cmd: OpenAccount, state: &Self) -> ExecutionResult<AccountOpened> {
//!         match state.status {
//!             AccountStatus::Initialized => Ok(vec![AccountOpened {
//!                 account_id: cmd.account_id,
//!                 name: cmd.name,
//!             }]),
//!             _ => Err(CommandExecutorError::InternalRouterError),
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

use crate::{prelude::ApplyError, Application};
use event_store::prelude::RecordedEvent;

#[doc(hidden)]
pub mod resolver;

#[doc(hidden)]
pub use instance::AggregateInstance;
#[doc(hidden)]
pub use registry::AggregateInstanceRegistry;

use self::resolver::EventResolverRegistry;

/// Define an Aggregate
///
/// We don't recommend implementing this trait directly. use the `Aggregate` derive macro instead
///
///
/// ```rust
/// # use chekov::prelude::*;
///
/// #[derive(Debug, Clone, Default, Aggregate)]
/// #[aggregate(identity = "account")]
/// struct Account {
///     account_id: Option<uuid::Uuid>
/// }
/// ```
///
pub trait Aggregate: Send + Clone + Default + std::marker::Unpin + 'static {
    #[doc(hidden)]
    fn get_event_resolver() -> &'static EventResolverRegistry<Self>;

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
        _stream: &str,
        _ctx: &actix::Context<AggregateInstance<Self>>,
    ) {
    }
}

// Type that represent a `StaticState`. The `State` will not be used if modified.
pub type StaticState<A> = A;
