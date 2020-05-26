/// A `Storage` is responsible for storing and managing `Stream` and `Event`for a `Backend`
pub trait Storage {
    /// Create a new stream with an identifier
    ///
    /// # Errors
    /// The stream creation can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to create the stream on the backend)
    /// - The stream already exists
    /// - The `stream_uuid` is invalid
    fn create_stream(&mut self, stream_uuid: &str) -> Result<&Stream, StreamCreationError>;

    /// Delete a stream from the `Backend`
    ///
    /// Do we need to provide a hard/soft deletion?
    ///
    /// # Errors
    /// The stream deletion can fail for multiple reasons:
    ///
    /// - pure storage failure (unable to delete the stream on the backend)
    /// - The stream doesn't exists
    /// - The `stream_uuid` is invalid
    fn delete_stream(&mut self, stream_uuid: &str) -> Result<(), StreamDeletionError>;
}

mod inmemory;

#[cfg(test)]
mod test;

/// Types that will be implemented later
#[derive(Debug, PartialEq)]
pub struct Stream {
    deleted: bool,
}

#[derive(Debug, PartialEq)]
pub enum StreamCreationError {
    AlreadyExists,
    MalformedStreamUUID,
    StorageError(StorageError),
}

#[derive(Debug, PartialEq)]
pub enum StreamDeletionError {
    DoesntExists,
    MalformedStreamUUID,
    StorageError(StorageError),
}

#[derive(Debug, PartialEq)]
pub enum StorageError {
    Unknown,
}
