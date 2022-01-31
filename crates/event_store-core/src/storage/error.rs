use thiserror::Error;

// Convenience type alias for usage within event_store.
type BoxDynError = Box<dyn std::error::Error + 'static + Send + Sync>;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("The stream doesn't exists")]
    StreamDoesntExists,
    #[error("The stream already exists")]
    StreamAlreadyExists,
    #[error("Internal storage error: {0}")]
    InternalStorageError(#[source] BoxDynError),
}
