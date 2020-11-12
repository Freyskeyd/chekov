use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct MyCommand(pub String);

#[async_trait::async_trait]
impl Command for MyCommand {
    type Event = MyEvent;
    type Executor = AggTest;
    type ExecutorRegistry = crate::aggregate_registry::AggregateInstanceRegistry<AggTest>;
    fn identifier(&self) -> String {
        self.0.clone()
    }
    async fn dispatch(&self) -> Result<(), ()> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct MyEvent(String);
impl Event for MyEvent {
    fn event_type(&self) -> &'static str {
        "MyEvent"
    }
}

impl std::convert::TryFrom<event_store::prelude::RecordedEvent> for MyEvent {
    type Error = ();
    fn try_from(e: event_store::prelude::RecordedEvent) -> Result<Self, Self::Error> {
        serde_json::from_value(e.data).map_err(|_| ())
    }
}

#[derive(Default)]
struct AggTest {
    handled_event: Option<String>,
}
impl Aggregate for AggTest {
    fn identity() -> &'static str {
        "agg_test"
    }
}

impl CommandExecutor<MyCommand> for AggTest {
    fn execute(
        cmd: MyCommand,
        _state: &Self,
    ) -> Result<Vec<<MyCommand as Command>::Event>, CommandExecutorError> {
        Ok(vec![MyEvent(cmd.0)])
    }
}

impl EventApplier<MyEvent> for AggTest {
    fn apply(&mut self, event: &MyEvent) -> Result<(), crate::error::ApplyError> {
        self.handled_event = Some(event.0.clone());
        Ok(())
    }
}

#[test]
fn test_command_executor() {
    let _cmd = MyCommand("a command".into());

    let _reg = crate::registry::Registry::default();

    let _agg = AggTest {
        handled_event: None,
    };
}
