use crate::core::event::RecordedEvent;
use crate::core::stream::Stream;
use crate::subscriptions::pub_sub::{PubSub, PubSubNotification};
use crate::EventStoreError;
use actix::{Actor, Addr, AsyncContext, Context, Handler, ResponseFuture, SystemService};
use actix::{ActorFutureExt, Message};
use actix::{StreamHandler, WrapFuture};
use event_store_core::backend::Backend;
use event_store_core::event_bus::error::EventBusError;
use event_store_core::event_bus::EventBusMessage;
use event_store_core::storage::Storage;
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use std::str::FromStr;
use tokio::sync::mpsc;
use tracing::trace;
use tracing::Instrument;
use uuid::Uuid;

mod messaging;

pub use messaging::{Append, CreateStream, Read, StreamForward, StreamForwardResult, StreamInfo};

#[derive(Message)]
#[rtype("()")]
// TODO: Remove this by a better subscription channel
pub struct OpenNotificationChannel {
    pub(crate) sender: mpsc::UnboundedSender<EventBusMessage>,
}

#[derive(Debug)]
pub struct Connection<S: Storage> {
    storage: S,
    pub_sub: Addr<PubSub>,
}

impl<S: Storage> Connection<S> {
    pub fn make(storage: S) -> Self {
        Self {
            storage,
            pub_sub: PubSub::from_registry(),
        }
    }
}

impl<S: Storage> Actor for Connection<S> {
    type Context = Context<Self>;

    #[tracing::instrument(name = "Connection", skip(self, ctx), fields(backend = %S::storage_name()))]
    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("Starting with {} storage", S::storage_name());
        let event_bus = self.storage.create_stream();
        ctx.wait(event_bus.into_actor(self).map(|stream, _, ctx| {
            ctx.add_stream(stream);
        }));
    }
}

