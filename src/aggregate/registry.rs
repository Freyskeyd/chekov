use crate::Application;
use crate::{aggregate::AggregateInstance, Aggregate, Command, CommandExecutorError, Dispatch};
use actix::prelude::{Actor, Addr};
use actix::{ActorFutureExt, ActorTryFutureExt, WrapFuture};
use actix_interop::{critical_section, with_ctx, FutureInterop};
use event_store::prelude::{EventStoreError, ReadVersion, RecordedEvent};
use futures::Future;
use tracing::trace;
use tracing::Instrument;
use uuid::Uuid;

#[doc(hidden)]
#[derive(Default)]
pub struct AggregateInstanceRegistry<A: Aggregate> {
    registry: ::std::collections::HashMap<String, ::actix::Addr<AggregateInstance<A>>>,
}

impl<A: Aggregate> AggregateInstanceRegistry<A> {
    async fn fetch_existing_state<APP: Application>(
        stream_id: String,
        correlation_id: Uuid,
    ) -> Result<Vec<RecordedEvent>, EventStoreError> {
        crate::event_store::EventStore::<APP>::with_reader(
            event_store::prelude::Reader::with_correlation_id(correlation_id)
                .stream(stream_id)
                .unwrap()
                .from(ReadVersion::Origin)
                .limit(10),
        )
        .await?
    }

    async fn create_new_instance<
        C: Command,
        E: Future<Output = Result<Vec<RecordedEvent>, EventStoreError>>,
    >(
        events: E,
        identity: String,
    ) -> Result<Addr<AggregateInstance<C::Executor>>, CommandExecutorError> {
        let events = match events.await {
            Ok(events) => events,
            _ => return Err(CommandExecutorError::Any),
        };

        let addr = AggregateInstance::create(move |ctx_agg| {
            trace!("Creating aggregate instance");
            let inner = C::Executor::default();

            inner.on_start(&identity, ctx_agg);
            let current_version = 0;

            AggregateInstance {
                inner,
                current_version,
            }
        });

        for event in events {
            trace!("Applying {:?}", event.event_type);
            match C::Executor::get_event_resolver(&event.event_type) {
                Some(resolver) => {
                    if let Err(_) = (resolver)(event, addr.clone()).await {
                        return Err(CommandExecutorError::Any);
                    }
                }
                None => {
                    println!("No resolver");
                }
            }
        }

        Ok(addr)
    }
}

impl<A: Aggregate> ::actix::registry::ArbiterService for AggregateInstanceRegistry<A> {}
impl<A: Aggregate> ::actix::Supervised for AggregateInstanceRegistry<A> {}
impl<A: Aggregate> ::actix::Actor for AggregateInstanceRegistry<A> {
    type Context = ::actix::Context<Self>;
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

            let stream_id = cmd.command.identifier();
            let states =
                Self::fetch_existing_state::<A>(stream_id.clone(), cmd.metadatas.correlation_id);
            let create_new_instance = Self::create_new_instance::<C, _>(states, stream_id.clone());

            Box::pin(
                async move { create_new_instance.await }
                    .instrument(tracing::Span::current())
                    .into_actor(self)
                    .then(move |result, actor, ctx| {
                        let addr = match result {
                            Ok(addr) => {
                                actor.registry.insert(stream_id.clone(), addr.clone());
                                addr
                            }
                            Err(_) => todo!(),
                        };

                        AggregateInstance::execute_command(addr, cmd)
                            .instrument(tracing::Span::current())
                            .into_actor(actor)
                    })
                    .map(|result, _, _| result),
            )
    }
}
