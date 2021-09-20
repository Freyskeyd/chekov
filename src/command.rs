use crate::aggregate::StaticState;
use crate::event::Event;
use crate::Aggregate;
use crate::CommandExecutorError;
use crate::{event::*, message::Dispatch, Application};
use actix::SystemService;

mod consistency;
mod handler;
mod metadata;

pub use consistency::Consistency;
use futures::future::BoxFuture;
pub(crate) use handler::instance::CommandHandlerInstance;
pub use handler::NoHandler;
pub use metadata::CommandMetadatas;

/// Define a Command which can be dispatch
pub trait Command: std::fmt::Debug + Send + 'static {
    /// The Event that can be generated for this command
    type Event: Event + event_store::Event;

    /// The Executor that will execute the command and produce the events
    ///
    /// Note that for now, onlu Aggregate can be used as Executor.
    type Executor: CommandExecutor<Self> + EventApplier<Self::Event>;

    /// The registry where the command will be dispatched
    type ExecutorRegistry: SystemService;

    type CommandHandler: CommandHandler + Handler<Self, Self::Executor>;

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
pub trait CommandHandler: std::marker::Unpin + Default + 'static + SystemService {}

pub trait Handler<C: Command + ?Sized, A: CommandExecutor<C>> {
    fn handle(
        &mut self,
        command: C,
        state: StaticState<A>,
    ) -> BoxFuture<'static, Result<Vec<C::Event>, CommandExecutorError>>;
}

/// Receives a command and an immutable State and optionally returns events
pub trait CommandExecutor<T: Command + ?Sized>: Aggregate {
    fn execute(cmd: T, state: &Self) -> Result<Vec<T::Event>, CommandExecutorError>;
}
