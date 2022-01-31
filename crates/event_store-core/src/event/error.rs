use thiserror::Error;

/// Errors related to a recorded event
#[derive(Error, Debug)]
pub enum RecordedEventError {
    #[error("Unable to deserialize the recorded event")]
    DeserializeError(serde_json::Error),
}

/// Errors related to a unsaved event
#[derive(Error, Debug)]
pub enum UnsavedEventError {
    #[error("Unable to serialize the event")]
    SerializeError(serde_json::Error),
}

impl From<serde_json::Error> for UnsavedEventError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerializeError(e)
    }
}
impl From<serde_json::Error> for RecordedEventError {
    fn from(e: serde_json::Error) -> Self {
        Self::DeserializeError(e)
    }
}
