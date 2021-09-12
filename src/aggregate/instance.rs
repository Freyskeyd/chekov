use crate::command::Command;
use crate::event::Event;
use crate::message::{AggregateVersion, ResolveAndApply, ResolveAndApplyMany};
use crate::prelude::ApplyError;
use crate::{command::CommandExecutor, error::CommandExecutorError};
use crate::{message::Dispatch, prelude::EventApplier};
use crate::{Aggregate, Application};
use actix::prelude::*;
use actix::{Addr, Handler};
use event_store::prelude::{EventStoreError, ReadVersion, RecordedEvent};
use tracing::trace;
use uuid::Uuid;

use super::resolver::EventResolverRegistry;

/// Deals with the lifetime of a particular aggregate
pub struct AggregateInstance<A: Aggregate> {
    pub(crate) inner: A,
    #[allow(dead_code)]
    pub(crate) current_version: i64,
    pub(crate) resolver: &'static EventResolverRegistry<A>,
}

impl<A: Aggregate> AggregateInstance<A> {
    pub(crate) async fn new<APP: Application>(
        identity: String,
        correlation_id: Uuid,
    ) -> Result<Addr<Self>, CommandExecutorError> {
        let events = Self::fetch_existing_state::<APP>(identity.to_owned(), correlation_id).await;

        trace!("Creating aggregate instance");

        let inner = A::default();
        let current_version = 0;

        let mut instance = AggregateInstance {
            inner,
            current_version,
            resolver: A::get_event_resolver(),
        };

        for event in events.unwrap() {
            if instance.apply_recorded_event(event).is_err() {
                return Err(CommandExecutorError::Any);
            }
        }

        let addr = AggregateInstance::create(move |ctx| {
            instance.inner.on_start::<APP>(&identity, ctx);

            instance
        });

        Ok(addr)
    }

    fn create_mutable_state(&self) -> A {
        self.inner.clone()
    }

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

    fn directly_apply<T>(state: &mut A, event: &T)
    where
        T: Event,
        A: EventApplier<T>,
    {
        let _ = state.apply(event);
    }

    fn apply_recorded_event(&mut self, event: RecordedEvent) -> Result<(), ApplyError> {
        if let Some(resolver) = self.resolver.get_applier(&event.event_type) {
            let _ = (resolver)(&mut self.inner, event);
        }

        Ok(())
    }

    pub(crate) async fn execute_command<APP: Application, C: Command<Executor = A>>(
        addr: Addr<Self>,
        cmd: Dispatch<C, APP>,
    ) -> Result<Vec<C::Event>, CommandExecutorError> {
        addr.send(cmd).await?
    }

    async fn execute_and_apply<C: Command, APP: Application>(
        mut state: A,
        command: Dispatch<C, APP>,
        current_version: i64,
    ) -> Result<CommandExecutionResult<C::Event, A>, CommandExecutorError>
    where
        A: CommandExecutor<C>,
        A: EventApplier<C::Event>,
    {
        let correlation_id = command.metadatas.correlation_id;
        let stream_id = command.command.identifier();

        if let Ok(events) = A::execute(command.command, &state) {
            // apply events
            events
                .iter()
                .for_each(|event| Self::directly_apply(&mut state, event));
            let ev: Vec<&_> = events.iter().collect();
            match crate::event_store::EventStore::<APP>::with_appender(
                event_store::prelude::Appender::with_correlation_id(correlation_id)
                    .events(&ev[..])?
                    .to(&stream_id)?
                    .expected_version(event_store::prelude::ExpectedVersion::AnyVersion),
            )
            // TODO deal with mailbox error
            .await
            {
                Ok(Ok(_)) => {
                    let new_version = current_version + events.len() as i64;
                    Ok(CommandExecutionResult {
                        events,
                        new_version,
                        state,
                    })
                }
                _ => Err(CommandExecutorError::Any),
            }
        } else {
            Err(CommandExecutorError::Any)
        }
    }
}

struct CommandExecutionResult<E, A> {
    events: Vec<E>,
    new_version: i64,
    state: A,
}

impl<A: Aggregate> Handler<AggregateVersion> for AggregateInstance<A> {
    type Result = i64;

    fn handle(&mut self, _: AggregateVersion, _: &mut Self::Context) -> Self::Result {
        self.current_version
    }
}

impl<A: Aggregate> Actor for AggregateInstance<A> {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        trace!("Aggregate {:?} started", std::any::type_name::<Self>());
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        trace!("Aggregate {:?} stopped", std::any::type_name::<Self>());
    }
}

impl<C: Command, A: Application> Handler<Dispatch<C, A>> for AggregateInstance<C::Executor> {
    type Result = ResponseActFuture<Self, Result<Vec<C::Event>, CommandExecutorError>>;

    #[tracing::instrument(
        name = "AggregateInstance",
        skip(self, _ctx, cmd),
        fields(correlation_id = %cmd.metadatas.correlation_id, aggregate_id = %cmd.command.identifier(), aggregate_type = %::std::any::type_name::<C::Executor>())
    )]
    fn handle(&mut self, cmd: Dispatch<C, A>, _ctx: &mut Self::Context) -> Self::Result {
        trace!("Executing command {}", std::any::type_name::<C>(),);

        let mutable_state = self.create_mutable_state();
        let current_version = self.current_version;

        Box::pin(
            async move { Self::execute_and_apply(mutable_state, cmd, current_version).await }
                .into_actor(self)
                .map(|result, actor, _| match result {
                    Ok(CommandExecutionResult {
                        events,
                        new_version,
                        state,
                    }) => {
                        actor.current_version = new_version;
                        actor.inner = state;
                        Ok(events)
                    }
                    Err(e) => Err(e),
                }),
        )
    }
}

impl<A: Aggregate> Handler<ResolveAndApply> for AggregateInstance<A> {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: ResolveAndApply, _: &mut Self::Context) -> Self::Result {
        self.apply_recorded_event(msg.0).map_err(|_| ())
    }
}

impl<A: Aggregate> Handler<ResolveAndApplyMany> for AggregateInstance<A> {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: ResolveAndApplyMany, _: &mut Self::Context) -> Self::Result {
        for event in msg.0 {
            let _ = self.apply_recorded_event(event);
        }

        Ok(())
    }
}
