use futures::Future;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    event::{RecordedEvent, UnsavedEvent},
    event_bus::{BoxedStream, EventBus, EventBusMessage},
    stream::Stream,
};

/// A `Storage` is responsible for storing and managing `Stream` and `Event`for a `Backend`
pub trait Storage: std::fmt::Debug + Default + Send + std::marker::Unpin + 'static {
    type Backend: Backend;
    type EventBus: EventBus;

    fn storage_name() -> &'static str;

    #[doc(hidden)]
    fn direct_channel(&mut self, _notifier: mpsc::UnboundedSender<EventBusMessage>) {}

    fn create_stream(&mut self) -> BoxedStream;
    fn backend(&mut self) -> &mut Self::Backend;
}

#[derive(Debug, PartialEq)]
pub enum StorageError {
    StreamDoesntExists,
    StreamAlreadyExists,
    Unknown,
}

pub trait Backend {
    fn backend_name() -> &'static str;

    #[doc(hidden)]
    fn direct_channel(&mut self, _notifier: mpsc::UnboundedSender<EventBusMessage>) {}

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
        correlation_id: Uuid,
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
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send>>;

    fn append_to_stream(
        &mut self,
        stream_uud: &str,
        events: &[UnsavedEvent],
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<Uuid>, StorageError>> + Send>>;

    // async fn read_stream(
    fn read_stream(
        &self,
        stream_uud: String,
        version: usize,
        limit: usize,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<RecordedEvent>, StorageError>> + Send>>;

    fn read_stream_info(
        &mut self,
        stream_uuid: String,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Stream, StorageError>> + Send>>;
}
