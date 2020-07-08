use crate::ParseEventError;
use std::fmt;

#[derive(Debug)]
pub enum EventStoreError {
    Any,
}

impl fmt::Display for EventStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EventStore Error!")
    }
}
impl std::error::Error for EventStoreError {}

impl std::convert::From<actix::MailboxError> for EventStoreError {
    fn from(_: actix::MailboxError) -> Self {
        Self::Any
    }
}

impl std::convert::From<ParseEventError> for EventStoreError {
    fn from(_: ParseEventError) -> Self {
        Self::Any
    }
}
