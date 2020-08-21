use crate::storage::StorageError;
use crate::ParseEventError;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum EventStoreError {
    Any,
    Storage(StorageError),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn testing_that_error_can_be_displayed() {
        println!("{}", EventStoreError::Any);
    }

    #[test]
    fn testing_that_a_mailboxerror_can_be_converted() {
        let err = actix::MailboxError::Closed;

        let _c: EventStoreError = err.into();
    }

    #[test]
    fn testing_that_a_parse_event_error_can_be_converted() {
        let err = crate::ParseEventError::UnknownFailure;

        let _: EventStoreError = err.into();
    }
}
