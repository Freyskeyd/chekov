use crate::event::handler::EventHandlerBuilder;
use crate::event::EventHandler;
use crate::message::StartListening;
use crate::Application;
use crate::Router;
use crate::SubscriberManager;
use actix::Actor;
use actix::SystemService;
use event_store::prelude::Storage;
use futures::Future;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::pin::Pin;
use tracing::trace;

use super::InternalApplication;

#[async_trait::async_trait]
pub trait StorageConfig<S>
where
    S: Storage,
{
    async fn build(&self) -> S;
}

/// Struct to configure and launch an `Application` instance
///
/// An application can only be launch though an ApplicationBuilder.
///
/// This builder will have the responsability to configure every part of the application.
///
/// Note: In most cases you will not have to use `ApplicationBuilder` directly.
/// `Application` will be the root of everything.
pub struct ApplicationBuilder<A: Application> {
    app: std::marker::PhantomData<A>,
    storage: Pin<Box<dyn Future<Output = A::Storage>>>,
    event_handlers: Vec<Pin<Box<dyn Future<Output = ()>>>>,
    listener_url: String,
}

impl<A: Application> std::default::Default for ApplicationBuilder<A> {
    fn default() -> Self {
        Self {
            listener_url: String::new(),
            event_handlers: Vec::new(),
            app: std::marker::PhantomData,
            storage: Box::pin(async { A::Storage::default() }),
        }
    }
}

impl<A> ApplicationBuilder<A>
where
    A: Application,
{
    pub fn listener_url(mut self, url: String) -> Self {
        self.listener_url = url;

        self
    }

    /// Adds an EventHandler to the application
    pub fn event_handler<E: EventHandler + 'static>(mut self, handler: E) -> Self {
        self.event_handlers
            .push(Box::pin(EventHandlerBuilder::new(handler).register::<A>()));
        self
    }

    /// Adds a StorageConfig use later to create the event_store Storage
    pub fn with_storage_config<CFG: StorageConfig<A::Storage> + 'static>(
        mut self,
        storage: CFG,
    ) -> Self {
        self.storage = Box::pin(async move { storage.build().await });

        self
    }

    /// Adds a preconfigured Backend as Storage.
    /// The storage isn't start, only when the application is launched.
    pub fn storage<F: Future<Output = Result<A::Storage, E>> + 'static, E: std::fmt::Debug>(
        mut self,
        storage: F,
    ) -> Self {
        self.storage =
            Box::pin(async move { storage.await.expect("Unable to connect the storage") });

        self
    }

    /// Launch the application
    ///
    /// Meaning that:
    /// - The storage future is resolved and registered
    /// - An EventStore instance is started based on this storage
    /// - A Router is started to handle commands
    /// - A SubscriberManager is started to dispatch incomming events
    #[tracing::instrument(name = "Chekov::Launch", skip(self), fields(app = %A::get_name()))]
    pub async fn launch(self) {
        trace!(
            "Launching a new Chekov instance with {}",
            std::any::type_name::<A::Storage>()
        );

        let storage: A::Storage = self.storage.await;
        let event_store: event_store::EventStore<A::Storage> = event_store::EventStore::builder()
            .storage(storage)
            .build()
            .await
            .unwrap();

        let subscriber_manager_addr = SubscriberManager::<A>::new(self.listener_url).start();

        ::actix::SystemRegistry::set(subscriber_manager_addr);

        use futures::future::join_all;

        join_all(self.event_handlers).await;

        let event_store_addr = event_store.start();

        let router = Router::<A> {
            _app: std::marker::PhantomData,
            _before_dispatch: vec![],
        };

        let addr = router.start();
        let event_store = crate::event_store::EventStore::<A> {
            addr: event_store_addr,
        }
        .start();

        SubscriberManager::<A>::from_registry().do_send(StartListening);

        ::actix::SystemRegistry::set(event_store);
        ::actix::SystemRegistry::set(addr);
        ::actix::SystemRegistry::set(
            InternalApplication::<A> {
                _phantom: PhantomData,
            }
            .start(),
        );
    }
}
