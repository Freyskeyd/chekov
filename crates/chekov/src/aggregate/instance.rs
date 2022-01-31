use self::internal::CommandExecutionResult;
use super::resolver::EventResolverRegistry;
use crate::command::{Command, Handler, NoHandler};
use crate::event::handler::Subscribe;
use crate::event::Event;
use crate::message::{DispatchWithState, ResolveAndApplyMany};
use crate::prelude::ApplyError;
use crate::{command::CommandExecutor, error::CommandExecutorError};
use crate::{message::Dispatch, prelude::EventApplier};
use crate::{Aggregate, Application};
use actix::prelude::*;
use actix::Addr;
use event_store::prelude::{EventStoreError, ReadVersion, RecordedEvent, SubscriptionNotification};
use tracing::trace;
use uuid::Uuid;

// TODO rename this module to match the behaviours
mod internal;
mod runtime;

/// Deals with the lifetime of a particular aggregate
pub struct AggregateInstance<A: Aggregate> {
    pub(crate) inner: A,
    pub(crate) current_version: i64,
    pub(crate) resolver: &'static EventResolverRegistry<A>,
}

impl<A: Aggregate> Default for AggregateInstance<A> {
    fn default() -> Self {
        Self {
            inner: A::default(),
            current_version: 0,
            resolver: A::get_event_resolver(),
        }
    }
}

impl<A: Aggregate> AggregateInstance<A> {
    pub(crate) async fn new<APP: Application>(
        identity: String,
        correlation_id: Uuid,
    ) -> Result<Addr<Self>, CommandExecutorError> {
        // Populate aggregate state
        let events = Self::fetch_existing_state::<APP>(identity.to_owned(), correlation_id).await;

        trace!("AggregateInstance received {:?}", events);
        let mut instance = AggregateInstance::<A>::default();

        if let Ok(events) = events {
            for event in events {
                trace!("Applying {} event ({})", event.event_uuid, event.event_type);
                if instance.apply_recorded_event(event).is_err() {
                    return Err(CommandExecutorError::Any);
                }

                instance.current_version += 1;
            }
        }

        trace!("AggregateInstance applied past events");

        // subscribe to events
        let addr = AggregateInstance::create(move |ctx| {
            instance.inner.on_start::<APP>(&identity, ctx);
            let broker = crate::subscriber::SubscriberManager::<APP>::from_registry();
            let recipient = ctx.address().recipient::<ResolveAndApplyMany>();
            let recipient_sub = ctx.address().recipient::<SubscriptionNotification>();
            broker.do_send(Subscribe(identity, recipient, recipient_sub));

            instance
        });

        Ok(addr)
    }

    pub(crate) fn create_mutable_state(&self) -> A {
        self.inner.clone()
    }

    pub(crate) async fn fetch_existing_state<APP: Application>(
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

    pub(crate) fn directly_apply<T>(state: &mut A, event: &T) -> Result<(), ApplyError>
    where
        T: Event,
        A: EventApplier<T>,
    {
        state.apply(event)
    }

    fn apply_recorded_event(&mut self, event: RecordedEvent) -> Result<(), ApplyError> {
        if let Some(resolver) = self.resolver.get_applier(&event.event_type) {
            return (resolver)(&mut self.inner, event);
        }

        Ok(())
    }

    pub(crate) async fn execute_command<APP: Application, C: Command<Executor = A>>(
        addr: Addr<Self>,
        cmd: Dispatch<C, APP>,
    ) -> Result<Vec<C::Event>, CommandExecutorError>
    where
        A: CommandExecutor<C>,
        C::CommandHandler: Handler<C, A>,
    {
        addr.send(cmd).await?
    }

    pub(crate) async fn execute<C: Command, APP: Application>(
        state: A,
        command: Dispatch<C, APP>,
    ) -> Result<(Vec<C::Event>, A), CommandExecutorError>
    where
        A: CommandExecutor<C>,
        A: EventApplier<C::Event>,
        C::CommandHandler: Handler<C, A>,
    {
        if std::any::TypeId::of::<C::CommandHandler>() == std::any::TypeId::of::<NoHandler>() {
            A::execute(command.command, &state).map(|events| (events, state))
        } else {
            crate::command::CommandHandlerInstance::<C::CommandHandler>::from_registry()
                .send(DispatchWithState::from_dispatch(command, state))
                .await?
        }
    }

    pub(crate) fn apply_many<E: Event>(state: &mut A, events: &[E]) -> Result<(), ApplyError>
    where
        A: EventApplier<E>,
    {
        for event in events.iter() {
            if Self::directly_apply(state, event).is_err() {
                return Err(ApplyError::Any);
            }
        }

        Ok(())
    }

    async fn persist_events<E: Event + event_store::Event, APP: Application>(
        events: Vec<E>,
        mut state: A,
        correlation_id: Uuid,
        stream_id: String,
        current_version: i64,
    ) -> Result<CommandExecutionResult<E, A>, CommandExecutorError>
    where
        A: EventApplier<E>,
    {
        Self::apply_many(&mut state, &events)?;
        let ev: Vec<&_> = events.iter().collect();
        match crate::event_store::EventStore::<APP>::with_appender(
            event_store::prelude::Appender::with_correlation_id(correlation_id)
                .events(&ev[..])?
                .to(&stream_id)?
                .expected_version(event_store::prelude::ExpectedVersion::Version(
                    current_version,
                )),
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
    }

    async fn execute_and_apply<C: Command, APP: Application>(
        state: A,
        command: Dispatch<C, APP>,
        current_version: i64,
    ) -> Result<CommandExecutionResult<C::Event, A>, CommandExecutorError>
    where
        A: CommandExecutor<C>,
        A: EventApplier<C::Event>,
        C::CommandHandler: Handler<C, A>,
    {
        let correlation_id = command.metadatas.correlation_id;
        let stream_id = command.command.identifier();

        match Self::execute(state, command).await {
            Ok((events, state)) => {
                Self::persist_events::<_, APP>(
                    events,
                    state,
                    correlation_id,
                    stream_id,
                    current_version,
                )
                .await
            }
            Err(_) => Err(CommandExecutorError::Any),
        }
    }
}
