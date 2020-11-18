use crate::RecordedEvent;
use crate::{storage::Storage, stream::Stream, EventStoreError};
use actix::{Actor, Context, Handler};
use actix_interop::{with_ctx, FutureInterop};
use log::{debug, trace};
use std::borrow::Cow;
use std::str::FromStr;
use uuid::Uuid;

mod messaging;

pub use messaging::{Append, CreateStream, Read, StreamInfo};

pub struct Connection<S: Storage> {
    // storage: Arc<Mutex<S>>,
    storage: S,
}

impl<S: Storage> Connection<S> {
    pub fn make(storage: S) -> Self {
        Self {
            // storage: Arc::new(Mutex::new(storage)),
            storage: storage,
        }
    }
}

impl<S: Storage> Actor for Connection<S> {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        debug!("Starting with {} storage", S::storage_name());
    }
}

impl<S: Storage> Handler<Read> for Connection<S> {
    // type Result = actix::AtomicResponse<Self, Result<Vec<RecordedEvent>, EventStoreError>>;
    type Result = actix::ResponseActFuture<Self, Result<Vec<RecordedEvent>, EventStoreError>>;

    fn handle(&mut self, msg: Read, _ctx: &mut Context<Self>) -> Self::Result {
        let stream = msg.stream;
        let limit = msg.limit;
        let version = msg.version;

        trace!("Reading {} event(s)", stream);
        // let storage = self.storage.clone();

        //         actix::AtomicResponse::new(Box::pin(
        // Box::pin(
        async move {
            let res =
                with_ctx(|actor: &mut Self, _| actor.storage.read_stream(stream, version, limit));
            // match storage
            //     // .lock()
            //     // .await
            //     .read_stream(&stream, version, limit)
            //     .await
            match res.await {
                Ok(events) => Ok(events),
                Err(_) => Err(EventStoreError::Any),
            }
        }
        .interop_actor_boxed(self)
        // )
        // ))
    }
}

impl<S: Storage> Handler<Append> for Connection<S> {
    type Result = actix::ResponseActFuture<Self, Result<Vec<Uuid>, EventStoreError>>;

    fn handle(&mut self, msg: Append, _ctx: &mut Context<Self>) -> Self::Result {
        let events = msg.events;
        let stream = msg.stream;

        trace!("Appending {} event(s) to {}", events.len(), stream);

        async move {
            match with_ctx(|actor: &mut Self, _| actor.storage.append_to_stream(&stream, &events))
                .await
            {
                Ok(events_ids) => Ok(events_ids),
                Err(_) => Err(EventStoreError::Any),
            }
        }
        .interop_actor_boxed(self)
    }
}

impl<S: Storage> Handler<CreateStream> for Connection<S> {
    // type Result = actix::AtomicResponse<Self, Result<Cow<'static, Stream>, EventStoreError>>;
    type Result = actix::ResponseActFuture<Self, Result<Cow<'static, Stream>, EventStoreError>>;

    fn handle(&mut self, msg: CreateStream, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Creating {} stream", msg.0);

        let stream = Stream::from_str(&msg.0).unwrap();
        async move {
            match with_ctx(|actor: &mut Self, _| actor.storage.create_stream(stream)).await {
                Ok(s) => Ok(Cow::Owned(s)),
                Err(_) => Err(EventStoreError::Any),
            }
        }
        .interop_actor_boxed(self)
    }
}

impl<S: Storage> Handler<StreamInfo> for Connection<S> {
    // type Result = actix::AtomicResponse<Self, Result<Cow<'static, Stream>, EventStoreError>>;
    type Result = actix::ResponseActFuture<Self, Result<Cow<'static, Stream>, EventStoreError>>;

    fn handle(&mut self, msg: StreamInfo, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Execute StreamInfo for {}", msg.0);

        async move {
            let res = with_ctx(|actor: &mut Self, _| actor.storage.read_stream_info(msg.0));
            match res.await {
                Ok(s) => Ok(Cow::Owned(s)),
                Err(e) => Err(EventStoreError::Storage(e)),
            }
        }
        .interop_actor_boxed(self)
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
        let _ = storage.create_stream(stream.clone()).await;

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

    #[actix_rt::test]
    async fn asking_for_stream_info() {
        let (connection, stream) = init_with_stream("stream_name").await;

        let result = connection
            .send(StreamInfo("stream_name".into()))
            .await
            .unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().into_owned(), stream);
    }

    #[actix_rt::test]
    async fn creating_stream() {
        let storage = InMemoryBackend::default();

        let conn: Connection<InMemoryBackend> = Connection::make(storage);

        let connection = conn.start();

        let result = connection
            .send(CreateStream("stream_name".into()))
            .await
            .unwrap();

        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn appending_to_stream() {
        let (connection, _) = init_with_stream("stream_name").await;

        let event = UnsavedEvent::try_from(&MyEvent {}).unwrap();

        let result = connection
            .send(Append {
                stream: "stream_name".into(),
                expected_version: ExpectedVersion::AnyVersion,
                events: vec![event],
            })
            .await
            .unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[actix_rt::test]
    async fn appending_to_stream_failed() {
        let (connection, _) = init_with_stream("stream_name").await;

        let event = UnsavedEvent::try_from(&MyEvent {}).unwrap();

        let result = connection
            .send(Append {
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
