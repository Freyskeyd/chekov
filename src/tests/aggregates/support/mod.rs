use crate as chekov;
use crate::prelude::*;
use event_store::prelude::InMemoryStorage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default)]
pub(crate) struct MyApplication {}

impl Application for MyApplication {
    type Storage = InMemoryStorage;
}

#[derive(Clone, Aggregate, Default, Debug)]
#[aggregate(identity = "example")]
pub(crate) struct ExampleAggregate {
    items: Vec<i64>,
    last_index: usize,
}

impl CommandExecutor<ValidCommand> for ExampleAggregate {
    fn execute(cmd: ValidCommand, _: &Self) -> ExecutionResult<MyEvent> {
        ExecutionResult::Ok(vec![MyEvent { id: cmd.0 }])
    }
}

impl CommandExecutor<InvalidCommand> for ExampleAggregate {
    fn execute(_cmd: InvalidCommand, _: &Self) -> ExecutionResult<InvalidEvent> {
        ExecutionResult::Err(CommandExecutorError::Any)
    }
}

impl CommandExecutor<AppendItem> for ExampleAggregate {
    fn execute(cmd: AppendItem, _: &Self) -> ExecutionResult<ItemAppended> {
        ExecutionResult::Ok(vec![ItemAppended(cmd.0)])
    }
}

#[crate::applier]
impl EventApplier<ItemAppended> for ExampleAggregate {
    fn apply(&mut self, event: &ItemAppended) -> Result<(), ApplyError> {
        self.items.push(event.0);

        self.last_index = self.items.len() - 1;
        Ok(())
    }
}

#[crate::applier]
impl EventApplier<InvalidEvent> for ExampleAggregate {
    fn apply(&mut self, _event: &InvalidEvent) -> Result<(), ApplyError> {
        Err(ApplyError::Any)
    }
}

#[crate::applier]
impl EventApplier<MyEvent> for ExampleAggregate {
    fn apply(&mut self, _event: &MyEvent) -> Result<(), ApplyError> {
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ValidCommand(pub(crate) Uuid);
impl Command for ValidCommand {
    type Event = MyEvent;

    type Executor = ExampleAggregate;

    type ExecutorRegistry = AggregateInstanceRegistry<ExampleAggregate>;

    type CommandHandler = NoHandler;

    fn identifier(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug)]
pub(crate) struct InvalidCommand(pub(crate) Uuid);
impl Command for InvalidCommand {
    type Event = InvalidEvent;

    type Executor = ExampleAggregate;

    type ExecutorRegistry = AggregateInstanceRegistry<ExampleAggregate>;

    type CommandHandler = NoHandler;

    fn identifier(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug)]
pub(crate) struct AppendItem(pub(crate) i64, pub(crate) Uuid);

impl Command for AppendItem {
    type Event = ItemAppended;

    type Executor = ExampleAggregate;

    type ExecutorRegistry = AggregateInstanceRegistry<ExampleAggregate>;

    type CommandHandler = NoHandler;

    fn identifier(&self) -> String {
        self.1.to_string()
    }
}

#[derive(Clone, Debug, crate::Event, Deserialize, Serialize)]
pub(crate) struct ItemAppended(pub(crate) i64);

#[derive(Clone, PartialEq, Debug, crate::Event, Deserialize, Serialize)]
#[event(event_type = "MyEvent")]
pub(crate) struct MyEvent {
    pub(crate) id: Uuid,
}

#[derive(Clone, PartialEq, Debug, crate::Event, Deserialize, Serialize)]
#[event(event_type = "InvalidEvent")]
pub(crate) struct InvalidEvent {
    pub(crate) id: Uuid,
}

#[macro_export]
macro_rules! assert_aggregate_version {
    ($instance: ident, $number: expr) => {
        let value = $instance.send(crate::message::AggregateVersion).await?;

        assert_eq!(
            value, $number,
            "Aggregate versions doesn't match => current: {}, expected: {}",
            value, $number
        );
    };
}
