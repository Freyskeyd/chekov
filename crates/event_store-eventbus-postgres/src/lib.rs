use std::pin::Pin;

use event_store_core::event_bus::{BoxedStream, EventBus, EventBusMessage, EventNotification};
use futures::{FutureExt, Stream, StreamExt};
use sqlx::postgres::PgListener;

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
