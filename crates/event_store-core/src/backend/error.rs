/// Trait that represents a backend error
///
/// Errors that implement this trait can be wrapped by `StorageError`
pub trait BackendError: std::error::Error + 'static + Send + Sync {}
