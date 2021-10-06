use crate::command::{Command, Handler, NoHandler};
use crate::event::Event;
use crate::message::{AggregateVersion, DispatchWithState, ResolveAndApply, ResolveAndApplyMany};
use crate::prelude::ApplyError;
use crate::{command::CommandExecutor, error::CommandExecutorError};
use crate::{message::Dispatch, prelude::EventApplier};
use crate::{Aggregate, Application};
use actix::prelude::*;
use actix::{Addr, Handler as ActixHandler};
use event_store::prelude::{EventStoreError, ReadVersion, RecordedEvent};
use tracing::trace;
use uuid::Uuid;

use super::resolver::EventResolverRegistry;

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
        let events = Self::fetch_existing_state::<APP>(identity.to_owned(), correlation_id).await;

        trace!("Creating aggregate instance");

        let mut instance = AggregateInstance::<A>::default();

        if let Ok(events) = events {
            for event in events {
                if instance.apply_recorded_event(event).is_err() {
                    return Err(CommandExecutorError::Any);
                }

                instance.current_version += 1;
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

    fn directly_apply<T>(state: &mut A, event: &T) -> Result<(), ApplyError>
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

    async fn execute<C: Command, APP: Application>(
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
            e => {
                trace!("{:?}", e);
                Err(CommandExecutorError::Any)
            }
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

struct CommandExecutionResult<E, A> {
    events: Vec<E>,
    new_version: i64,
    state: A,
}

impl<A: Aggregate> ActixHandler<AggregateVersion> for AggregateInstance<A> {
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

impl<C: Command, A: Application> ActixHandler<Dispatch<C, A>> for AggregateInstance<C::Executor>
where
    C::CommandHandler: Handler<C, C::Executor>,
{
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

impl<A: Aggregate> ActixHandler<ResolveAndApply> for AggregateInstance<A> {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: ResolveAndApply, _: &mut Self::Context) -> Self::Result {
        self.apply_recorded_event(msg.0).map_err(|_| ())
    }
}

impl<A: Aggregate> ActixHandler<ResolveAndApplyMany> for AggregateInstance<A> {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: ResolveAndApplyMany, _: &mut Self::Context) -> Self::Result {
        for event in msg.0 {
            let _ = self.apply_recorded_event(event);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use event_store::{prelude::Appender, InMemoryBackend};

    use crate::{
        command::CommandMetadatas,
        event_store::EventStore,
        tests::aggregates::support::{
            ExampleAggregate, InvalidCommand, InvalidEvent, MyApplication, MyEvent, ValidCommand,
        },
    };

    use super::*;
    #[test]
    fn state_can_be_duplicated() {
        let instance = AggregateInstance {
            inner: ExampleAggregate::default(),
            current_version: 0,
            resolver: ExampleAggregate::get_event_resolver(),
        };

        let _: ExampleAggregate = instance.create_mutable_state();
    }

    #[actix::test]
    async fn can_execute_command() {
        let instance = AggregateInstance {
            inner: ExampleAggregate::default(),
            current_version: 0,
            resolver: ExampleAggregate::get_event_resolver(),
        };

        assert_eq!(
            Ok(vec![MyEvent {}]),
            AggregateInstance::execute(
                instance.create_mutable_state(),
                Dispatch::<_, MyApplication> {
                    storage: PhantomData,
                    command: ValidCommand(Uuid::new_v4()),
                    metadatas: CommandMetadatas::default(),
                },
            )
            .await
            .map(|(v, _)| v)
        );
    }

    #[actix::test]
    async fn can_recover_from_fail_execution() {
        let instance = AggregateInstance {
            inner: ExampleAggregate::default(),
            current_version: 1,
            resolver: ExampleAggregate::get_event_resolver(),
        };

        let result = AggregateInstance::execute(
            instance.create_mutable_state(),
            Dispatch::<_, MyApplication> {
                storage: PhantomData,
                command: InvalidCommand(Uuid::new_v4()),
                metadatas: CommandMetadatas::default(),
            },
        )
        .await;

        assert!(matches!(result, Err(_)));

        let result = AggregateInstance::execute(
            instance.create_mutable_state(),
            Dispatch::<_, MyApplication> {
                storage: PhantomData,
                command: ValidCommand(Uuid::new_v4()),
                metadatas: CommandMetadatas::default(),
            },
        )
        .await;

        assert!(matches!(result, Ok(_)));
    }

    #[test]
    fn can_apply_event() {
        let instance = AggregateInstance {
            inner: ExampleAggregate::default(),
            current_version: 1,
            resolver: ExampleAggregate::get_event_resolver(),
        };

        let result =
            AggregateInstance::directly_apply(&mut instance.create_mutable_state(), &MyEvent {});

        assert!(matches!(result, Ok(_)));
    }

    #[actix::test]
    async fn can_fetch_existing_state() {
        let storage = InMemoryBackend::default();
        let event_store = crate::event_store::EventStore::<MyApplication> {
            addr: event_store::EventStore::builder()
                .storage(storage)
                .build()
                .await
                .unwrap()
                .start(),
        }
        .start();

        ::actix::SystemRegistry::set(event_store);

        let identifier = Uuid::new_v4();
        let _ = EventStore::<MyApplication>::with_appender(
            Appender::default()
                .event(&MyEvent {})
                .unwrap()
                .to(&identifier)
                .unwrap(),
        )
        .await;

        let result = AggregateInstance::<ExampleAggregate>::fetch_existing_state::<MyApplication>(
            identifier.to_string(),
            Uuid::new_v4(),
        )
        .await;

        assert_eq!(result.expect("shouldn't fail").len(), 1);
    }

    #[actix::test]
    async fn recover_from_failing_events() {
        let storage = InMemoryBackend::default();
        let event_store = crate::event_store::EventStore::<MyApplication> {
            addr: event_store::EventStore::builder()
                .storage(storage)
                .build()
                .await
                .unwrap()
                .start(),
        }
        .start();

        ::actix::SystemRegistry::set(event_store);

        let identifier = Uuid::new_v4();
        let _ = EventStore::<MyApplication>::with_appender(
            Appender::default()
                .event(&InvalidEvent {})
                .unwrap()
                .to(&identifier)
                .unwrap(),
        )
        .await;

        let result = AggregateInstance::<ExampleAggregate>::fetch_existing_state::<MyApplication>(
            identifier.to_string(),
            Uuid::new_v4(),
        )
        .await;

        assert_eq!(result.expect("shouldn't fail").len(), 1);

        assert!(AggregateInstance::<ExampleAggregate>::new::<MyApplication>(
            identifier.to_string(),
            Uuid::new_v4(),
        )
        .await
        .is_err());
    }
}
