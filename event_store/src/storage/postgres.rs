use crate::event::RecordedEvent;
use crate::event::UnsavedEvent;
use crate::storage::{
    AppendToStreamError, Storage, StorageError, StreamCreationError, StreamDeletionError,
};
use crate::stream::Stream;
use log::{debug, info, trace};
use sqlx::PgPool;
use uuid::Uuid;

mod sql;

pub struct PostgresBackend {
    pool: PgPool,
}

impl PostgresBackend {
    pub async fn with_url(url: &str) -> Result<Self, sqlx::Error> {
        Ok(Self {
            pool: PgPool::new(url).await?,
        })
    }
}

impl std::convert::From<sqlx::Error> for AppendToStreamError {
    fn from(e: sqlx::Error) -> Self {
        println!("{:?}", e);
        Self::DoesntExists
    }
}

#[async_trait::async_trait]
impl Storage for PostgresBackend {
    fn storage_name() -> &'static str {
        "PostgresBackend"
    }

    async fn create_stream(&mut self, stream: Stream) -> Result<Stream, StreamCreationError> {
        trace!("Attempting to create stream {}", stream.stream_uuid());

        let stream_uuid = stream.stream_uuid().to_owned();
        match sql::create_stream(&self.pool, &stream_uuid).await {
            Err(_) => Err(StreamCreationError::AlreadyExists),
            Ok(s) => {
                info!("Created stream {}", stream_uuid);
                Ok(s)
            }
        }
    }

    async fn delete_stream(&mut self, _stream: &Stream) -> Result<(), StreamDeletionError> {
        unimplemented!()
    }

    async fn append_to_stream(
        &mut self,
        stream_uuid: &str,
        events: &[UnsavedEvent],
    ) -> Result<Vec<Uuid>, AppendToStreamError> {
        trace!(
            "Attempting to append {} event(s) to stream {}",
            events.len(),
            stream_uuid,
        );

        match self.pool.begin().await {
            Ok(mut tx) => {
                let stream = sql::stream_info(&mut tx, stream_uuid).await?;
                let uuids = sql::transactional_insert_events(&mut tx, events).await?;
                sql::insert_stream_events(&mut tx, events, stream.stream_id).await?;
                sql::insert_link_events(&mut tx, &uuids, "$all").await?;

                tx.commit().await?;

                debug!(
                    "Successfully append {} event(s) to stream {}",
                    uuids.len(),
                    stream_uuid
                );

                Ok(uuids)
            }
            _ => Err(AppendToStreamError::DoesntExists),
        }
    }

    async fn read_stream_info(&mut self, stream_uuid: String) -> Result<Stream, StorageError> {
        match sql::stream_info(&self.pool, &stream_uuid).await {
            Err(sqlx::Error::RowNotFound) => Err(StorageError::StreamDoesntExists),
            Err(_) => Err(StorageError::Unknown),
            Ok(s) => Ok(s),
        }
    }
}
