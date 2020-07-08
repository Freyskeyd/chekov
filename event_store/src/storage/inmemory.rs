use crate::event::RecordedEvent;
use crate::event::UnsavedEvent;
use crate::storage::{AppendToStreamError, Storage, StreamCreationError, StreamDeletionError};
use crate::stream::Stream;
use log::{debug, trace};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryBackend {
    streams: HashMap<String, Stream>,
    events: HashMap<String, Vec<RecordedEvent>>,
}

#[async_trait::async_trait]
impl Storage for InMemoryBackend {
    fn storage_name() -> &'static str {
        "InMemory"
    }

    async fn create_stream(&mut self, stream: Stream) -> Result<&Stream, StreamCreationError> {
        trace!("Attempting to create stream {}", stream.stream_uuid());

        let stream_uuid = stream.stream_uuid().to_owned();

        if self.streams.contains_key(&stream_uuid) {
            return Err(StreamCreationError::AlreadyExists);
        }

        self.streams.insert(stream_uuid.to_owned(), stream);
        self.events.insert(stream_uuid.clone(), Vec::new());

        trace!("Created stream {}", stream_uuid);

        Ok(self.streams.get(&stream_uuid).unwrap())
    }

    async fn delete_stream(&mut self, stream: &Stream) -> Result<(), StreamDeletionError> {
        if !self.streams.contains_key(stream.stream_uuid()) {
            return Err(StreamDeletionError::DoesntExists);
        }

        self.streams.remove(stream.stream_uuid());
        self.events.remove(stream.stream_uuid());

        Ok(())
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
        if !self.streams.contains_key(stream_uuid) {
            trace!("Stream {} does not exists", stream_uuid);
            return Err(AppendToStreamError::DoesntExists);
        }

        if let Some(e) = self.events.get_mut(stream_uuid) {
            let mut re: Vec<RecordedEvent> = events
                .iter()
                .map(|event| RecordedEvent {
                    event_number: 0,
                    event_uuid: Uuid::new_v4(),
                    stream_uuid: event.stream_uuid.clone(),
                    stream_version: event.stream_version,
                    causation_id: event.causation_id,
                    correlation_id: event.correlation_id,
                    event_type: event.event_type.clone(),
                    data: event.data.clone(),
                    metadata: String::new(),
                    created_at: String::new(),
                })
                .collect();

            let events_ids: Vec<Uuid> = re.iter().map(|event| event.event_uuid).collect();
            e.append(&mut re);
            debug!(
                "Successfully append {} event(s) to {}",
                events.len(),
                stream_uuid
            );

            Ok(events_ids)
        } else {
            Err(AppendToStreamError::DoesntExists)
        }
    }

    async fn read_stream_info(
        &mut self,
        stream_uuid: String,
    ) -> Result<&Stream, AppendToStreamError> {
        self.streams
            .get(&stream_uuid)
            .ok_or(AppendToStreamError::DoesntExists)
    }
}
