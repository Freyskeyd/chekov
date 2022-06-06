use crate::command::{CommandExecutor, CommandMetadatas};
use crate::message::{GetAggregateAddr, ShutdownAggregate};
use crate::Application;
use crate::{aggregate::AggregateInstance, Aggregate, Command, CommandExecutorError, Dispatch};
use actix::registry::SystemService;
use actix::{ActorFutureExt, ActorTryFutureExt, Addr, Handler, WrapFuture};
use tracing::Instrument;
use tracing::{debug, trace};
use uuid::Uuid;

#[doc(hidden)]
#[derive(Default)]
pub struct AggregateInstanceRegistry<A: Aggregate> {
    registry: ::std::collections::HashMap<String, ::actix::Addr<AggregateInstance<A>>>,
}

impl<A: Aggregate> SystemService for AggregateInstanceRegistry<A> {}
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

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        trace!(
            "AggregateInstanceRegistry {:?} stopping",
            std::any::type_name::<Self>()
        );

        actix::Running::Stop
    }
}

impl<A: Aggregate> AggregateInstanceRegistry<A> {
    pub async fn execute<APP: Application, C: Command>(
        command: C,
    ) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        A: CommandExecutor<C>,
        C: Command<Executor = A>,
    {
        Self::from_registry()
            .send(Dispatch::<_, APP> {
                metadatas: CommandMetadatas::default(),
                storage: std::marker::PhantomData,
                command,
            })
            .await?
    }

    pub async fn shutdown_aggregate<APP: Application>(identifier: String) -> Result<(), ()> {
        trace!(
            "AggregateInstanceRegistry {:?} is asked to shutdown {}",
            std::any::type_name::<Self>(),
            identifier
        );
        Self::from_registry()
            .send(ShutdownAggregate { identifier })
            .await
            .map_err(|_| ())?
    }
}

impl<A: Aggregate> Handler<ShutdownAggregate> for AggregateInstanceRegistry<A> {
    type Result = actix::ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, msg: ShutdownAggregate, _ctx: &mut Self::Context) -> Self::Result {
        // TODO: No need to alloc ?
        let identifier = msg.identifier.clone();
        if let Some(addr) = self.registry.get(&identifier).cloned() {
            Box::pin(async move { addr.send(msg).await }.into_actor(self).map(
                move |_, actor, _| {
                    actor.registry.remove(&identifier);

                    Ok(())
                },
            ))
        } else {
            Box::pin(async { Ok(()) }.into_actor(self))
        }
    }
}

impl<A: Aggregate> Handler<GetAggregateAddr<A>> for AggregateInstanceRegistry<A> {
    type Result = Option<Addr<AggregateInstance<A>>>;

    fn handle(&mut self, msg: GetAggregateAddr<A>, _ctx: &mut Self::Context) -> Self::Result {
        self.registry.get(&msg.identifier).cloned()
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
                fut.instrument(tracing::Span::current())
                    .into_actor(self)
                    .map(move |result, actor, _| match result {
                        Ok(addr) => {
                            actor.registry.insert(stream_id, addr.clone());
                            Ok(addr)
                        }
                        Err(e) => {
                            debug!("Error: {:?}", e);
                            Err(e)
                        }
                    })
                    .and_then(move |addr, actor, _ctx| {
                        AggregateInstance::execute_command(addr, cmd)
                            .instrument(tracing::Span::current())
                            .into_actor(actor)
                    }),
            )
        }
    }
}
