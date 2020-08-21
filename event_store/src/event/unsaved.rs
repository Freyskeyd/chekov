use super::Event;
use chrono::{DateTime, Utc};
use uuid::Uuid;
/// An `UnsavedEvent` is created from a type that implement Event
///
/// This kind of event represents an unsaved event, meaning that it has less information
/// than a `RecordedEvent`. It's a generic form to simplify the event processing but also a way to
/// define `metadata`, `causation_id` and `correlation_id`
#[derive(Debug, Clone, PartialEq)]
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
    pub(crate) stream_version: i64,
    pub(crate) created_at: DateTime<chrono::offset::Utc>,
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
    /// # Errors
    /// If `serde` isn't able to serialize the `Event`
    pub fn try_from<E: Event>(event: &E) -> Result<Self, ParseEventError> {
        Ok(Self {
            causation_id: None,
            correlation_id: None,
            event_type: event.event_type().to_owned(),
            data: serde_json::to_string(&event)?,
            metadata: String::from("{}"),
            event_uuid: Uuid::new_v4(),
            stream_uuid: String::new(),
            stream_version: 0,
            created_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_that_serde_error_are_handled() {
        use serde::ser::Error;
        let err = ParseEventError::from(serde_json::Error::custom("test"));

        let _: ParseEventError = err.into();
    }

    #[derive(serde::Serialize)]
    struct MyEvent(pub String);

    impl Event for MyEvent {
        fn event_type(&self) -> &'static str {
            "MyEvent"
        }
    }

    #[test]
    fn test_that_ids_can_be_setted() {
        let event = MyEvent("Hello".into());
        let unsaved = match UnsavedEvent::try_from(&event) {
            Ok(unsaved) => {
                assert!(unsaved.causation_id.is_none());
                assert!(unsaved.correlation_id.is_none());
                assert_eq!(unsaved.event_type, "MyEvent");
                assert_eq!(unsaved.event_type, event.event_type());
                assert_eq!(unsaved.data, "\"Hello\"");
                assert_eq!(unsaved.metadata, "{}");
                unsaved
            }
            Err(_) => panic!("Couldnt convert into UnsavedEvent"),
        };

        let expected = UnsavedEvent {
            causation_id: None,
            correlation_id: None,
            event_type: "MyEvent".into(),
            data: "\"Hello\"".into(),
            metadata: String::new(),
            event_uuid: Uuid::new_v4(),
            stream_uuid: String::new(),
            stream_version: 0,
            created_at: chrono::offset::Utc::now(),
        };
    }
}
