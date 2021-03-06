use crate::event::RecordedEvent;
use crate::event::UnsavedEvent;
use crate::storage::{Storage, StorageError};
use crate::stream::Stream;
use chrono::Utc;
use futures::Future;
use std::collections::HashMap;
use tracing::{debug, trace};
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct InMemoryBackend {
    streams: HashMap<String, Stream>,
    events: HashMap<String, Vec<RecordedEvent>>,
}

impl Storage for InMemoryBackend {
    fn storage_name() -> &'static str {
        "InMemory"
    }

    #[tracing::instrument(name = "InMemoryBackend::CreateStream", skip(self, stream))]
    fn create_stream(
        &mut self,
        stream: Stream,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Stream, StorageError>> + Send>> {
        trace!("Attempting to create stream {}", stream.stream_uuid());

        let stream_uuid = stream.stream_uuid().to_owned();

        if self.streams.contains_key(&stream_uuid) {
            return Box::pin(async move { Err(StorageError::StreamAlreadyExists) });
        }

        self.streams.insert(stream_uuid.to_owned(), stream);
        self.events.insert(stream_uuid.clone(), Vec::new());

        trace!("Created stream {}", stream_uuid);

        let res = self.streams.get(&stream_uuid).unwrap().clone();
        Box::pin(async move { Ok(res) })
    }

    #[tracing::instrument(name = "InMemoryBackend::DeleteStream", skip(self, stream))]
    fn delete_stream(
        &mut self,
        stream: &Stream,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send>> {
        if !self.streams.contains_key(stream.stream_uuid()) {
            return Box::pin(async move { Err(StorageError::StreamDoesntExists) });
        }

        self.streams.remove(stream.stream_uuid());
        self.events.remove(stream.stream_uuid());

        Box::pin(async move { Ok(()) })
    }

    #[tracing::instrument(name = "InMemoryBackend::ReadStream", skip(self))]
    fn read_stream(
        &self,
        _: String,
        _: usize,
        _: usize,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<RecordedEvent>, StorageError>> + Send>>
    {
        Box::pin(async { Ok(vec![]) })
    }

    #[tracing::instrument(
        name = "InMemoryBackend::AppendToStream",
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
        if !self.streams.contains_key(stream_uuid) {
            trace!("Stream {} does not exists", stream_uuid);
            return Box::pin(async move { Err(StorageError::StreamDoesntExists) });
        }

        if let Some(e) = self.events.get_mut(stream_uuid) {
            let mut re: Vec<RecordedEvent> = events
                .iter()
                .map(|event| RecordedEvent {
                    event_number: 0,
                    event_uuid: Uuid::new_v4(),
                    stream_uuid: event.stream_uuid.clone(),
                    stream_version: Some(event.stream_version),
                    causation_id: event.causation_id,
                    correlation_id: event.correlation_id,
                    event_type: event.event_type.clone(),
                    data: serde_json::to_value(&event.data).unwrap(),
                    metadata: None,
                    created_at: Utc::now(),
                })
                .collect();

            let events = re.clone();
            e.append(&mut re);
            debug!(
                "Successfully append {} event(s) to {}",
                events.len(),
                stream_uuid
            );

            Box::pin(async move { Ok(events.iter().map(|e| e.event_uuid).collect()) })
        } else {
            Box::pin(async move { Err(StorageError::StreamDoesntExists) })
        }
    }

    #[tracing::instrument(name = "InMemoryBackend::ReadStreamInfo", skip(self, stream_uuid))]
    fn read_stream_info(
        &mut self,
        stream_uuid: String,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Stream, StorageError>> + Send>> {
        let res = self
            .streams
            .get(&stream_uuid)
            .cloned()
            .ok_or(StorageError::StreamDoesntExists);

        Box::pin(async move { res })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_storage_name() {
        assert_eq!("InMemory", InMemoryBackend::storage_name());
    }

    #[test]
    fn can_be_instantiate() {
        let _storage = InMemoryBackend::default();
    }
}
