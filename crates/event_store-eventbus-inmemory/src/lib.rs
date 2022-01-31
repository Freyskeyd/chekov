use actix::prelude::*;
use async_stream::try_stream;
use event_store_core::event_bus::{error::EventBusError, BoxedStream, EventBus, EventBusMessage};
use futures::FutureExt;
use std::pin::Pin;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub struct InMemoryEventBus {
    receiver: Option<UnboundedReceiver<EventBusMessage>>,
    pub sender: UnboundedSender<EventBusMessage>,
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
    ) -> Pin<Box<dyn Stream<Item = Result<EventBusMessage, EventBusError>>>> {
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
