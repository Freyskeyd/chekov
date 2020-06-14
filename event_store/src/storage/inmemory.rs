use crate::storage::{Storage, StreamCreationError, StreamDeletionError};
use crate::stream::Stream;
use std::collections::HashMap;

#[derive(Default)]
pub struct InMemoryBackend {
    streams: HashMap<String, Stream>,
}

impl Storage for InMemoryBackend {
    fn create_stream(&mut self, stream: Stream) -> Result<&Stream, StreamCreationError> {
        let stream_uuid = stream.stream_uuid().to_owned();

        if self.streams.contains_key(&stream_uuid) {
            return Err(StreamCreationError::AlreadyExists);
        }

        self.streams.insert(stream_uuid.to_owned(), stream);

        Ok(self.streams.get(&stream_uuid).unwrap())
    }

    fn delete_stream(&mut self, stream: &Stream) -> Result<(), StreamDeletionError> {
        if !self.streams.contains_key(stream.stream_uuid()) {
            return Err(StreamDeletionError::DoesntExists);
        }

        self.streams.remove(stream.stream_uuid());

        Ok(())
    }
}
