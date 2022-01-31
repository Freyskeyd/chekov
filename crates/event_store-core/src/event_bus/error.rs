use thiserror::Error;

// Convenience type alias for usage within event_store.
type BoxDynError = Box<dyn std::error::Error + 'static + Send + Sync>;

#[derive(Error, Debug)]
pub enum EventBusError {
    #[error("Notification error: {0}")]
    EventNotificationError(EventNotificationError),
    #[error("Internal storage error: {0}")]
    InternalEventBusError(#[source] BoxDynError),
}

#[derive(Error, Debug)]
pub enum EventNotificationError {
    #[error("Unable to parse {field}")]
    ParsingError { field: &'static str },
    #[error("Invalid stream_uuid")]
    InvalidStreamUUID,
}
