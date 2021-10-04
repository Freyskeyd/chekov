use event_store::prelude::EventStoreError;

/// Error returns by a CommandExecutor
#[derive(serde::Serialize, PartialEq, Debug)]
pub enum CommandExecutorError {
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

/// Error returns by a failling EventApplier
#[derive(Debug)]
pub enum ApplyError {
    Any,
}
