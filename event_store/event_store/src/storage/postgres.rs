use event_store_backend_postgres::PostgresBackend;
use event_store_core::{
    event_bus::{BoxedStream, EventBus},
    storage::Storage,
};

use super::event_bus::PostgresEventBus;

#[derive(Debug, Default)]
pub struct PostgresStorage {
    backend: PostgresBackend,
    event_bus: PostgresEventBus,
}

impl PostgresStorage {
    /// # Errors
    ///
    /// In case of Postgres connection error
    #[tracing::instrument(name = "PostgresBackend", skip(url))]
    pub async fn with_url(url: &str) -> Result<Self, sqlx::Error> {
        Ok(Self {
            backend: PostgresBackend::with_url(url).await?,
            // TODO Add DatabaseError convertor
            event_bus: PostgresEventBus::initiate(url.into()).await.unwrap(),
        })
    }
}

impl Storage for PostgresStorage {
    type Backend = PostgresBackend;
    type EventBus = PostgresEventBus;

    fn storage_name() -> &'static str {
        "Postgres"
    }

    fn backend(&mut self) -> &mut Self::Backend {
        &mut self.backend
    }

    fn create_stream(&mut self) -> BoxedStream {
        self.event_bus.create_stream()
    }
}
