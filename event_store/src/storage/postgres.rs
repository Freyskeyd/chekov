use crate::event::RecordedEvent;
use crate::event::UnsavedEvent;
use crate::storage::{Storage, StorageError};
use crate::stream::Stream;
use futures::Future;
use sqlx::PgPool;
use sqlx::{postgres::PgPoolOptions, Acquire};
use tracing::Instrument;
use tracing::{debug, info, trace};
use uuid::Uuid;

mod sql;

#[derive(Debug)]
pub struct PostgresBackend {
    pool: PgPool,
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
    pub async fn with_url(url: &str) -> Result<Self, sqlx::Error> {
        trace!("Connecting a new PostgresBackend");

        Ok(Self {
            pool: PgPoolOptions::new()
                .min_connections(40)
                .connect(url)
                .await?,
        })
    }
}

impl std::convert::From<sqlx::Error> for StorageError {
    fn from(e: sqlx::Error) -> Self {
        println!("{:?}", e);
        Self::StreamDoesntExists
    }
}

impl Storage for PostgresBackend {
    fn storage_name() -> &'static str {
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
                    Ok(mut conn) => {
                        match sql::read_stream(&mut conn, &stream_uuid, version, limit).await {
                            Err(_) => Err(StorageError::StreamAlreadyExists),
                            Ok(s) => {
                                info!("Read stream {} {}", stream_uuid, s.len());
                                Ok(s)
                            }
                        }
                    }
                    _ => Err(StorageError::Unknown),
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
                    Ok(mut conn) => match conn.begin().await {
                        Ok(mut tx) => {
                            let stream = sql::stream_info(&mut tx, &stream_uuid).await?;
                            let uuids = sql::transactional_insert_events(&mut tx, &events).await?;
                            sql::insert_stream_events(&mut tx, &events, stream.stream_id).await?;
                            // 0 -> is $all stream
                            sql::insert_link_events(&mut tx, &uuids, 0).await?;

                            tx.commit().await?;

                            debug!(
                                "Successfully append {} event(s) to stream {}",
                                uuids.len(),
                                stream_uuid
                            );

                            Ok(uuids)
                        }
                        Err(_) => Err(StorageError::StreamDoesntExists),
                    },
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
