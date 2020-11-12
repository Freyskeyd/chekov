use super::{CommandExecutor, CommandExecutorError};
use crate::command::Command;
use crate::message::Dispatch;
use log::trace;

pub trait Aggregate: Default + std::marker::Unpin + 'static {
    fn identity() -> &'static str;
}

#[derive(Default)]
pub struct AggregateInstance<A: Aggregate> {
    pub(crate) inner: A,
}

impl<A: Aggregate> ::actix::Actor for AggregateInstance<A> {
    type Context = ::actix::Context<Self>;
}

impl<C: Command, S: event_store::prelude::Storage> ::actix::Handler<Dispatch<C, S>>
    for AggregateInstance<C::Executor>
{
    type Result = Result<Vec<C::Event>, CommandExecutorError>;
    fn handle(&mut self, cmd: Dispatch<C, S>, _ctx: &mut Self::Context) -> Self::Result {
        trace!(
            "Executing command {:?} from {} {:?}",
            std::any::type_name::<C>(),
            std::any::type_name::<Self>(),
            cmd.command
        );
        C::Executor::execute(cmd.command, &self.inner)
    }
}
