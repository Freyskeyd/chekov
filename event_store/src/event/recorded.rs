use chrono::DateTime;
use uuid::Uuid;

/// A `RecordedEvent` represents an `Event` which have been append to a `Stream`
#[derive(Debug, Clone)]
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
    pub(crate) causation_id: Option<Uuid>,
    /// a `correlation_id` correlates multiple events
    pub(crate) correlation_id: Option<Uuid>,
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
    /// `()` for now
    pub fn try_deserialize<
        'de,
        T: serde::Deserialize<'de> + crate::Event + serde::de::Deserialize<'de>,
    >(
        &'de self,
    ) -> Result<T, ()> {
        match T::deserialize(&self.data) {
            Ok(e) => Ok(e),
            Err(e) => {
                println!("{:?}", e);

                Err(())
            }
        }
    }
}
