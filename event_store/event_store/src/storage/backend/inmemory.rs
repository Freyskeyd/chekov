// TODO: Remove this when https://github.com/rust-lang/rust/issues/88104 is resolved
#![allow(unused_braces)]

use crate::core::event::RecordedEvent;
use crate::core::event::UnsavedEvent;
use crate::core::stream::Stream;
use chrono::Utc;
use event_store_core::event_bus::EventBusMessage;
use event_store_core::storage::Backend;
use event_store_core::storage::StorageError;
use futures::Future;
use futures::FutureExt;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, trace};
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct InMemoryBackend {
    stream_counter: usize,
    streams: HashMap<String, Stream>,
    events: HashMap<String, Vec<RecordedEvent>>,
    pub(crate) notifier: Option<mpsc::UnboundedSender<EventBusMessage>>,
}

impl InMemoryBackend {
    pub async fn initiate(
        notifier: Option<mpsc::UnboundedSender<EventBusMessage>>,
    ) -> Result<Self, ()> {
        Ok(Self {
            stream_counter: 0,
            streams: HashMap::new(),
            events: HashMap::new(),
            notifier,
        })
    }
}

impl Backend for InMemoryBackend {
    fn backend_name() -> &'static str {
        "InMemoryBackend"
    }

    #[tracing::instrument(name = "InMemoryBackend::CreateStream", skip(self, stream))]
    fn create_stream(
        &mut self,
        mut stream: Stream,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Stream, StorageError>> + Send>> {
        trace!("Attempting to create stream {}", stream.stream_uuid());

        let stream_uuid = stream.stream_uuid().to_owned();

        if self.streams.contains_key(&stream_uuid) {
            return Box::pin(async move { Err(StorageError::StreamAlreadyExists) });
        }

        self.stream_counter += 1;
        stream.stream_id = self.stream_counter as i64;

        self.streams.insert(stream_uuid.clone(), stream);
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
        stream_uuid: String,
        version: usize,
        limit: usize,
        correlation_id: Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<RecordedEvent>, StorageError>> + Send>>
    {
        let version = if version == 0 { version } else { version - 1 };

        let result = self
            .events
            .get(&stream_uuid)
            .cloned()
            .map(|stream| stream.into_iter().skip(version).take(limit).collect())
            .ok_or(StorageError::StreamDoesntExists);

        async move { result }.boxed()
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
                .enumerate()
                .map(|(i, event)| RecordedEvent {
                    event_number: (e.len() + 1 + i) as i64,
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

            if let Some(s) = self.streams.get_mut(stream_uuid) {
                s.stream_version += re.len() as i64;
            } else {
                return Box::pin(async move { Err(StorageError::StreamDoesntExists) });
            }

            let events = re.clone();
            e.append(&mut re);

            debug!(
                "Successfully append {} event(s) to {}",
                events.len(),
                stream_uuid
            );

            let notifier = self.notifier.clone();

            Box::pin(async move {
                let ids = events.iter().map(|e| e.event_uuid).collect();
                if let Some(n) = notifier {
                    let _ = n.send(EventBusMessage::Events(events));
                }

                Ok(ids)
            })
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
        assert_eq!("InMemoryBackend", InMemoryBackend::backend_name());
    }

    #[test]
    fn can_be_instantiate() {
        let _storage = InMemoryBackend::default();
    }
}