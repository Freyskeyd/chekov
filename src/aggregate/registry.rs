use crate::Application;
use crate::{aggregate::AggregateInstance, Aggregate, Command, CommandExecutorError, Dispatch};
use actix::{ActorFutureExt, WrapFuture};
use tracing::trace;
use tracing::Instrument;

#[doc(hidden)]
#[derive(Default)]
pub struct AggregateInstanceRegistry<A: Aggregate> {
    registry: ::std::collections::HashMap<String, ::actix::Addr<AggregateInstance<A>>>,
}

impl<A: Aggregate> ::actix::registry::SystemService for AggregateInstanceRegistry<A> {}
impl<A: Aggregate> ::actix::Supervised for AggregateInstanceRegistry<A> {}
impl<A: Aggregate> ::actix::Actor for AggregateInstanceRegistry<A> {
    type Context = ::actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        trace!(
            "AggregateInstanceRegistry {:?} started",
            std::any::type_name::<Self>()
        );
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        trace!(
            "AggregateInstanceRegistry {:?} stopped",
            std::any::type_name::<Self>()
        );
    }
}

impl<C: Command, A: Application> ::actix::Handler<Dispatch<C, A>>
    for AggregateInstanceRegistry<C::Executor>
{
    type Result = actix::ResponseActFuture<Self, Result<Vec<C::Event>, CommandExecutorError>>;

    #[tracing::instrument(
        name = "AggregateRegistry",
        skip(self, _ctx, cmd),
        fields(correlation_id = %cmd.metadatas.correlation_id, aggregate_id = %cmd.command.identifier(), aggregate_type = %::std::any::type_name::<C::Executor>())
    )]
    fn handle(&mut self, cmd: Dispatch<C, A>, _ctx: &mut Self::Context) -> Self::Result {
        println!("Registry: {:?}", self.registry);
        // Open aggregate
        if let Some(addr) = self.registry.get(&cmd.command.identifier()).cloned() {
            trace!("Instance already started");
            Box::pin(
                AggregateInstance::execute_command(addr, cmd)
                    .instrument(tracing::Span::current())
                    .into_actor(self),
            )
        } else {
            // start it?
            trace!("Instance not found");
            trace!("Rebuilding instance state");

            let fut = AggregateInstance::<C::Executor>::new::<A>(
                cmd.command.identifier(),
                cmd.metadatas.correlation_id,
            );

            let stream_id = cmd.command.identifier();

            Box::pin(
                async move { fut.await }
                    .instrument(tracing::Span::current())
                    .into_actor(self)
                    .map(move |result, actor, _| {
                        let addr = match result {
                            Ok(addr) => {
                                actor.registry.insert(stream_id, addr.clone());
                                println!("Registry: {:?}", actor.registry);
                                addr
                            }
                            Err(_) => todo!(),
                        };

                        addr
                    })
                    .then(move |addr, actor, _ctx| {
                        AggregateInstance::execute_command(addr, cmd)
                            .instrument(tracing::Span::current())
                            .into_actor(actor)
                    }),
            )
        }
    }
}
