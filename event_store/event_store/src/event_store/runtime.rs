use std::borrow::Cow;

use crate::{
    connection::{CreateStream, StreamInfo},
    core::stream::Stream,
    event::RecordedEvent,
    prelude::EventStoreError,
    storage::{appender, reader},
};

use super::EventStore;
use actix::prelude::*;
use event_store_core::{event_bus::EventBusMessage, storage::Storage};
use tracing::{debug, Instrument};
use uuid::Uuid;

impl<S: Storage> Supervised for EventStore<S> {}
impl<S: Storage> Actor for EventStore<S> {
    type Context = ::actix::Context<Self>;

    #[tracing::instrument(name = "EventStore::Started", skip(self, _ctx))]
    fn started(&mut self, _ctx: &mut Self::Context) {}
}

impl<S: Storage> StreamHandler<EventBusMessage> for EventStore<S> {
    fn handle(&mut self, item: EventBusMessage, _ctx: &mut Context<Self>) {
        debug!("EventBusMessage {:?}", item);
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        debug!("finished");
    }
}

impl<S: Storage> Handler<reader::ReadStreamRequest> for EventStore<S> {
    type Result = actix::ResponseActFuture<Self, Result<Vec<RecordedEvent>, EventStoreError>>;

    #[tracing::instrument(name = "EventStore::ReadStreamRequest", skip(self, request, _ctx), fields(correlation_id = %request.correlation_id))]
    fn handle(
        &mut self,
        request: reader::ReadStreamRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let connection = self.connection.clone();

        let fut = Self::read(connection, request)
            .instrument(tracing::Span::current())
            .into_actor(self);

        Box::pin(fut)
    }
}

impl<S: Storage> Handler<StreamInfo> for EventStore<S> {
    type Result = actix::ResponseActFuture<Self, Result<Cow<'static, Stream>, EventStoreError>>;

    #[tracing::instrument(name = "EventStore::StreamInfo", skip(self, request, _ctx), fields(correlation_id = %request.correlation_id))]
    fn handle(&mut self, request: StreamInfo, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Asking for stream {} infos", request.stream_uuid);

        Box::pin(
            Self::stream_info(self.connection.clone(), request)
                .instrument(tracing::Span::current())
                .into_actor(self),
        )
    }
}

impl<S: Storage> Handler<CreateStream> for EventStore<S> {
    type Result = actix::ResponseActFuture<Self, Result<Cow<'static, Stream>, EventStoreError>>;

    #[tracing::instrument(name = "EventStore::CreateStream", skip(self, request, _ctx), fields(correlation_id = %request.correlation_id))]
    fn handle(&mut self, request: CreateStream, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Creating stream {}", request.stream_uuid);

        Box::pin(
            Self::create_stream(self.connection.clone(), request)
                .instrument(tracing::Span::current())
                .into_actor(self),
        )
    }
}

impl<S: Storage> Handler<appender::AppendToStreamRequest> for EventStore<S> {
    type Result = actix::ResponseActFuture<Self, Result<Vec<Uuid>, EventStoreError>>;

    #[tracing::instrument(name = "EventStore::AppendToStream", skip(self, request, _ctx), fields(correlation_id = %request.correlation_id))]
    fn handle(
        &mut self,
        request: appender::AppendToStreamRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::pin(
            Self::append(self.connection.clone(), request)
                .instrument(tracing::Span::current())
                .into_actor(self),
        )
    }
}
