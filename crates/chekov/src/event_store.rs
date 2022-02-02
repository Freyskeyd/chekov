use crate::application::Application;
use crate::message::{ExecuteAppender, ExecuteReader, ExecuteStreamInfo, GetAddr};
pub use ::event_store::prelude::Event;
pub use ::event_store::prelude::RecordedEvent;
use actix::{Addr, Context, Handler, MailboxError, ResponseFuture, SystemService};
use event_store::prelude::EventStoreError;
use event_store::prelude::Stream;
use futures::TryFutureExt;
use std::marker::PhantomData;
use uuid::Uuid;

pub use event_store::prelude::PostgresEventBus;
pub use event_store::storage::PostgresStorage;

pub(crate) struct EventStore<A: Application> {
    pub(crate) addr: Addr<event_store::EventStore<A::Storage>>,
}

impl<A> EventStore<A>
where
    A: Application,
{
    pub async fn with_appender(
        appender: event_store::prelude::Appender,
    ) -> Result<Result<Vec<Uuid>, EventStoreError>, MailboxError> {
        Self::from_registry().send(ExecuteAppender(appender)).await
    }
    pub async fn with_reader(
        reader: event_store::prelude::Reader,
    ) -> Result<Result<Vec<RecordedEvent>, EventStoreError>, MailboxError> {
        Self::from_registry().send(ExecuteReader(reader)).await
    }

    #[allow(dead_code)]
    pub async fn stream_info(
        stream_uuid: &str,
    ) -> Result<Result<event_store::prelude::Stream, EventStoreError>, MailboxError> {
        Self::from_registry()
            .send(ExecuteStreamInfo(stream_uuid.to_string()))
            .await
    }

    pub async fn get_addr() -> Result<Addr<event_store::EventStore<A::Storage>>, MailboxError> {
        Self::from_registry()
            .send(GetAddr {
                _phantom: PhantomData,
            })
            .await
    }
}

impl<A> Handler<GetAddr<A::Storage>> for EventStore<A>
where
    A: Application,
{
    type Result = Addr<event_store::EventStore<A::Storage>>;

    fn handle(&mut self, _: GetAddr<A::Storage>, _: &mut Self::Context) -> Self::Result {
        self.addr.clone()
    }
}

impl<A> Handler<ExecuteReader> for EventStore<A>
where
    A: Application,
{
    type Result = ResponseFuture<Result<Vec<RecordedEvent>, EventStoreError>>;

    fn handle(&mut self, reader: ExecuteReader, _: &mut Self::Context) -> Self::Result {
        let addr = self.addr.clone();
        Box::pin(reader.0.execute(addr))
    }
}

impl<A> Handler<ExecuteAppender> for EventStore<A>
where
    A: Application,
{
    type Result = ResponseFuture<Result<Vec<uuid::Uuid>, EventStoreError>>;

    fn handle(&mut self, appender: ExecuteAppender, _: &mut Self::Context) -> Self::Result {
        let addr = self.addr.clone();
        Box::pin(appender.0.execute(addr))
    }
}

impl<A> Handler<ExecuteStreamInfo> for EventStore<A>
where
    A: Application,
{
    type Result = ResponseFuture<Result<Stream, EventStoreError>>;

    fn handle(&mut self, appender: ExecuteStreamInfo, _: &mut Self::Context) -> Self::Result {
        let addr = self.addr.clone();

        Box::pin(
            addr.send(event_store::prelude::StreamInfo {
                correlation_id: uuid::Uuid::new_v4(),
                stream_uuid: appender.0,
            })
            .map_ok_or_else(|e| Err(e.into()), |r| r),
        )
    }
}

impl<A> actix::Actor for EventStore<A>
where
    A: Application,
{
    type Context = Context<Self>;
}

impl<A> actix::SystemService for EventStore<A> where A: Application {}
impl<A> actix::Supervised for EventStore<A> where A: Application {}
impl<A> std::default::Default for EventStore<A>
where
    A: Application,
{
    fn default() -> Self {
        unimplemented!()
    }
}