impl<S: Storage> StreamHandler<Result<EventBusMessage, EventBusError>> for Connection<S> {
    fn handle(&mut self, item: Result<EventBusMessage, EventBusError>, _ctx: &mut Context<Self>) {
        trace!("Received Message on event bus");
        if let Ok(message) = item {
            match message {
                EventBusMessage::Events(stream_uuid, events) => {
                    trace!("spawning future to pubsub");

                    tokio::spawn(
                        self.pub_sub
                            .send(PubSubNotification {
                                stream: stream_uuid,
                                events,
                            })
                            .in_current_span(),
                    );
                }
                EventBusMessage::Notification(notification) => {
                    trace!(
                        "Fetching Related RecordedEvents for {}",
                        notification.stream_uuid
                    );

                    let stream_uuid = notification.stream_uuid;
                    let correlation_id = Uuid::new_v4();
                    let pub_sub = self.pub_sub.clone();

                    let fut = self
                        .storage
                        .backend()
                        .read_stream(
                            stream_uuid.to_string(),
                            notification.first_stream_version as usize,
                            (notification.last_stream_version - notification.first_stream_version
                                + 1) as usize,
                            correlation_id,
                        )
                        .in_current_span()
                        .map(move |res| {
                            res.map_or_else(
                                |_| todo!(),
                                |events| {
                                    pub_sub.send(PubSubNotification {
                                        stream: stream_uuid,
                                        events,
                                    })
                                },
                            )
                        });

                    tokio::spawn(fut);
                }
                EventBusMessage::Unkown => {}
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        trace!("finished");
    }
}

impl<S: Storage> Handler<OpenNotificationChannel> for Connection<S> {
    type Result = ();

    fn handle(&mut self, msg: OpenNotificationChannel, _ctx: &mut Self::Context) -> Self::Result {
        self.storage.direct_channel(msg.sender);
    }
}

impl<S: Storage> Handler<Read> for Connection<S> {
    type Result = ResponseFuture<Result<Vec<RecordedEvent>, EventStoreError>>;

    #[tracing::instrument(name = "Connection::Read", skip(self, msg, _ctx), fields(backend = %S::storage_name(), correlation_id = %msg.correlation_id))]
    fn handle(&mut self, msg: Read, _ctx: &mut Context<Self>) -> Self::Result {
        let limit = msg.limit;
        let version = msg.version;

        trace!("Reading {} event(s)", msg.stream);

        self.storage
            .backend()
            .read_stream(msg.stream, version, limit, msg.correlation_id)
            .map_err(Into::into)
            .instrument(tracing::Span::current())
            .boxed()
    }
}

impl<S: Storage> Handler<Append> for Connection<S> {
    type Result = ResponseFuture<Result<Vec<Uuid>, EventStoreError>>;

    #[tracing::instrument(name = "Connection::Append", skip(self, msg, _ctx), fields(backend = %S::storage_name(), correlation_id = %msg.correlation_id))]
    fn handle(&mut self, msg: Append, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Appending {} event(s) to {}", msg.events.len(), msg.stream);

        self.storage
            .backend()
            .append_to_stream(&msg.stream, &msg.events, msg.correlation_id)
            .map_err(Into::into)
            .boxed()
    }
}

impl<S: Storage> Handler<CreateStream> for Connection<S> {
    type Result = ResponseFuture<Result<Stream, EventStoreError>>;

    #[tracing::instrument(name = "Connection::CreateStream", skip(self, msg, _ctx), fields(backend = %S::storage_name(), correlation_id = %msg.correlation_id))]
    fn handle(&mut self, msg: CreateStream, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Creating {} stream", msg.stream_uuid);

        let stream = Stream::from_str(&msg.stream_uuid).unwrap();

        self.storage
            .backend()
            .create_stream(stream, msg.correlation_id)
            .map_err(Into::into)
            .boxed()
    }
}

impl<S: Storage> Handler<StreamInfo> for Connection<S> {
    type Result = ResponseFuture<Result<Stream, EventStoreError>>;

    #[tracing::instrument(name = "Connection::StreanInfo", skip(self, msg, _ctx), fields(backend = %S::storage_name(), correlation_id = %msg.correlation_id))]
    fn handle(&mut self, msg: StreamInfo, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Execute StreamInfo for {}", msg.stream_uuid);

        self.storage
            .backend()
            .read_stream_info(msg.stream_uuid, msg.correlation_id)
            .map_err(Into::into)
            .boxed()
    }
}

impl<S: Storage> Handler<StreamForward> for Connection<S> {
    type Result = Result<StreamForwardResult, EventStoreError>;

    #[tracing::instrument(name = "Connection::StreanForward", skip(self, msg, _ctx), fields(backend = %S::storage_name(), correlation_id = %msg.correlation_id))]
    fn handle(&mut self, msg: StreamForward, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Execute StreamForward for {}", msg.stream_uuid);

        Ok(StreamForwardResult {
            stream: self
                .storage
                .backend()
                .stream_forward(msg.stream_uuid, 100, msg.correlation_id)
                .map_err(Into::into)
                .boxed(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::core::event::Event;
    use crate::core::stream::Stream;

    use crate::event::UnsavedEvent;
    use crate::storage::InMemoryStorage;
    use crate::ExpectedVersion;
    use serde::Deserialize;
    use serde::Serialize;

    async fn init_with_stream(name: &str) -> (actix::Addr<Connection<InMemoryStorage>>, Stream) {
        let mut storage = InMemoryStorage::default();
        let stream = Stream::from_str(name).unwrap();
        let _ = storage
            .backend()
            .create_stream(stream.clone(), uuid::Uuid::new_v4())
            .await;

        let conn: Connection<crate::storage::InMemoryStorage> = Connection::make(storage);

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

    #[actix::test]
    async fn connection_can_be_created() {
        let storage = InMemoryStorage::default();
        let _conn: Connection<InMemoryStorage> = Connection::make(storage);
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
        assert_eq!(result.unwrap(), stream);
    }

    #[actix::test]
    async fn creating_stream() {
        let storage = InMemoryStorage::default();

        let conn: Connection<InMemoryStorage> = Connection::make(storage);

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
