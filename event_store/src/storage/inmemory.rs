use crate::storage::{Storage, Stream, StreamCreationError, StreamDeletionError};
use std::collections::HashMap;

#[derive(Default)]
pub struct InMemoryBackend {
    streams: HashMap<String, Stream>,
}

impl Storage for InMemoryBackend {
    fn create_stream(&mut self, stream_uuid: &str) -> Result<&Stream, StreamCreationError> {
        // Move this in a generic function later
        // stream_uuid must be validated way before this point
        if stream_uuid.contains(' ') {
            return Err(StreamCreationError::MalformedStreamUUID);
        }

        if self.streams.contains_key(stream_uuid) {
            return Err(StreamCreationError::AlreadyExists);
        }

        self.streams
            .insert(stream_uuid.to_owned(), Stream { deleted: false });

        Ok(self.streams.get(stream_uuid).unwrap())
    }

    fn delete_stream(&mut self, stream_uuid: &str) -> Result<(), StreamDeletionError> {
        // Move this in a generic function later
        // stream_uuid must be validated way before this point
        if stream_uuid.contains(' ') {
            return Err(StreamDeletionError::MalformedStreamUUID);
        }

        if !self.streams.contains_key(stream_uuid) {
            return Err(StreamDeletionError::DoesntExists);
        }

        self.streams.remove(stream_uuid);

        Ok(())
    }
}
