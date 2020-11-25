use super::Application;
use super::Command;
use super::CommandExecutorError;
use actix::prelude::*;
use event_store::prelude::*;

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<Vec<C::Event>, CommandExecutorError>")]
pub struct Dispatch<C: Command, A: Application> {
    pub storage: std::marker::PhantomData<A>,
    // pub to: actix::Recipient<Dispatch<T, S>>,
    pub command: C,
}

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<usize, ()>")]
pub struct EventEnvelope(pub RecordedEvent);
