use super::Command;
use super::CommandExecutorError;
use actix::prelude::*;
use event_store::prelude::*;

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<Vec<T::Event>, CommandExecutorError>")]
pub struct Dispatch<T: Command, S: event_store::prelude::Storage> {
    pub storage: std::marker::PhantomData<S>,
    // pub to: actix::Recipient<Dispatch<T, S>>,
    pub command: T,
}

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<usize, ()>")]
pub struct EventEnvelope(pub RecordedEvent);
