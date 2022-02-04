use error::PostgresBackendError;
use event_store_core::backend::Backend;
use event_store_core::event::{RecordedEvent, UnsavedEvent};
use event_store_core::storage::StorageError;
use event_store_core::stream::Stream;
use futures::{Future, TryFutureExt};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::Instrument;
use tracing::{debug, info, trace};
use uuid::Uuid;

mod error;
mod sql;

pub struct PostgresBackend {
    pool: PgPool,
}

impl std::fmt::Debug for PostgresBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresBackend")
            .field("pool", &self.pool)
            .finish()
    }
}

impl std::default::Default for PostgresBackend {
    fn default() -> Self {
        unimplemented!()
    }
}

impl PostgresBackend {
    /// # Errors
    ///
    /// In case of Postgres connection error
    #[tracing::instrument(name = "PostgresBackend", skip(url))]
    pub async fn with_url(url: &str) -> Result<Self, sqlx::Error> {
        trace!("Creating");

        Ok(Self {
            pool: PgPoolOptions::new()
                .min_connections(40)
                .connect(url)
                .await?,
        })
    }
}

impl Backend for PostgresBackend {
    fn backend_name() -> &'static str {
        "PostgresBackend"
    }

    #[tracing::instrument(name = "PostgresBackend::CreateStream", skip(self, stream))]
    fn create_stream(
        &mut self,
        stream: Stream,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Stream, StorageError>> + Send>> {
        trace!("Attempting to create stream {}", stream.stream_uuid());

        let stream_uuid = stream.stream_uuid().to_owned();
        let mut pool = self.pool.try_acquire().unwrap();

        Box::pin(
            async move {
                let s = sql::create_stream(&mut pool, &stream_uuid).await?;

                info!("Created stream {}", stream_uuid);
                Ok(s)
            }
            .instrument(tracing::Span::current()),
        )
    }

    #[tracing::instrument(name = "PostgresBackend::DeleteStream", skip(self))]
    fn delete_stream(
        &mut self,
        _: &Stream,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send>> {
        unimplemented!()
    }

    #[tracing::instrument(
        name = "PostgresBackend::ReadStream",
        skip(self, stream_uuid, version, limit)
    )]
    fn read_stream(
        &self,
        stream_uuid: String,
        version: usize,
        limit: usize,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<RecordedEvent>, StorageError>> + Send>>
    {
        let pool = self.pool.acquire();

        Box::pin(
            async move {
                let mut conn = pool.await?;

                let stream = sql::stream_info(&mut conn, &stream_uuid).await?;

                let events = sql::read_stream(&mut conn, stream.stream_id, version, limit).await?;

                info!("Read stream {} {}", stream_uuid, events.len());

                Ok(events)
            }
            .map_err(|error| PostgresBackendError::SQLError(error).into())
            .instrument(tracing::Span::current()),
        )
    }

    #[tracing::instrument(
        name = "PostgresBackend::AppendToStream",
        skip(self, stream_uuid, events)
    )]
    fn append_to_stream(
        &mut self,
        stream_uuid: &str,
        events: &[UnsavedEvent],
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<Uuid>, StorageError>> + Send>> {
        trace!(
            "Attempting to append {} event(s) to stream {}",
            events.len(),
            stream_uuid,
        );

        let pool = self.pool.acquire();
        let stream_uuid = stream_uuid.to_string();
        let events = events.to_vec();

        Box::pin(
            async move {
                let mut conn = pool
                    .map_err(|error| PostgresBackendError::PoolAcquisitionError(error))
                    .await?;

                let events = sql::insert_events(&mut conn, &stream_uuid, &events)
                    .map_err(|error| match error {
                        sqlx::Error::RowNotFound => StorageError::StreamDoesntExists,
                        e => PostgresBackendError::SQLError(e).into(),
                    })
                    .await?;

                debug!(
                    "Successfully append {} event(s) to stream {}",
                    events.len(),
                    stream_uuid
                );

                Ok(events)
            }
            .instrument(tracing::Span::current()),
        )
    }

    #[tracing::instrument(name = "PostgresBackend::ReadStreamInfo", skip(self, stream_uuid))]
    fn read_stream_info(
        &mut self,
        stream_uuid: String,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Stream, StorageError>> + Send>> {
        let mut pool = self.pool.try_acquire().unwrap();

        Box::pin(
            async move {
                sql::stream_info(&mut pool, &stream_uuid)
                    .map_err(|error| match error {
                        sqlx::Error::RowNotFound => StorageError::StreamDoesntExists,
                        e => PostgresBackendError::SQLError(e).into(),
                    })
                    .await
            }
            .instrument(tracing::Span::current()),
        )
    }
}
