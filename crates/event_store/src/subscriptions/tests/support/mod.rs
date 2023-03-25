use std::{collections::VecDeque, sync::Arc};

use actix::{Actor, Addr, Context, Handler, ResponseFuture};
use event_store_core::{
    error::EventStoreError, event::Event, storage::Storage, versions::ExpectedVersion,
};
use futures::Future;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{subscriptions::SubscriptionNotification, EventStore};

pub mod event;
pub mod subscriber;

pub type Tracker = Arc<Mutex<VecDeque<SubscriptionNotification>>>;
pub struct InnerSub {
    pub(crate) reference: Tracker,
}

impl Actor for InnerSub {
    type Context = Context<Self>;
}

impl Handler<SubscriptionNotification> for InnerSub {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: SubscriptionNotification, _ctx: &mut Self::Context) -> Self::Result {
        let aquire = Arc::clone(&self.reference);

        Box::pin(async move {
            let mut mutex = aquire.lock().await;
            mutex.push_back(msg);

            Ok(())
        })
    }
}

pub struct EventStoreHelper<T: Storage> {
    event_store: Addr<EventStore<T>>,
}

impl<T: Storage> EventStoreHelper<T> {
    pub(crate) async fn new(storage: T) -> Self {
        let event_store = EventStore::builder()
            .storage(storage)
            .build()
            .await
            .unwrap()
            .start();

        Self { event_store }
    }

    pub(crate) fn append<E: Event>(
        &self,
        identity: &Uuid,
        version: ExpectedVersion,
        events: &[&E],
    ) -> impl Future<Output = Result<Vec<Uuid>, EventStoreError>> {
        let appender = crate::append()
            .to(identity)
            .unwrap()
            .expected_version(version)
            .events(events);

        let addr = self.get_addr();
        async move { appender.unwrap().execute(addr).await }
    }

    pub(crate) fn get_addr(&self) -> Addr<EventStore<T>> {
        self.event_store.clone()
    }
}
