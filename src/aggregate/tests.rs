use std::{collections::BTreeMap, convert::TryFrom};

use event_store::InMemoryBackend;
use uuid::Uuid;

use crate::{
    aggregate::AggregateInstanceRegistry,
    command::{CommandExecutor, ExecutionResult, NoHandler},
    prelude::CommandExecutorError,
    Command, Event, EventApplier,
};

use super::*;

crate::lazy_static! {
    #[derive(Clone, Debug)]
    static ref REGISTRY: EventResolverRegistry<MyAggregate> = {
        EventResolverRegistry {
            names: BTreeMap::new(),
            appliers: BTreeMap::new(),
        }
    };
}

#[derive(Clone, Default)]
pub(crate) struct MyAggregate {}
impl Aggregate for MyAggregate {
    fn get_event_resolver() -> &'static EventResolverRegistry<Self> {
        &REGISTRY
    }

    fn identity() -> &'static str {
        "my_aggregate"
    }
}

impl EventApplier<MyEvent> for MyAggregate {
    fn apply(&mut self, _event: &MyEvent) -> Result<(), ApplyError> {
        Ok(())
    }
}

impl CommandExecutor<ValidCommand> for MyAggregate {
    fn execute(_cmd: ValidCommand, _state: &Self) -> crate::command::ExecutionResult<MyEvent> {
        ExecutionResult::Ok(vec![MyEvent {}])
    }
}

impl CommandExecutor<InvalidCommand> for MyAggregate {
    fn execute(_cmd: InvalidCommand, _state: &Self) -> crate::command::ExecutionResult<MyEvent> {
        ExecutionResult::Err(CommandExecutorError::Any)
    }
}

#[derive(Default)]
pub(crate) struct DefaultAPP {}

impl Application for DefaultAPP {
    type Storage = InMemoryBackend;
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct MyEvent {}

impl Event for MyEvent {}
impl event_store::Event for MyEvent {
    fn event_type(&self) -> &'static str {
        "MyEvent"
    }

    fn all_event_types() -> Vec<&'static str> {
        vec!["MyEvent"]
    }
}

impl TryFrom<RecordedEvent> for MyEvent {
    type Error = ();

    fn try_from(_value: RecordedEvent) -> Result<Self, Self::Error> {
        todo!()
    }
}

pub(crate) struct ValidCommand(pub(crate) Uuid);

impl Command for ValidCommand {
    type Event = MyEvent;

    type Executor = MyAggregate;

    type ExecutorRegistry = AggregateInstanceRegistry<MyAggregate>;

    type CommandHandler = NoHandler;

    fn identifier(&self) -> String {
        self.0.to_string()
    }
}

pub(crate) struct InvalidCommand(pub(crate) Uuid);

impl Command for InvalidCommand {
    type Event = MyEvent;

    type Executor = MyAggregate;

    type ExecutorRegistry = AggregateInstanceRegistry<MyAggregate>;

    type CommandHandler = NoHandler;

    fn identifier(&self) -> String {
        self.0.to_string()
    }
}
