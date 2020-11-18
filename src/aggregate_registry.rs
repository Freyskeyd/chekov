use crate::Application;
use crate::{
    aggregate::AggregateInstance, event_applier::EventApplier, Aggregate, Command,
    CommandExecutorError, Dispatch,
};
use actix::prelude::{Actor, Addr, AsyncContext};
use event_store::prelude::ReadVersion;
use log::trace;
use std::convert::TryFrom;

#[derive(Default)]
pub struct AggregateInstanceRegistry<A: Aggregate> {
    registry: ::std::collections::HashMap<String, ::actix::Addr<AggregateInstance<A>>>,
}

impl<A: Aggregate> ::actix::registry::ArbiterService for AggregateInstanceRegistry<A> {}
impl<A: Aggregate> ::actix::Supervised for AggregateInstanceRegistry<A> {}
impl<A: Aggregate> ::actix::Actor for AggregateInstanceRegistry<A> {
    type Context = ::actix::Context<Self>;
}
use actix_interop::{critical_section, with_ctx, FutureInterop};

impl<C: Command, A: Application> ::actix::Handler<Dispatch<C, A>>
    for AggregateInstanceRegistry<C::Executor>
{
    type Result = actix::ResponseActFuture<Self, Result<Vec<C::Event>, CommandExecutorError>>;

    fn handle(&mut self, cmd: Dispatch<C, A>, _ctx: &mut Self::Context) -> Self::Result {
        // Open aggregate
        async move {
            critical_section::<Self, _>(async move {
                let id = cmd.command.identifier();
                let addr: Addr<_> = if let Some(addr) =
                    with_ctx(|actor: &mut Self, _| actor.registry.get(&id).cloned())
                {
                    trace!(
                        "{}({}) already started",
                        std::any::type_name::<C::Executor>(),
                        id
                    );
                    addr
                } else {
                    // start it?
                    trace!("{}({}) not found", std::any::type_name::<C::Executor>(), id);
                    trace!(
                        "Rebuilding state for {}({}) ",
                        std::any::type_name::<C::Executor>(),
                        id
                    );
                    let result = match event_store::read()
                        .stream(&id)
                        .unwrap()
                        .from(ReadVersion::Origin)
                        .limit(10)
                        .execute_async::<A::Storage>()
                        .await
                    {
                        Ok(events) => events,
                        Err(_) => panic!(""),
                    };

                    let addr = AggregateInstance::create(move |ctx_agg| {
                        trace!("Creating aggregate instance");
                        let _ctx_address = ctx_agg.address();
                        let mut inner = C::Executor::default();
                        for event in result {
                            let _res = match C::Event::try_from(event.clone()) {
                                Ok(parsed_event) => inner.apply(&parsed_event).map_err(|_| ()),
                                _ => Err(()),
                            };
                        }
                        AggregateInstance { inner }
                    });

                    with_ctx(|actor: &mut Self, _| actor.registry.insert(id.clone(), addr.clone()));
                    addr
                };

                match addr.send(cmd).await {
                    Ok(res) => {
                        if let Ok(ref events) = res {
                            trace!("Generated {:?}", events.len());
                            let ev: Vec<&_> = events.iter().collect();
                            let _result = event_store::append()
                                .events(&ev[..])
                                .unwrap()
                                .to(&id)
                                .unwrap()
                                .expected_version(event_store::prelude::ExpectedVersion::AnyVersion)
                                .execute::<A::Storage>()
                                .await;
                        }
                        res
                    }
                    Err(_) => Err(CommandExecutorError::Any),
                }
            })
            .await
        }
        .interop_actor_boxed(self)
    }
}
