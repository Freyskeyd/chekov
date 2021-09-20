use crate as chekov;
use crate::prelude::*;
use event_store::prelude::InMemoryBackend;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub(crate) struct MyApplication {}

impl Application for MyApplication {
    type Storage = InMemoryBackend;
}

#[derive(Clone, Aggregate, Default, Debug)]
#[aggregate(identity = "example")]
pub(crate) struct ExampleAggregate {
    items: Vec<i64>,
    last_index: usize,
}

impl CommandExecutor<AppendItem> for ExampleAggregate {
    fn execute(cmd: AppendItem, _: &Self) -> Result<Vec<ItemAppended>, CommandExecutorError> {
        Ok(vec![ItemAppended(cmd.0)])
    }
}

impl EventApplier<ItemAppended> for ExampleAggregate {
    fn apply(&mut self, event: &ItemAppended) -> Result<(), ApplyError> {
        self.items.push(event.0);

        self.last_index = self.items.len() - 1;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct AppendItem(pub(crate) i64);

impl Command for AppendItem {
    type Event = ItemAppended;

    type Executor = ExampleAggregate;

    type ExecutorRegistry = AggregateInstanceRegistry<ExampleAggregate>;

    type CommandHandler = NoHandler;

    fn identifier(&self) -> String {
        "example_aggregate".to_string()
    }
}

#[derive(Clone, Debug, crate::Event, Deserialize, Serialize)]
pub(crate) struct ItemAppended(i64);

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
