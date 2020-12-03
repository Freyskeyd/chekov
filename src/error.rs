/// Error returns by a CommandExecutor
#[derive(serde::Serialize, Debug)]
pub enum CommandExecutorError {
    Any,
}
impl std::convert::From<actix::MailboxError> for CommandExecutorError {
    fn from(_: actix::MailboxError) -> Self {
        Self::Any
    }
}

/// Error returns by a failling EventApplier
#[derive(Debug)]
pub enum ApplyError {
    Any,
}
