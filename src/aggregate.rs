use super::{CommandExecutor, CommandExecutorError};
use crate::command::Command;
use crate::message::Dispatch;
use crate::Application;
use tracing::trace;

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

impl<C: Command, A: Application> ::actix::Handler<Dispatch<C, A>>
    for AggregateInstance<C::Executor>
{
    type Result = Result<Vec<C::Event>, CommandExecutorError>;

    #[tracing::instrument(
        name = "AggregateInstance",
        skip(self, _ctx, cmd),
        fields(correlation_id = %cmd.metadatas.correlation_id, aggregate_id = %cmd.command.identifier(), aggregate_type = %::std::any::type_name::<C::Executor>())
    )]
    fn handle(&mut self, cmd: Dispatch<C, A>, _ctx: &mut Self::Context) -> Self::Result {
        trace!("Executing command {}", std::any::type_name::<C>(),);
        C::Executor::execute(cmd.command, &self.inner)
    }
}
