use std::{convert::TryFrom, pin::Pin, str::FromStr};

use actix::prelude::*;
use async_stream::try_stream;
use futures::{
    future::{self, BoxFuture},
    Future, FutureExt, StreamExt,
};
use sqlx::postgres::PgListener;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{trace, Instrument, Span};
use uuid::Uuid;

use crate::{
    connection::Connection, event::RecordedEvents, prelude::Storage,
    storage::reader::ReadStreamRequest, EventStore,
};

pub type BoxedStream =
    Pin<Box<dyn Future<Output = Pin<Box<dyn Stream<Item = Result<EventBusMessage, ()>>>>>>>;

pub trait EventBus: std::fmt::Debug + Default + Send + std::marker::Unpin + 'static {
    fn bus_name() -> &'static str;

    fn prepare<S: Storage>(&mut self, storage: Addr<Connection<S>>) -> BoxFuture<'static, ()>;
    fn create_stream(self) -> BoxedStream;
}

#[derive(Debug, Message)]
#[rtype("()")]
pub enum EventBusMessage {
    Notification(EventNotification),
    Events(RecordedEvents),
    Unkown,
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct EventNotification {
    pub(crate) stream_id: i32,
    pub(crate) stream_uuid: String,
    pub(crate) first_stream_version: i32,
    pub(crate) last_stream_version: i32,
}

struct NonEmptyString(String);

impl FromStr for NonEmptyString {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("unable to parse stream_uuid")
        } else {
            Ok(NonEmptyString(s.to_owned()))
        }
    }
}
impl From<NonEmptyString> for String {
    fn from(s: NonEmptyString) -> Self {
        s.0
    }
}

impl<'a> TryFrom<&'a str> for EventNotification {
    type Error = &'static str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut through = value.splitn(4, ',');

        let stream_uuid = through
            .next()
            .ok_or("unable to parse stream_uuid")?
            .parse::<NonEmptyString>()?
            .into();

        let stream_id = through
            .next()
            .ok_or("unable to parse stream_id")?
            .parse::<i32>()
            .or(Err("unable to convert stream_id to i32"))?;

        let first_stream_version = through
            .next()
            .ok_or("unable to parse first_stream_version")?
            .parse::<i32>()
            .or(Err("unable to convert first_stream_version to i32"))?;

        let last_stream_version = through
            .next()
            .ok_or("unable to parse last_stream_version")?
            .parse::<i32>()
            .or(Err("unable to convert last_stream_version to i32"))?;

        Ok(Self {
            stream_uuid,
            stream_id,
            first_stream_version,
            last_stream_version,
        })
    }
}

pub struct EventBusConnection<S: Storage> {
    storage: Addr<Connection<S>>,
}

impl<S: Storage> EventBusConnection<S> {
    pub fn make<E: EventBus>(mut event_bus: E, storage: Addr<Connection<S>>) -> Addr<Self> {
        Self::create(move |ctx| {
            let instance = Self {
                storage: storage.clone(),
            };
            async move {
                event_bus.prepare(storage).await;
                event_bus.create_stream().await
            }
            .into_actor(&instance)
            .map(|res, _actor, ctx| {
                ctx.add_stream(res);
            })
            .wait(ctx);

            instance
        })
    }
}

impl<S: Storage> Actor for EventBusConnection<S> {
    type Context = Context<Self>;

    #[tracing::instrument(name = "EventBusConnection", skip(self, _ctx))]
    fn started(&mut self, _ctx: &mut Self::Context) {}
}

