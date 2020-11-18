use crate::event::RecordedEvent;
use crate::event::UnsavedEvent;
use crate::stream::Stream;
use futures::Future;
use uuid::Uuid;

/// A `Storage` is responsible for storing and managing `Stream` and `Event`for a `Backend`
pub trait Storage: Default + Send + std::marker::Unpin + 'static {
    fn storage_name() -> &'static str;
    /// Create a new stream with an identifier
    ///
    /// # Errors
    /// The stream creation can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to create the stream on the backend)
    /// - The stream already exists
    fn create_stream(
        &mut self,
        stream: Stream,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Stream, StorageError>> + Send>>;

    /// Delete a stream from the `Backend`
    ///
    /// Do we need to provide a hard/soft deletion?
    ///
    /// # Errors
    /// The stream deletion can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to delete the stream on the backend)
    /// - The stream doesn't exists
    fn delete_stream(
        &mut self,
        stream: &Stream,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send>>;

    fn append_to_stream(
        &mut self,
        stream_uud: &str,
        events: &[UnsavedEvent],
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<Uuid>, StorageError>> + Send>>;

    // async fn read_stream(
    fn read_stream(
        &self,
        stream_uud: String,
        version: usize,
        limit: usize,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<RecordedEvent>, StorageError>> + Send>>;

    fn read_stream_info(
        &mut self,
        stream_uuid: String,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Stream, StorageError>> + Send>>;
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
