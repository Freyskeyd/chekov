use crate::event::UnsavedEvent;
use crate::stream::{Stream, StreamError};
use uuid::Uuid;

/// A `Storage` is responsible for storing and managing `Stream` and `Event`for a `Backend`
#[async_trait::async_trait]
pub trait Storage: Send + std::marker::Unpin + 'static {
    fn storage_name() -> &'static str;
    /// Create a new stream with an identifier
    ///
    /// # Errors
    /// The stream creation can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to create the stream on the backend)
    /// - The stream already exists
    async fn create_stream(&mut self, stream: Stream) -> Result<Stream, StreamCreationError>;

    /// Delete a stream from the `Backend`
    ///
    /// Do we need to provide a hard/soft deletion?
    ///
    /// # Errors
    /// The stream deletion can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to delete the stream on the backend)
    /// - The stream doesn't exists
    async fn delete_stream(&mut self, stream: &Stream) -> Result<(), StreamDeletionError>;

    async fn append_to_stream(
        &mut self,
        stream_uud: &str,
        events: &[UnsavedEvent],
    ) -> Result<Vec<Uuid>, AppendToStreamError>;

    async fn read_stream_info(&mut self, stream_uuid: String) -> Result<Stream, StorageError>;
}

pub mod appender;
pub mod inmemory;
pub mod postgres;

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
pub enum AppendToStreamError {
    DoesntExists,
    StreamError(StreamError),
    StorageError(StorageError),
}

impl std::convert::From<StreamError> for AppendToStreamError {
    fn from(e: StreamError) -> Self {
        Self::StreamError(e)
    }
}
#[derive(Debug, PartialEq)]
pub enum StorageError {
    StreamDoesntExists,
    Unknown,
}
