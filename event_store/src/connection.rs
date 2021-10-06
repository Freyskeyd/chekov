use crate::RecordedEvent;
use crate::{storage::Storage, stream::Stream, EventStoreError};
use actix::WrapFuture;
use actix::{Actor, Context, Handler};
use std::borrow::Cow;
use std::str::FromStr;
use tracing::Instrument;
use tracing::{debug, trace};
use uuid::Uuid;

mod messaging;

pub use messaging::{Append, CreateStream, Read, StreamInfo};

pub struct Connection<S: Storage> {
    storage: S,
}

impl<S: Storage> Connection<S> {
    pub fn make(storage: S) -> Self {
        Self { storage }
    }
}

impl<S: Storage> Actor for Connection<S> {
    type Context = Context<Self>;

    #[tracing::instrument(name = "Connection", skip(self, _ctx), fields(backend = %S::storage_name()))]
    fn started(&mut self, _ctx: &mut Self::Context) {
        debug!("Starting with {} storage", S::storage_name());
    }
}

impl<S: Storage> Handler<Read> for Connection<S> {
    type Result = actix::ResponseActFuture<Self, Result<Vec<RecordedEvent>, EventStoreError>>;

    #[tracing::instrument(name = "Connection::Read", skip(self, msg, _ctx), fields(backend = %S::storage_name(), correlation_id = %msg.correlation_id))]
    fn handle(&mut self, msg: Read, _ctx: &mut Context<Self>) -> Self::Result {
        let limit = msg.limit;
        let version = msg.version;

        trace!("Reading {} event(s)", msg.stream);
        let fut = self
            .storage
            .read_stream(msg.stream, version, limit, msg.correlation_id);

        Box::pin(
            async move {
                match fut.await {
                    Ok(events) => Ok(events),
                    Err(e) => {
                        tracing::error!("Connection error: {:?}", e);
                        Err(EventStoreError::Any)
                    }
                }
            }
            .instrument(tracing::Span::current())
            .into_actor(self),
        )
    }
}

impl<S: Storage> Handler<Append> for Connection<S> {
    type Result = actix::ResponseActFuture<Self, Result<Vec<Uuid>, EventStoreError>>;

    #[tracing::instrument(name = "Connection::Append", skip(self, msg, _ctx), fields(backend = %S::storage_name(), correlation_id = %msg.correlation_id))]
    fn handle(&mut self, msg: Append, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Appending {} event(s) to {}", msg.events.len(), msg.stream);
        let fut = self
            .storage
            .append_to_stream(&msg.stream, &msg.events, msg.correlation_id);

        Box::pin(
            async move {
                match fut.await {
                    Ok(events_ids) => Ok(events_ids),
                    Err(_) => Err(EventStoreError::Any),
                }
            }
            .into_actor(self),
        )
    }
}

impl<S: Storage> Handler<CreateStream> for Connection<S> {
    type Result = actix::ResponseActFuture<Self, Result<Cow<'static, Stream>, EventStoreError>>;

    #[tracing::instrument(name = "Connection::CreateStream", skip(self, msg, _ctx), fields(backend = %S::storage_name(), correlation_id = %msg.correlation_id))]
    fn handle(&mut self, msg: CreateStream, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Creating {} stream", msg.stream_uuid);

        let stream = Stream::from_str(&msg.stream_uuid).unwrap();
        let fut = self.storage.create_stream(stream, msg.correlation_id);

        Box::pin(
            async move {
                match fut.await {
                    Ok(s) => Ok(Cow::Owned(s)),
                    Err(_) => Err(EventStoreError::Any),
                }
            }
            .into_actor(self),
        )
    }
}

impl<S: Storage> Handler<StreamInfo> for Connection<S> {
    type Result = actix::ResponseActFuture<Self, Result<Cow<'static, Stream>, EventStoreError>>;

    #[tracing::instrument(name = "Connection::StreanInfo", skip(self, msg, _ctx), fields(backend = %S::storage_name(), correlation_id = %msg.correlation_id))]
    fn handle(&mut self, msg: StreamInfo, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Execute StreamInfo for {}", msg.stream_uuid);
        let fut = self
            .storage
            .read_stream_info(msg.stream_uuid, msg.correlation_id);

        Box::pin(
            async move {
                match fut.await {
                    Ok(s) => Ok(Cow::Owned(s)),
                    Err(e) => Err(EventStoreError::Storage(e)),
                }
            }
            .into_actor(self),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::event::UnsavedEvent;
    use crate::storage::inmemory::InMemoryBackend;
    use crate::Event;
    use crate::ExpectedVersion;
    use serde::Deserialize;
    use serde::Serialize;

    async fn init_with_stream(name: &str) -> (actix::Addr<Connection<InMemoryBackend>>, Stream) {
        let mut storage = InMemoryBackend::default();
        let stream = Stream::from_str(name).unwrap();
        let _ = storage
            .create_stream(stream.clone(), uuid::Uuid::new_v4())
            .await;

        let conn: Connection<crate::storage::inmemory::InMemoryBackend> = Connection::make(storage);

        let addr = conn.start();

        (addr, stream)
    }

    #[derive(Deserialize, Serialize)]
    struct MyEvent {}
    impl Event for MyEvent {
        fn event_type(&self) -> &'static str {
            "MyEvent"
        }

        fn all_event_types() -> Vec<&'static str> {
            vec!["MyEvent"]
        }
    }

    impl std::convert::TryFrom<crate::prelude::RecordedEvent> for MyEvent {
        type Error = ();
        fn try_from(e: crate::prelude::RecordedEvent) -> Result<Self, Self::Error> {
            serde_json::from_value(e.data).map_err(|_| ())
        }
    }

    #[test]
    fn connection_can_be_created() {
        let storage = InMemoryBackend::default();
        let _conn: Connection<InMemoryBackend> = Connection::make(storage);
    }

    #[actix::test]
    async fn asking_for_stream_info() {
        let identifier = Uuid::new_v4();
        let (connection, mut stream) = init_with_stream(&identifier.to_string()).await;

        let result = connection
            .send(StreamInfo {
                correlation_id: Uuid::new_v4(),
                stream_uuid: identifier.to_string(),
            })
            .await
            .unwrap();

        assert!(result.is_ok());
        stream.stream_id = 1;
        assert_eq!(result.unwrap().into_owned(), stream);
    }

    #[actix::test]
    async fn creating_stream() {
        let storage = InMemoryBackend::default();

        let conn: Connection<InMemoryBackend> = Connection::make(storage);

        let connection = conn.start();

        let result = connection
            .send(CreateStream {
                correlation_id: Uuid::new_v4(),
                stream_uuid: "stream_name".into(),
            })
            .await
            .unwrap();

        assert!(result.is_ok());
    }

    #[actix::test]
    async fn appending_to_stream() {
        let (connection, _) = init_with_stream("stream_name").await;

        let event = UnsavedEvent::try_from(&MyEvent {}).unwrap();

        let result = connection
            .send(Append {
                correlation_id: Uuid::new_v4(),
                stream: "stream_name".into(),
                expected_version: ExpectedVersion::AnyVersion,
                events: vec![event],
            })
            .await
            .unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[actix::test]
    async fn appending_to_stream_failed() {
        let (connection, _) = init_with_stream("stream_name").await;

        let event = UnsavedEvent::try_from(&MyEvent {}).unwrap();

        let result = connection
            .send(Append {
                correlation_id: Uuid::new_v4(),
                stream: "stream_name".into(),
                expected_version: ExpectedVersion::Version(2),
                events: vec![event],
            })
            .await
            .unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}
