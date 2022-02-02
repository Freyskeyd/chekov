use event_store_core::backend::error::BackendError;

#[derive(thiserror::Error, Debug)]
pub enum PostgresBackendError {
    #[error("Postgres SQL error: {0}")]
    SQLError(sqlx::Error),
    #[error("Postgres connection error: {0}")]
    PoolAcquisitionError(sqlx::Error),
}

impl From<sqlx::Error> for PostgresBackendError {
    fn from(e: sqlx::Error) -> Self {
        Self::SQLError(e)
    }
}

impl BackendError for PostgresBackendError {}
