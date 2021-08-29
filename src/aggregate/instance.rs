use crate::event::Event;
use crate::{command::Command, message::EventEnvelope};
use crate::{command::CommandExecutor, error::CommandExecutorError};
use crate::{message::Dispatch, prelude::EventApplier};
use crate::{Aggregate, Application};
use tracing::trace;

/// Deals with the lifetime of a particular aggregate
#[derive(Default)]
pub struct AggregateInstance<A: Aggregate> {
    pub(crate) inner: A,
    #[allow(dead_code)]
    pub(crate) current_version: i64,
}

impl<A: Aggregate> AggregateInstance<A> {
    fn apply<T>(&mut self, event: &T)
    where
        T: Event,
        A: EventApplier<T>,
    {
        let _ = self.inner.apply(event);
    }
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

impl<A: Aggregate, T: Event> ::actix::Handler<EventEnvelope<T>> for AggregateInstance<A>
where
    A: EventApplier<T>,
{
    type Result = ();

    fn handle(&mut self, msg: EventEnvelope<T>, _: &mut Self::Context) -> Self::Result {
        self.apply(&msg.event);
    }
}
