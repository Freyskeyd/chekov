use super::Event;
use uuid::Uuid;

/// An `UnsavedEvent` is created from a type that implement Event
///
/// This kind of event represents an unsaved event, meaning that it has less information
/// than a `RecordedEvent`. It's a generic form to simplify the event processing but also a way to
/// define `metadata`, `causation_id` and `correlation_id`
#[derive(Debug, Clone)]
pub struct UnsavedEvent {
    /// a `causation_id` defines who caused this event
    pub(crate) causation_id: Option<Uuid>,
    /// a `correlation_id` correlates multiple events
    pub(crate) correlation_id: Option<Uuid>,
    /// Human readable event type
    pub(crate) event_type: String,
    /// Payload of this event
    pub(crate) data: String,
    /// Metadata defined for this event
    pub(crate) metadata: String,
    pub(crate) event_uuid: Uuid,
    pub(crate) stream_uuid: String,
    pub(crate) stream_version: i32,
    pub(crate) created_at: String,
}

#[derive(Debug)]
pub enum ParseEventError {
    UnknownFailure,
}

impl From<serde_json::Error> for ParseEventError {
    fn from(_: serde_json::Error) -> Self {
        Self::UnknownFailure
    }
}

impl UnsavedEvent {
    pub fn try_from<E: Event>(event: &E) -> Result<Self, ParseEventError> {
        Ok(Self {
            causation_id: None,
            correlation_id: None,
            event_type: event.event_type().to_owned(),
            data: serde_json::to_string(&event)?,
            metadata: String::new(),
            event_uuid: Uuid::new_v4(),
            stream_uuid: String::new(),
            stream_version: 0,
            created_at: String::new(),
        })
    }
}
