use crate::{
    connection::{CreateStream, StreamForward, StreamForwardResult, StreamInfo, StreamList},
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
    type Result = ResponseFuture<Result<Vec<RecordedEvent>, EventStoreError>>;

    #[tracing::instrument(name = "EventStore::ReadStreamRequest", skip(self, request, _ctx), fields(correlation_id = %request.correlation_id))]
    fn handle(
        &mut self,
        request: reader::ReadStreamRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let connection = self.connection.clone();

        Box::pin(Self::read(connection, request).instrument(tracing::Span::current()))
    }
}

impl<S: Storage> Handler<StreamInfo> for EventStore<S> {
    type Result = ResponseFuture<Result<Stream, EventStoreError>>;

    #[tracing::instrument(name = "EventStore::StreamInfo", skip(self, request, _ctx), fields(correlation_id = %request.correlation_id))]
    fn handle(&mut self, request: StreamInfo, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Asking for stream {} infos", request.stream_uuid);

        Box::pin(
            Self::stream_info(self.connection.clone(), request)
                .instrument(tracing::Span::current()),
        )
    }
}

impl<S: Storage> Handler<CreateStream> for EventStore<S> {
    type Result = ResponseFuture<Result<Stream, EventStoreError>>;

    #[tracing::instrument(name = "EventStore::CreateStream", skip(self, request, _ctx), fields(correlation_id = %request.correlation_id))]
    fn handle(&mut self, request: CreateStream, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Creating stream {}", request.stream_uuid);

        Box::pin(
            Self::create_stream(self.connection.clone(), request)
                .instrument(tracing::Span::current()),
        )
    }
}

impl<S: Storage> Handler<appender::AppendToStreamRequest> for EventStore<S> {
    type Result = ResponseFuture<Result<Vec<Uuid>, EventStoreError>>;
    #[tracing::instrument(name = "EventStore::AppendToStream", skip(self, request, _ctx), fields(correlation_id = %request.correlation_id))]
    fn handle(
        &mut self,
        request: appender::AppendToStreamRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::pin(
            Self::append(self.connection.clone(), request).instrument(tracing::Span::current()),
        )
    }
}

impl<S: Storage> Handler<StreamForward> for EventStore<S> {
    type Result = ResponseFuture<Result<StreamForwardResult, EventStoreError>>;

    fn handle(&mut self, request: StreamForward, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(Self::stream_forward(self.connection.clone(), request))
    }
}

impl<S: Storage> Handler<StreamList> for EventStore<S> {
    type Result = ResponseFuture<Result<Vec<Stream>, EventStoreError>>;

    fn handle(&mut self, request: StreamList, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(Self::get_streams(self.connection.clone(), request))
    }
}
