use tokio::sync::mpsc;

use crate::{
    backend::Backend,
    event_bus::{BoxedStream, EventBus, EventBusMessage},
};

pub use self::error::StorageError;
pub mod error;

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
