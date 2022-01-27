use std::{convert::TryFrom, pin::Pin, str::FromStr};

use actix::prelude::*;
use async_stream::try_stream;
use event_store_core::{
    event_bus::{BoxedStream, EventBus, EventBusMessage, EventNotification},
    storage::Storage,
};
use futures::{
    future::{self, BoxFuture},
    Future, FutureExt, StreamExt,
};
use sqlx::postgres::PgListener;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{connection::Connection, event::RecordedEvents};

#[derive(Message)]
#[rtype("()")]
pub struct OpenNotificationChannel {
    pub(crate) sender: mpsc::UnboundedSender<EventBusMessage>,
}

#[derive(Debug)]
pub struct InMemoryEventBus {
    receiver: Option<UnboundedReceiver<EventBusMessage>>,
    pub(crate) sender: UnboundedSender<EventBusMessage>,
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<EventBusMessage>();

        Self {
            receiver: Some(receiver),
            sender,
        }
    }
}

impl InMemoryEventBus {
    pub async fn initiate() -> Result<Self, ()> {
        Ok(Self::default())
    }

    async fn start_listening(
        mut receiver: UnboundedReceiver<EventBusMessage>,
    ) -> Pin<Box<dyn Stream<Item = Result<EventBusMessage, ()>>>> {
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

    // fn prepare<S: Storage>(&mut self, storage: Addr<Connection<S>>) -> BoxFuture<'static, ()> {
    //     let storage = storage.clone();
    //     let sender = self.sender.clone();
    //     async move {
    //         let _ = storage.send(OpenNotificationChannel { sender }).await;
    //     }
    //     .boxed()
    // }

    fn create_stream(&mut self) -> BoxedStream {
        let receiver = self.receiver.take().unwrap();

        Self::start_listening(receiver).boxed()
    }
}

#[derive(Debug)]
pub struct PostgresEventBus {
    listener: Option<PgListener>,
}

impl Default for PostgresEventBus {
    fn default() -> Self {
        unimplemented!()
    }
}

impl PostgresEventBus {
    pub async fn initiate(url: String) -> Result<Self, ()> {
        let listener = sqlx::postgres::PgListener::connect(&url).await.unwrap();

        Ok(Self {
            listener: Some(listener),
        })
    }

    async fn start_listening(
        mut listener: PgListener,
    ) -> Pin<Box<dyn Stream<Item = Result<EventBusMessage, ()>>>> {
        listener.listen("events").await.unwrap();

        listener
            .into_stream()
            .map(|res| match res {
                Ok(notification) => {
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

    // fn prepare<S: Storage>(&mut self, _: Addr<Connection<S>>) -> BoxFuture<'static, ()> {
    //     future::ready(()).boxed()
    // }

    fn create_stream(&mut self) -> BoxedStream {
        let listener = self.listener.take().unwrap();
        Self::start_listening(listener).boxed()
    }
}
