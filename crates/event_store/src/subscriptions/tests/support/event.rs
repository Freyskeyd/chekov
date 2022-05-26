use std::convert::TryFrom;

use event_store_core::event::{Event, RecordedEvent};
use serde::{Deserialize, Serialize};

pub struct EventFactory {}

impl EventFactory {
    pub fn create_event(number: usize) -> TestEvent {
        TestEvent { event: number }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TestEvent {
    event: usize,
}

impl Event for TestEvent {
    fn event_type(&self) -> &'static str {
        "TestEvent"
    }

    fn all_event_types() -> Vec<&'static str> {
        vec!["TestEvent"]
    }
}

impl TryFrom<RecordedEvent> for TestEvent {
    type Error = ();

    fn try_from(e: RecordedEvent) -> Result<Self, Self::Error> {
        serde_json::from_value(e.data).map_err(|_| ())
    }
}

#[derive(serde::Serialize)]
pub(crate) struct MyEvent {}
impl Event for MyEvent {
    fn event_type(&self) -> &'static str {
        "MyEvent"
    }

    fn all_event_types() -> Vec<&'static str> {
        vec!["MyEvent"]
    }
}

impl TryFrom<RecordedEvent> for MyEvent {
    type Error = ();

    fn try_from(_: RecordedEvent) -> Result<Self, Self::Error> {
        Ok(MyEvent {})
    }
}
