use crate::{storage::Storage, stream::Stream, EventStoreError};
use actix::{Actor, Context, Handler, WrapFuture};
use log::{debug, trace};
use std::borrow::Cow;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

mod messaging;

pub use messaging::{Append, CreateStream, StreamInfo};

pub struct Connection<S: Storage> {
    storage: Arc<Mutex<S>>,
}

impl<S: Storage> Connection<S> {
    pub fn make(storage: S) -> Self {
        Self {
            storage: Arc::new(Mutex::new(storage)),
        }
    }
}

impl<S: Storage> Actor for Connection<S> {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        debug!("Starting with {} storage", S::storage_name());
    }
}

impl<S: Storage> Handler<Append> for Connection<S> {
    type Result = actix::AtomicResponse<Self, Result<Vec<Uuid>, EventStoreError>>;

    fn handle(&mut self, msg: Append, _ctx: &mut Context<Self>) -> Self::Result {
        let events = msg.events;
        let stream = msg.stream;

        trace!("Appending {} event(s) to {}", events.len(), stream);
        let storage = self.storage.clone();

        actix::AtomicResponse::new(Box::pin(
            async move {
                match storage
                    .lock()
                    .await
                    .append_to_stream(&stream, &events)
                    .await
                {
                    Ok(events_ids) => Ok(events_ids),
                    Err(_) => Err(EventStoreError::Any),
                }
            }
            .into_actor(self),
        ))
    }
}

impl<S: Storage> Handler<CreateStream> for Connection<S> {
    type Result = actix::AtomicResponse<Self, Result<Cow<'static, Stream>, EventStoreError>>;

    fn handle(&mut self, msg: CreateStream, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Creating {} stream", msg.0);
        let storage = self.storage.clone();

        let stream = Stream::from_str(&msg.0).unwrap();
        actix::AtomicResponse::new(Box::pin(
            async move {
                match storage.lock().await.create_stream(stream).await {
                    Ok(s) => Ok(Cow::Owned(s)),
                    Err(_) => Err(EventStoreError::Any),
                }
            }
            .into_actor(self),
        ))
    }
}

impl<S: Storage> Handler<StreamInfo> for Connection<S> {
    type Result = actix::AtomicResponse<Self, Result<Cow<'static, Stream>, EventStoreError>>;

    fn handle(&mut self, msg: StreamInfo, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Execute StreamInfo for {}", msg.0);

        let storage = self.storage.clone();

        actix::AtomicResponse::new(Box::pin(
            async move {
                match storage.lock().await.read_stream_info(msg.0).await {
                    Ok(s) => Ok(Cow::Owned(s)),
                    Err(e) => Err(EventStoreError::Storage(e)),
                }
            }
            .into_actor(self),
        ))
    }
}
