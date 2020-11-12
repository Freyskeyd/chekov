#[derive(serde::Serialize, Debug)]
pub enum CommandExecutorError {
    Any,
}
impl std::convert::From<actix::MailboxError> for CommandExecutorError {
    fn from(_: actix::MailboxError) -> Self {
        Self::Any
    }
}
#[derive(Debug)]
pub enum ApplyError {
    Any,
}
