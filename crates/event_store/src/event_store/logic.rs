use super::{EventStore, EventStoreBuilder};
use crate::{
    connection::{
        Append, Connection, CreateStream, Read, StreamForward, StreamForwardResult, StreamInfo,
    },
    core::stream::Stream,
    event::RecordedEvent,
    prelude::EventStoreError,
    storage::{appender::AppendToStreamRequest, reader},
};
use actix::Addr;
use event_store_core::storage::Storage;
use tracing::info;
use uuid::Uuid;

impl<S: Storage> std::default::Default for EventStore<S> {
    fn default() -> Self {
        unimplemented!()
    }
}

impl<S: Storage> EventStore<S> {
    #[must_use]
    pub const fn builder() -> EventStoreBuilder<S> {
        EventStoreBuilder { storage: None }
    }

    pub(crate) async fn read(
        connection: Addr<Connection<S>>,
        request: reader::ReadStreamRequest,
    ) -> Result<Vec<RecordedEvent>, EventStoreError> {
        let stream: String = request.stream.to_string();

        info!("Attempting to read {} stream event(s)", stream);

        match connection
            .send(Read {
                correlation_id: request.correlation_id,
                #[cfg(feature = "verbose")]
                stream: stream.clone(),
                #[cfg(not(feature = "verbose"))]
                stream,
                version: request.version,
                limit: request.limit,
            })
            .await?
        {
            Ok(events) => {
                #[cfg(feature = "verbose")]
                info!("Read {} event(s) to {}", events.len(), stream);
                Ok(events)
            }
            Err(e) => {
                #[cfg(feature = "verbose")]
                info!("Failed to read event(s) from {}", stream);
                Err(e)
            }
        }
    }

    pub(crate) async fn stream_info(
        connection: Addr<Connection<S>>,
        request: StreamInfo,
    ) -> Result<Stream, EventStoreError> {
        connection.send(request).await?
    }

    pub(crate) async fn create_stream(
        connection: Addr<Connection<S>>,
        request: CreateStream,
    ) -> Result<Stream, EventStoreError> {
        connection.send(request).await?
    }

    pub(crate) async fn stream_forward(
        connection: Addr<Connection<S>>,
        request: StreamForward,
    ) -> Result<StreamForwardResult, EventStoreError> {
        connection.send(request).await?
    }

    pub(crate) async fn append(
        connection: Addr<Connection<S>>,
        request: AppendToStreamRequest,
    ) -> Result<Vec<Uuid>, EventStoreError> {
        let stream: String = request.stream.to_string();

        #[cfg(feature = "verbose")]
        let events_number = request.events.len();
        info!(
            "Attempting to append {} event(s) to {} with ExpectedVersion::{:?}",
            request.events.len(),
            request.stream,
            request.expected_version
        );

        match connection
            .send(Append {
                correlation_id: request.correlation_id,
                #[cfg(feature = "verbose")]
                stream: stream.clone(),
                #[cfg(not(feature = "verbose"))]
                stream,
                expected_version: request.expected_version,
                events: request.events,
            })
            .await?
        {
            Ok(events) => {
                #[cfg(feature = "verbose")]
                info!("Appended {} event(s) to {}", events.len(), stream);
                Ok(events)
            }
            Err(e) => {
                #[cfg(feature = "verbose")]
                info!("Failed to append {} event(s) to {}", events_number, stream);
                Err(e)
            }
        }
    }
}