impl<S: Storage> StreamHandler<Result<EventBusMessage, ()>> for EventBusConnection<S> {
    fn handle(&mut self, item: Result<EventBusMessage, ()>, ctx: &mut Context<Self>) {
        if let Ok(message) = item {
            match message {
                EventBusMessage::Events(events) => {
                    println!("events received {:?}", events);
                }
                EventBusMessage::Notification(notification) => {
                    trace!(
                        "Fetching Related RecordedEvents for {}",
                        notification.stream_uuid
                    );
                    let storage = self.storage.clone();
                    let _ = ctx.address();
                    let stream_uuid = notification.stream_uuid.clone();
                    let fut = async move {
                        let correlation_id = Uuid::new_v4();

                        let request = ReadStreamRequest{
                            correlation_id,
                            span: tracing::span!(parent: Span::current(), tracing::Level::TRACE, "ReadStreamRequest", correlation_id = ?correlation_id),
                            stream: stream_uuid,
                            version: notification.first_stream_version as usize,
                            limit: (notification.last_stream_version
                                    - notification.first_stream_version
                                    + 1) as usize
                        };

                        EventStore::<S>::read(storage, request).await
                    }
                    .in_current_span()
                    .into_actor(self)
                    .map(move |_res, _actor, _ctx| {
                    });

                    ctx.spawn(fut);
                }
                EventBusMessage::Unkown => {}
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        println!("finished");
    }
}

#[derive(Message)]
#[rtype("()")]
pub struct OpenNotificationChannel {
    pub(crate) sender: mpsc::UnboundedSender<EventBusMessage>,
}

#[derive(Debug)]
pub struct InMemoryEventBus {
    receiver: UnboundedReceiver<EventBusMessage>,
    sender: UnboundedSender<EventBusMessage>,
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<EventBusMessage>();

        Self { receiver, sender }
    }
}

impl InMemoryEventBus {
    pub async fn initiate() -> Result<Self, ()> {
        Ok(Self::default())
    }

    async fn start_listening(self) -> Pin<Box<dyn Stream<Item = Result<EventBusMessage, ()>>>> {
        let mut receiver = self.receiver;

        Box::pin(try_stream! {
            while let Some(event) = receiver.recv().await {
                yield event;
            }
        })
    }
}

impl EventBus for InMemoryEventBus {
    fn bus_name() -> &'static str {
        "InMemoryEventBus"
    }

    fn prepare<S: Storage>(&mut self, storage: Addr<Connection<S>>) -> BoxFuture<'static, ()> {
        let storage = storage.clone();
        let sender = self.sender.clone();
        async move {
            let _ = storage.send(OpenNotificationChannel { sender }).await;
        }
        .boxed()
    }

    fn create_stream(self) -> BoxedStream {
        let fut = Box::pin(async move { self.start_listening().await });

        fut
    }
}
#[derive(Debug)]
pub struct PostgresEventBus {
    listener: PgListener,
}

impl Default for PostgresEventBus {
    fn default() -> Self {
        unimplemented!()
    }
}

impl PostgresEventBus {
    pub async fn initiate(url: String) -> Result<Self, ()> {
        let listener = sqlx::postgres::PgListener::connect(&url).await.unwrap();

        Ok(Self { listener })
    }

    async fn start_listening(mut self) -> Pin<Box<dyn Stream<Item = Result<EventBusMessage, ()>>>> {
        self.listener.listen("events").await.unwrap();

        self.listener
            .into_stream()
            .map(|res| match res {
                Ok(notification) => {
                    println!("Notification: {:?}", notification);
                    if let Ok(event) = EventNotification::try_from(notification.payload()) {
                        return Ok(EventBusMessage::Notification(event));
                    }

                    Ok(EventBusMessage::Unkown)
                }
                Err(_) => Err(()),
            })
            .boxed()
    }
}

impl EventBus for PostgresEventBus {
    fn bus_name() -> &'static str {
        "PostgresEventBus"
    }

    fn prepare<S: Storage>(&mut self, _: Addr<Connection<S>>) -> BoxFuture<'static, ()> {
        future::ready(()).boxed()
    }

    fn create_stream(self) -> BoxedStream {
        let fut = Box::pin(async move { self.start_listening().await });

        fut
    }
}
