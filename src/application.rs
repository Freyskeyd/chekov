use crate::EventHandler;
use crate::EventHandlerBuilder;
use crate::Router;
use crate::SubscriberManager;
use actix::prelude::*;
use event_store::prelude::*;
use std::pin::Pin;
use tracing::trace;

use std::any::TypeId;
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait StorageConfig<S>
where
    S: Storage,
{
    async fn build(&self) -> S;
}

pub struct PgStorageConfig {
    url: String,
}

impl PgStorageConfig {
    pub fn with_url(url: &str) -> Self {
        Self { url: url.into() }
    }
}

#[async_trait::async_trait]
impl StorageConfig<PostgresBackend> for PgStorageConfig {
    async fn build(&self) -> PostgresBackend {
        // TODO: Remove unwrap
        PostgresBackend::with_url(&self.url).await.unwrap()
    }
}

pub struct ApplicationBuilder<A: Application> {
    app: std::marker::PhantomData<A>,
    storage: Pin<Box<dyn Future<Output = A::Storage>>>,
    event_handlers: Vec<Pin<Box<dyn Future<Output = ()>>>>,
    eventmapper: HashMap<String, TypeId>,
}

impl<A> ApplicationBuilder<A>
where
    A: Application,
{
    pub fn event_handler<E: EventHandler + 'static>(mut self, handler: E) -> Self {
        self.event_handlers
            .push(Box::pin(EventHandlerBuilder::new(handler).register::<A>()));
        self
    }

    pub fn with_storage_config<CFG: StorageConfig<A::Storage> + 'static>(
        mut self,
        storage: CFG,
    ) -> Self {
        self.storage = Box::pin(async move { storage.build().await });

        self
    }

    pub fn storage<F: Future<Output = Result<A::Storage, E>> + 'static, E: std::fmt::Debug>(
        mut self,
        storage: F,
    ) -> Self {
        self.storage =
            Box::pin(async move { storage.await.expect("Unable to connect the storage") });

        self
    }

    pub fn events(mut self, events: HashMap<String, TypeId>) -> Self {
        self.eventmapper.extend(events);

        self
    }

    pub async fn launch(self) {
        trace!(
            "[[{}]] Launching a new Chekov instance with {}",
            A::get_name(),
            std::any::type_name::<A::Storage>()
        );

        let storage: A::Storage = self.storage.await;
        let event_store: EventStore<A::Storage> = EventStore::builder()
            .storage(storage)
            .build()
            .await
            .unwrap();

        use futures::future::join_all;

        join_all(self.event_handlers).await;

        let event_store_addr = event_store.start();

        let router = Router::<A> {
            _app: std::marker::PhantomData,
            _before_dispatch: vec![],
        };

        let addr = router.start();
        let subscriber_manager_addr =
            SubscriberManager::<A>::new(event_store_addr.clone(), self.eventmapper).start();

        ::actix::SystemRegistry::set(event_store_addr.clone());
        ::actix::Registry::set(addr.clone());
        ::actix::Registry::set(subscriber_manager_addr.clone());
    }
}

#[async_trait::async_trait]
pub trait Application: Unpin + 'static + Send + std::default::Default {
    type Storage: Storage;

    fn with_default() -> ApplicationBuilder<Self> {
        ApplicationBuilder {
            event_handlers: Vec::new(),
            app: std::marker::PhantomData,
            storage: Box::pin(async { Self::Storage::default() }),
            eventmapper: HashMap::new(),
        }
    }

    fn get_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}
