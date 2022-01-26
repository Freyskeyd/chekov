use super::*;
use serde::Serialize;

#[derive(Serialize, serde::Deserialize)]
pub struct MyStructEvent {}

#[derive(Serialize, serde::Deserialize)]
pub enum MyEnumEvent {
    Created { id: i32 },
    Updated(String),
    Deleted,
}

impl Event for MyEnumEvent {
    fn event_type(&self) -> &'static str {
        match *self {
            Self::Deleted => "MyEnumEvent::Deleted",
            Self::Created { .. } => "MyEnumEvent::Created",
            Self::Updated(_) => "MyEnumEvent::Updated",
        }
    }

    fn all_event_types() -> Vec<&'static str> {
        vec![
            "MyEnumEvent::Deleted",
            "MyEnumEvent::Collect",
            "MyEnumEvent::Updated",
        ]
    }
}

impl Event for MyStructEvent {
    fn event_type(&self) -> &'static str {
        "MyStructEvent"
    }

    fn all_event_types() -> Vec<&'static str> {
        vec!["MyStructEvent"]
    }
}

impl std::convert::TryFrom<RecordedEvent> for MyEnumEvent {
    type Error = ();
    fn try_from(e: RecordedEvent) -> Result<Self, Self::Error> {
        serde_json::from_value(e.data).map_err(|_| ())
    }
}
impl std::convert::TryFrom<RecordedEvent> for MyStructEvent {
    type Error = ();
    fn try_from(e: RecordedEvent) -> Result<Self, Self::Error> {
        serde_json::from_value(e.data).map_err(|_| ())
    }
}

mod unsaved {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn must_have_a_valide_event_type() {
        let source_events = vec![
            MyEnumEvent::Created { id: 1 },
            MyEnumEvent::Updated("Updated".into()),
            MyEnumEvent::Deleted,
        ];

        let mut produces_events: Vec<UnsavedEvent> = source_events
            .iter()
            .map(UnsavedEvent::try_from)
            .filter(Result::is_ok)
            .map(Result::unwrap)
            .collect();

        let next = MyStructEvent {};

        produces_events.push(UnsavedEvent::try_from(&next).unwrap());

        let expected = vec![
            "MyEnumEvent::Created",
            "MyEnumEvent::Updated",
            "MyEnumEvent::Deleted",
            "MyStructEvent",
        ];

        assert_eq!(expected.len(), produces_events.len());
        expected
            .into_iter()
            .zip(produces_events)
            .for_each(|(ex, real)| assert_eq!(ex, real.event_type));
    }

    #[test]
    fn test_that_serde_error_are_handled() {
        use serde::ser::Error;
        let err = ParseEventError::from(serde_json::Error::custom("test"));

        let _: ParseEventError = err.into();
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct MyEvent(pub String);

    impl Event for MyEvent {
        fn event_type(&self) -> &'static str {
            "MyEvent"
        }

        fn all_event_types() -> Vec<&'static str> {
            vec!["MyEvent"]
        }
    }

    impl std::convert::TryFrom<RecordedEvent> for MyEvent {
        type Error = ();
        fn try_from(e: RecordedEvent) -> Result<Self, Self::Error> {
            serde_json::from_value(e.data).map_err(|_| ())
        }
    }

    #[test]
    fn test_that_ids_can_be_setted() {
        let event = MyEvent("Hello".into());
        let _unsaved = match UnsavedEvent::try_from(&event) {
            Ok(unsaved) => {
                assert!(unsaved.causation_id.is_none());
                assert!(unsaved.correlation_id.is_none());
                assert_eq!(unsaved.event_type, "MyEvent");
                assert_eq!(unsaved.event_type, event.event_type());
                assert_eq!(unsaved.data, serde_json::Value::String("Hello".into()));
                assert_eq!(unsaved.metadata, json!({}));
                unsaved
            }
            Err(_) => panic!("Couldnt convert into UnsavedEvent"),
        };

        let _expected = UnsavedEvent {
            causation_id: None,
            correlation_id: None,
            event_type: "MyEvent".into(),
            data: "\"Hello\"".into(),
            metadata: json!({}),
            event_uuid: Uuid::new_v4(),
            stream_uuid: String::new(),
            stream_version: 0,
            created_at: chrono::offset::Utc::now(),
        };
    }
}
