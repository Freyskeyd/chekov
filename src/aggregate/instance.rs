use crate::event::Event;
use crate::{command, Aggregate, Application};
use crate::{command::Command, message::EventEnvelope};
use crate::{command::CommandExecutor, error::CommandExecutorError};
use crate::{message::Dispatch, prelude::EventApplier};
use actix::{Addr, Handler};
use tracing::trace;
use uuid::Uuid;

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

    pub(crate) async fn execute_command<APP: Application, C: Command<Executor = A>>(
        addr: Addr<Self>,
        cmd: Dispatch<C, APP>,
    ) -> Result<Vec<C::Event>, CommandExecutorError> {
        let correlation_id = cmd.metadatas.correlation_id;
        let stream_id = cmd.command.identifier();
        match addr.send(cmd).await {
            Ok(res) => {
                if let Ok(ref events) = res {
                    trace!("Generated {:?}", events.len());
                    let ev: Vec<&_> = events.iter().collect();
                    match crate::event_store::EventStore::<APP>::with_appender(
                        event_store::prelude::Appender::with_correlation_id(correlation_id)
                            .events(&ev[..])
                            .unwrap()
                            .to(&stream_id)
                            .unwrap()
                            .expected_version(event_store::prelude::ExpectedVersion::AnyVersion),
                    )
                    // TODO deal with mailbox error
                    .await
                    {
                        Ok(Ok(_)) => res,
                        _ => Err(CommandExecutorError::Any),
                    }
                } else {
                    Err(CommandExecutorError::Any)
                }
            }
            Err(_) => Err(CommandExecutorError::Any),
        }
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
