use actix::Message;
use chrono::DateTime;
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

pub mod error;

pub use error::*;

#[cfg(test)]
mod test;

/// Represent event that can be handled by an `EventStore`
pub trait Event: Serialize + Send + std::convert::TryFrom<RecordedEvent> {
    /// Returns a `'static str` which defines the event type
    ///
    /// This `str` must be as precise as possible.
    fn event_type(&self) -> &'static str;

    /// Returns every possible string representations of the event.
    ///
    /// Useful to define particular variant types for an enum
    fn all_event_types() -> Vec<&'static str>;
}

/// A `RecordedEvent` represents an `Event` which have been append to a `Stream`
#[derive(sqlx::FromRow, Debug, Clone, Message, Serialize)]
#[rtype(result = "()")]
pub struct RecordedEvent {
    /// an incrementing and gapless integer used to order the event in a stream.
    #[sqlx(try_from = "i64")]
    pub event_number: u64,
    /// Unique identifier for this event
    pub event_uuid: Uuid,
    /// The stream identifier for thie event
    pub stream_uuid: String,
    /// The stream version when this event was appended
    pub stream_version: Option<i64>,
    /// a `causation_id` defines who caused this event
    pub causation_id: Option<Uuid>,
    /// a `correlation_id` correlates multiple events
    pub correlation_id: Option<Uuid>,
    /// Human readable event type
    pub event_type: String,
    /// Payload of this event
    pub data: serde_json::Value,
    /// Metadata defined for this event
    pub metadata: Option<String>,
    /// Event time creation
    pub created_at: DateTime<chrono::offset::Utc>,
}

impl RecordedEvent {
    /// # Errors
    pub fn try_deserialize<
        'de,
        T: serde::Deserialize<'de> + Event + serde::de::Deserialize<'de>,
    >(
        &'de self,
    ) -> Result<T, RecordedEventError> {
        Ok(T::deserialize(&self.data)?)
    }
}

/// An `UnsavedEvent` is created from a type that implement `Event`
///
/// This kind of event represents an unsaved event, meaning that it has less informations
/// than a `RecordedEvent`. It's a generic form to simplify the event processing but also a way to
/// define `metadata`, `causation_id` and `correlation_id`.
#[derive(Debug, Clone, PartialEq)]
pub struct UnsavedEvent {
    /// a `causation_id` defines who caused this event
    pub causation_id: Option<Uuid>,
    /// a `correlation_id` correlates multiple events
    pub correlation_id: Option<Uuid>,
    /// Human readable event type
    pub event_type: String,
    /// Payload of this event
    pub data: serde_json::Value,
    /// Metadata defined for this event
    pub metadata: serde_json::Value,
    pub event_uuid: Uuid,
    pub stream_uuid: String,
    pub stream_version: u64,
    pub created_at: DateTime<chrono::offset::Utc>,
}

impl UnsavedEvent {
    /// Try to create an `UnsavedEvent` from a struct that implement `Event`.
    ///
    /// In case of a success an `UnsavedEvent` is returned with no context or metadata.
    ///
    /// # Errors
    /// If `serde` isn't able to serialize the `Event` an `UnsavedEventError::SerializeError` is
    /// returned
    pub fn try_from<E: Event>(event: &E) -> Result<Self, UnsavedEventError> {
        Ok(Self {
            causation_id: None,
            correlation_id: None,
            event_type: event.event_type().to_owned(),
            data: serde_json::to_value(event)?,
            metadata: json!({}),
            event_uuid: Uuid::new_v4(),
            stream_uuid: String::new(),
            stream_version: 0,
            created_at: Utc::now(),
        })
    }
}
