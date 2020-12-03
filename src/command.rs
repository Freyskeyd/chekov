use crate::event::Event;
use crate::Aggregate;
use crate::CommandExecutorError;
use crate::{event::*, message::Dispatch, Application};
use actix::ArbiterService;
use uuid::Uuid;

mod consistency;
mod metadata;

pub use consistency::Consistency;
pub use metadata::CommandMetadatas;

/// Define a Command which can be dispatch
pub trait Command: std::fmt::Debug + Send + 'static {
    /// The Event that can be generated for this command
    type Event: Event;

    /// The Executor that will execute the command and produce the events
    ///
    /// Note that for now, onlu Aggregate can be used as Executor.
    type Executor: CommandExecutor<Self> + EventApplier<Self::Event>;

    /// The registry where the command will be dispatched
    type ExecutorRegistry: ArbiterService;

    /// Returns the correlation id of the command
    fn get_correlation_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    /// Returns the optional causation id of the command
    fn get_causation_id(&self) -> Option<Uuid> {
        Some(Uuid::new_v4())
    }

    /// Returns the identifier for this command.
    ///
    /// The identifier is used to choose the right executor.
    fn identifier(&self) -> String;
}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait Dispatchable<C, A>
where
    C: Command,
    A: Application,
{
    async fn dispatch(&self, cmd: C) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        <C as Command>::ExecutorRegistry: actix::Handler<Dispatch<C, A>>;
}

/// Receives a command and an immutable Executor and optionally returns events
pub trait CommandHandler<C: Command + ?Sized> {
    fn execute(command: C, executor: &C::Executor) -> Result<Vec<C::Event>, CommandExecutorError>;
}

/// Receives a command and an immutable State and optionally returns events
pub trait CommandExecutor<T: Command + ?Sized>: Aggregate {
    fn execute(cmd: T, state: &Self) -> Result<Vec<T::Event>, CommandExecutorError>;
}
