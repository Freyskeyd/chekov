use thiserror::Error;

pub enum EventBusError {
    EventNotificationError(EventNotificationError),
}

#[derive(Error, Debug)]
pub enum EventNotificationError {
    #[error("Unable to parse {field}")]
    ParsingError { field: &'static str },
    #[error("Invalid stream_uuid")]
    InvalidStreamUUID,
}
