#[derive(Debug, PartialEq)]
pub enum StorageError {
    StreamDoesntExists,
    StreamAlreadyExists,
    Unknown,
}