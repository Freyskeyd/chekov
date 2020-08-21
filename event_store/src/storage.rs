use crate::event::RecordedEvent;
use crate::event::UnsavedEvent;
use crate::stream::Stream;
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
    async fn create_stream(&mut self, stream: Stream) -> Result<Stream, StorageError>;

    /// Delete a stream from the `Backend`
    ///
    /// Do we need to provide a hard/soft deletion?
    ///
    /// # Errors
    /// The stream deletion can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to delete the stream on the backend)
    /// - The stream doesn't exists
    async fn delete_stream(&mut self, stream: &Stream) -> Result<(), StorageError>;

    async fn append_to_stream(
        &mut self,
        stream_uud: &str,
        events: &[UnsavedEvent],
    ) -> Result<Vec<Uuid>, StorageError>;

    async fn read_stream(
        &mut self,
        stream_uud: &str,
        version: usize,
        limit: usize,
    ) -> Result<Vec<RecordedEvent>, StorageError>;

    async fn read_stream_info(&mut self, stream_uuid: String) -> Result<Stream, StorageError>;
}

pub mod appender;
pub mod inmemory;
pub mod postgres;
pub mod reader;

#[cfg(test)]
mod test;

#[derive(Debug, PartialEq)]
pub enum StorageError {
    StreamDoesntExists,
    StreamAlreadyExists,
    Unknown,
}
