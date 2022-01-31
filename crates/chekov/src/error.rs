use event_store::prelude::EventStoreError;

/// Error returns by a CommandExecutor
#[derive(serde::Serialize, PartialEq, Debug)]
pub enum CommandExecutorError {
    ApplyError,
    Any,
}

impl std::convert::From<actix::MailboxError> for CommandExecutorError {
    fn from(_: actix::MailboxError) -> Self {
        Self::Any
    }
}

impl std::convert::From<EventStoreError> for CommandExecutorError {
    fn from(_: EventStoreError) -> Self {
        Self::Any
    }
}

impl std::convert::From<ApplyError> for CommandExecutorError {
    fn from(_: ApplyError) -> Self {
        Self::ApplyError
    }
}

/// Error returns by a failling EventApplier
#[derive(Debug)]
pub enum ApplyError {
    Any,
}

pub enum HandleError {
    Any,
}
