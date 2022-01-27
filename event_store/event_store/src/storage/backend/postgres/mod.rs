use crate::core::event::{RecordedEvent, UnsavedEvent};
use crate::core::stream::Stream;
use event_store_core::storage::{Backend, StorageError};
use futures::Future;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::Instrument;
use tracing::{debug, info, trace};
use uuid::Uuid;

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
                match sql::create_stream(&mut pool, &stream_uuid).await {
                    Err(_) => Err(StorageError::StreamAlreadyExists),
                    Ok(s) => {
                        info!("Created stream {}", stream_uuid);
                        Ok(s)
                    }
                }
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
                match pool.await {
                    Ok(mut conn) => match sql::stream_info(&mut conn, &stream_uuid).await {
                        // TODO Remove this hack, should be nicely handled by the client
                        Err(_) => Ok(vec![]),
                        Ok(stream) => {
                            match sql::read_stream(&mut conn, stream.stream_id, version, limit)
                                .await
                            {
                                Err(e) => {
                                    tracing::error!("{:?}", e);
                                    Err(StorageError::StreamAlreadyExists)
                                }
                                Ok(s) => {
                                    info!("Read stream {} {}", stream_uuid, s.len());
                                    Ok(s)
                                }
                            }
                        }
                    },
                    Err(..) => Err(StorageError::Unknown),
                }
            }
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
                match pool.await {
                    Ok(mut conn) => {
                        let events = sql::insert_events(&mut conn, &stream_uuid, &events)
                            .await
                            .map_err(|_| StorageError::StreamDoesntExists)?;
                        debug!(
                            "Successfully append {} event(s) to stream {}",
                            events.len(),
                            stream_uuid
                        );

                        Ok(events)
                    }
                    Err(_) => Err(StorageError::StreamDoesntExists),
                }
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
                match sql::stream_info(&mut pool, &stream_uuid).await {
                    Err(sqlx::Error::RowNotFound) => Err(StorageError::StreamDoesntExists),
                    Err(_) => Err(StorageError::Unknown),
                    Ok(s) => Ok(s),
                }
            }
            .instrument(tracing::Span::current()),
        )
    }
}
