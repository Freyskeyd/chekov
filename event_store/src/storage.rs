use crate::stream::{Stream, StreamError};

/// A `Storage` is responsible for storing and managing `Stream` and `Event`for a `Backend`
pub trait Storage {
    /// Create a new stream with an identifier
    ///
    /// # Errors
    /// The stream creation can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to create the stream on the backend)
    /// - The stream already exists
    fn create_stream(&mut self, stream: Stream) -> Result<&Stream, StreamCreationError>;

    /// Delete a stream from the `Backend`
    ///
    /// Do we need to provide a hard/soft deletion?
    ///
    /// # Errors
    /// The stream deletion can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to delete the stream on the backend)
    /// - The stream doesn't exists
    fn delete_stream(&mut self, stream: &Stream) -> Result<(), StreamDeletionError>;
}

mod inmemory;

#[cfg(test)]
mod test;

#[derive(Debug, PartialEq)]
pub enum StreamCreationError {
    AlreadyExists,
    StreamError(StreamError),
    StorageError(StorageError),
}

impl std::convert::From<StreamError> for StreamCreationError {
    fn from(e: StreamError) -> Self {
        Self::StreamError(e)
    }
}

#[derive(Debug, PartialEq)]
pub enum StreamDeletionError {
    DoesntExists,
    StreamError(StreamError),
    StorageError(StorageError),
}

impl std::convert::From<StreamError> for StreamDeletionError {
    fn from(e: StreamError) -> Self {
        Self::StreamError(e)
    }
}

#[derive(Debug, PartialEq)]
pub enum StorageError {
    Unknown,
}
