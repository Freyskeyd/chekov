use crate::Application;
use crate::{aggregate::AggregateInstance, Aggregate, Command, CommandExecutorError, Dispatch};
use actix::prelude::{Actor, Addr};
use actix_interop::{critical_section, with_ctx, FutureInterop};
use event_store::prelude::ReadVersion;
use tracing::trace;
use tracing::Instrument;

#[doc(hidden)]
#[derive(Default)]
pub struct AggregateInstanceRegistry<A: Aggregate> {
    registry: ::std::collections::HashMap<String, ::actix::Addr<AggregateInstance<A>>>,
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
        async move {
            critical_section::<Self, _>(
                async move {
                    let correlation_id = cmd.metadatas.correlation_id;
                    let id = cmd.command.identifier();
                    let addr_r: Result<Addr<_>, ()> = if let Some(addr) =
                        with_ctx(|actor: &mut Self, _| actor.registry.get(&id).cloned())
                    {
                        trace!("Instance already started");
                        Ok(addr)
                    } else {
                        // start it?
                        trace!("Instance not found");
                        trace!("Rebuilding instance state");

                        let events = match crate::event_store::EventStore::<A>::with_reader(
                            event_store::prelude::Reader::with_correlation_id(correlation_id)
                                .stream(&id)
                                .unwrap()
                                .from(ReadVersion::Origin)
                                .limit(10),
                        )
                        // TODO deal with mailbox error
                        .await
                        .unwrap()
                        {
                            Ok(events) => events,
                            Err(_) => panic!(""),
                        };

                        let identity = id.clone();
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

                        with_ctx(|actor: &mut Self, _| {
                            actor.registry.insert(id.clone(), addr.clone())
                        });
                        Ok(addr)
                    };

                    if let Ok(addr) = addr_r {
                        match addr.send(cmd).await {
                            Ok(res) => {
                                if let Ok(ref events) = res {
                                    trace!("Generated {:?}", events.len());
                                    let ev: Vec<&_> = events.iter().collect();
                                    match crate::event_store::EventStore::<A>::with_appender(
                                        event_store::prelude::Appender::with_correlation_id(
                                            correlation_id,
                                        )
                                        .events(&ev[..])
                                        .unwrap()
                                        .to(&id)
                                        .unwrap()
                                        .expected_version(
                                            event_store::prelude::ExpectedVersion::AnyVersion,
                                        ),
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
                    } else {
                        Err(CommandExecutorError::Any)
                    }
                }
                .instrument(tracing::Span::current()),
            )
            .await
        }
        .instrument(tracing::Span::current())
        .interop_actor_boxed(self)
    }
}
