use actix::Message;
use chrono::DateTime;
use serde::Serialize;
use uuid::Uuid;

/// A `RecordedEvent` represents an `Event` which have been append to a `Stream`
#[derive(sqlx::FromRow, Debug, Clone, Message, Serialize)]
#[rtype("()")]
pub struct RecordedEvent {
    /// an incrementing and gapless integer used to order the event in a stream.
    pub(crate) event_number: i64,
    /// Unique identifier for this event
    pub(crate) event_uuid: Uuid,
    /// The stream identifier for thie event
    pub stream_uuid: String,
    /// The stream version when this event was appended
    pub(crate) stream_version: Option<i64>,
    /// a `causation_id` defines who caused this event
    pub causation_id: Option<Uuid>,
    /// a `correlation_id` correlates multiple events
    pub correlation_id: Option<Uuid>,
    /// Human readable event type
    pub event_type: String,
    /// Payload of this event
    pub data: serde_json::Value,
    /// Metadata defined for this event
    pub(crate) metadata: Option<String>,
    /// Event time creation
    pub(crate) created_at: DateTime<chrono::offset::Utc>,
}

impl RecordedEvent {
    /// # Errors
    pub fn try_deserialize<
        'de,
        T: serde::Deserialize<'de> + crate::Event + serde::de::Deserialize<'de>,
    >(
        &'de self,
    ) -> Result<T, RecordedEventError> {
        match T::deserialize(&self.data) {
            Ok(e) => Ok(e),
            Err(_) => Err(RecordedEventError::Deserialize),
        }
    }
}

#[derive(Debug, Clone, Message)]
#[rtype("()")]
pub struct RecordedEvents {
    events: Vec<RecordedEvent>,
}

pub enum RecordedEventError {
    Deserialize,
}
