use thiserror::Error;

use crate::{event::UnsavedEventError, storage::StorageError};

#[derive(Error, Debug)]
pub enum EventStoreError {
    #[error("No storage is defined")]
    NoStorage,
    #[error("The streamId is invalid")]
    InvalidStreamId,
    #[error(transparent)]
    Storage(StorageError),
    #[error(transparent)]
    EventProcessing(UnsavedEventError),
    #[error("Internal event store error: {0}")]
    InternalEventStoreError(#[source] BoxDynError),
}

impl From<StorageError> for EventStoreError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

impl std::convert::From<UnsavedEventError> for EventStoreError {
    fn from(e: UnsavedEventError) -> Self {
        Self::EventProcessing(e)
    }
}

type BoxDynError = Box<dyn std::error::Error + 'static + Send + Sync>;

#[cfg(feature = "actix-rt")]
impl From<actix::MailboxError> for EventStoreError {
    fn from(error: actix::MailboxError) -> Self {
        Self::InternalEventStoreError(Box::new(error))
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn testing_that_a_mailboxerror_can_be_converted() {
        let err = actix::MailboxError::Closed;

        let _c: EventStoreError = err.into();
    }

    #[test]
    fn testing_that_a_parse_event_error_can_be_converted() {
        let error = serde_json::from_str::<'_, String>("{").unwrap_err();
        let err = UnsavedEventError::SerializeError(error);

        let _: EventStoreError = err.into();
    }

    #[test]
    fn an_event_store_error_can_be_dispayed() {
        let err = EventStoreError::InvalidStreamId;

        assert_eq!("The streamId is invalid", err.to_string());
    }
}
