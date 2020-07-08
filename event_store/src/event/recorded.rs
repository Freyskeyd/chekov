use uuid::Uuid;

/// A `RecordedEvent` represents an `Event` which have been append to a `Stream`
#[derive(Debug)]
pub struct RecordedEvent {
    /// an incrementing and gapless integer used to order the event in a stream.
    pub(crate) event_number: i32,
    /// Unique identifier for this event
    pub(crate) event_uuid: Uuid,
    /// The stream identifier for thie event
    pub(crate) stream_uuid: String,
    /// The stream version when this event was appended
    pub(crate) stream_version: i32,
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
    /// Event time creation
    pub(crate) created_at: String,
}
