use uuid::Uuid;

/// A `RecordedEvent` represents an `Event` which have been append to a `Stream`
pub struct RecordedEvent {
    /// an incrementing and gapless integer used to order the event in a stream.
    event_number: i32,
    /// Unique identifier for this event
    event_uuid: Uuid,
    /// The stream identifier for thie event
    stream_uuid: Uuid,
    /// The stream version when this event was appended
    stream_version: i32,
    /// a `causation_id` defines who caused this event
    causation_id: Option<Uuid>,
    /// a `correlation_id` correlates multiple events
    correlation_id: Option<Uuid>,
    /// Human readable event type
    event_type: String,
    /// Payload of this event
    data: String,
    /// Metadata defined for this event
    metadata: String,
    /// Event time creation
    created_at: String,
}
