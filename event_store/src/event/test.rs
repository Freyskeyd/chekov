use crate::event::{unsaved::UnsavedEvent, Event};

use super::*;

#[derive(Serialize)]
pub struct MyStructEvent {}

#[derive(Serialize)]
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
}

impl Event for MyStructEvent {
    fn event_type(&self) -> &'static str {
        "MyStructEvent"
    }
}

mod unsaved {
    use super::*;

    use pretty_assertions::{assert_eq, assert_ne};

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
}
