//! Define Backend structs and traits to build a backend storage.
//!
//! To be used by a storage a struct need to implement the `Backend` trait.

use std::pin::Pin;

use futures::Future;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    event::{RecordedEvent, UnsavedEvent},
    event_bus::EventBusMessage,
    storage::StorageError,
    stream::Stream,
};

/// Result type of backend action.
pub type BackendResult<T> = Pin<Box<dyn Future<Output = Result<T, StorageError>> + Send>>;

mod error;
pub use error::BackendError;

/// Used by the `Storage` to interact with the underlying `Backend`.
///
/// A backend is an abstraction of the underlying persistency layer which is used by a `Storage`.
///
/// It aims to provide generic API to manage stream of events.
///
pub trait Backend {
    /// Define the name of the backend implementation. This name is mostly used as a context name
    /// for tracing and logging.
    fn backend_name() -> &'static str;

    #[doc(hidden)]
    fn direct_channel(&mut self, _notifier: mpsc::UnboundedSender<EventBusMessage>) {}

    /// Create a new stream with an identifier
    ///
    /// # Errors
    /// The stream creation can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to create the stream on the persistency layer)
    /// - The stream already exists
    fn create_stream(&mut self, stream: Stream, correlation_id: Uuid) -> BackendResult<Stream>;

    /// Delete a stream from the `Backend`
    ///
    /// Do we need to provide a hard/soft deletion?
    ///
    /// # Errors
    /// The stream deletion can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to delete the stream on the persistency layer)
    /// - The stream doesn't exists
    fn delete_stream(&mut self, stream: &Stream, correlation_id: Uuid) -> BackendResult<()>;

    /// Append a list of `UnsavedEvent` to the defined `stream_uuid`.
    ///
    /// # Errors
    /// The append can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to delete the stream on the persistency layer)
    /// - The stream doesn't exists
    /// - event number doesn't match the expected one
    fn append_to_stream(
        &mut self,
        stream_uuid: &str,
        events: &[UnsavedEvent],
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<Uuid>, StorageError>> + Send>>;

    /// Read a stream forward
    ///
    /// # Errors
    /// The read can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to delete the stream on the persistency layer)
    /// - The stream doesn't exists
    fn read_stream(
        &self,
        stream_uud: String,
        version: usize,
        limit: usize,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<RecordedEvent>, StorageError>> + Send>>;

    /// Read the stream's metadatas without fetching events.
    fn read_stream_info(
        &mut self,
        stream_uuid: String,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Stream, StorageError>> + Send>>;
}
