use event_store::{core::error::BoxDynError, prelude::EventStoreError};
use thiserror::Error;
use tokio::time::error::Elapsed;

/// Error returns by a CommandExecutor
#[derive(Error, Debug)]
pub enum CommandExecutorError {
    #[error(transparent)]
    ApplyError(ApplyError),
    #[error("Internal router error")]
    InternalRouterError,
    #[error(transparent)]
    ExecutionError(BoxDynError),
    #[error(transparent)]
    EventStoreError(EventStoreError),
    #[error("Command execution timedout {0}")]
    Timedout(Elapsed),
}

impl std::convert::From<actix::MailboxError> for CommandExecutorError {
    fn from(_: actix::MailboxError) -> Self {
        Self::InternalRouterError
    }
}

impl std::convert::From<EventStoreError> for CommandExecutorError {
    fn from(error: EventStoreError) -> Self {
        Self::EventStoreError(error)
    }
}

impl std::convert::From<ApplyError> for CommandExecutorError {
    fn from(error: ApplyError) -> Self {
        Self::ApplyError(error)
    }
}

impl From<Elapsed> for CommandExecutorError {
    fn from(error: Elapsed) -> Self {
        Self::Timedout(error)
    }
}

/// Error returns by a failling EventApplier
#[derive(Error, Debug)]
pub enum ApplyError {
    #[error("Unknown")]
    Any,
}

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("Unknown")]
    Any,
}
