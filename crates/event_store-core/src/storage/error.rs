use thiserror::Error;

use crate::backend::error::BackendError;

pub trait InternalStorageError: std::error::Error + 'static + Send + Sync {}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("The stream doesn't exists")]
    StreamDoesntExists,
    #[error("The stream already exists")]
    StreamAlreadyExists,
    #[error("BackendError: {0}")]
    InternalBackendError(Box<dyn BackendError>),
}

impl<T> From<T> for StorageError
where
    T: BackendError,
{
    fn from(e: T) -> Self {
        Self::InternalBackendError(Box::new(e))
    }
}
