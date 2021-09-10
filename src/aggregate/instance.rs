use crate::event::Event;
use crate::message::{ResolveAndApply, ResolveAndApplyMany};
use crate::prelude::ApplyError;
use crate::{command::Command, message::EventEnvelope};
use crate::{command::CommandExecutor, error::CommandExecutorError};
use crate::{message::Dispatch, prelude::EventApplier};
use crate::{Aggregate, Application};
use actix::prelude::*;
use actix::{Addr, Handler};
use event_store::prelude::{EventStoreError, ReadVersion, RecordedEvent};
use tracing::trace;
use uuid::Uuid;

/// Deals with the lifetime of a particular aggregate
#[derive(Debug, Default)]
pub struct AggregateInstance<A: Aggregate> {
    pub(crate) inner: A,
    #[allow(dead_code)]
    pub(crate) current_version: i64,
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

    fn apply<T>(&mut self, event: &T)
    where
        T: Event,
        A: EventApplier<T>,
    {
        let _ = self.inner.apply(event);
    }

    fn apply_recorded_event(&mut self, event: RecordedEvent) -> Result<(), ApplyError> {
        println!("APPLYING {:?}", event.event_type);
        self.inner.apply_recorded_event(event)
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

impl<A: Aggregate, T: Event> Handler<EventEnvelope<T>> for AggregateInstance<A>
where
    A: EventApplier<T>,
{
    type Result = ();

    fn handle(&mut self, msg: EventEnvelope<T>, _: &mut Self::Context) -> Self::Result {
        self.apply(&msg.event);
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