use crate::event::ParseEventError;
use crate::event::UnsavedEvent;
use crate::Event;
use crate::EventStore;
use crate::EventStoreError;
use crate::ExpectedVersion;
use crate::Storage;
use log::trace;
use uuid::Uuid;

/// An appender defines the parameter of an append action
///
/// It defines:
///     - which stream will receive the events
///     - which version of this stream is expected
///     - which events must be append
///
pub struct Appender {
    id: Uuid,
    expected_version: ExpectedVersion,
    events: Vec<UnsavedEvent>,
    stream: String,
}

impl Default for Appender {
    fn default() -> Self {
        let appender = Self {
            id: Uuid::new_v4(),
            expected_version: ExpectedVersion::AnyVersion,
            events: Vec::new(),
            stream: String::new(),
        };

        trace!(
            "Appender[{}]: Created with default configuration",
            appender.id
        );

        appender
    }
}

impl Appender {
    pub fn events<E: Event>(mut self, events: &[&E]) -> Result<Self, EventStoreError> {
        trace!(
            "Appender[{}]: Attemting to add {} event(s)",
            self.id,
            events.len()
        );
        let events: Vec<Result<UnsavedEvent, ParseEventError>> = events
            .iter()
            .map(|event| UnsavedEvent::try_from(*event))
            .collect();

        if events.iter().any(Result::is_err) {
            trace!(
                "Appender[{}]: Failed to add {} event(s)",
                self.id,
                events.len(),
            );
            return Err(EventStoreError::Any);
        }

        let mut events: Vec<UnsavedEvent> = events.into_iter().map(Result::unwrap).collect();
        trace!("Appender[{}]: Added {} event(s)", self.id, events.len());
        self.events.append(&mut events);

        Ok(self)
    }

    pub fn event<E: Event>(mut self, event: &E) -> Result<Self, EventStoreError> {
        trace!("Appender[{}]: Attemting to add 1 event", self.id);
        self.events.push(UnsavedEvent::try_from(event)?);

        trace!("Appender[{}]: Added 1 event", self.id);
        Ok(self)
    }

    pub fn to<S: Into<String>>(mut self, stream_id: S) -> Self {
        self.stream = stream_id.into();

        trace!(
            "Appender[{}]: Defined stream {} as target",
            self.id,
            self.stream,
        );
        self
    }

    pub fn expected_version(mut self, version: ExpectedVersion) -> Self {
        self.expected_version = version;
        trace!("Appender[{}]: Defined {:?}", self.id, self.expected_version,);
        self
    }

    pub async fn execute<S: Storage>(
        self,
        event_store: &EventStore<S>,
    ) -> Result<Vec<Uuid>, EventStoreError> {
        trace!("Appender[{}]: Attempting to execute", self.id);
        // Fetch stream informations
        let stream = event_store.stream_info(&self.stream).await;

        let stream = if stream.is_err() && self.expected_version == ExpectedVersion::AnyVersion {
            trace!(
                "Appender[{}]: Stream {} does not exists",
                self.id,
                self.stream
            );
            event_store.create_stream(&self.stream).await?
        } else if stream.is_err() {
            trace!(
                "Appender[{}]: Stream {} does not exists",
                self.id,
                self.stream
            );

            return Err(EventStoreError::Any);
        } else {
            trace!(
                "Appender[{}]: Fetched stream {} informations",
                self.id,
                self.stream,
            );
            stream.unwrap()
        };

        let events: Vec<UnsavedEvent> = self
            .events
            .into_iter()
            .enumerate()
            .map(|(index, mut event)| {
                event.stream_version = stream.stream_version + (index + 1) as i32;
                event.stream_uuid = stream.stream_uuid.clone();
                event
            })
            .collect();

        trace!(
            "Appender[{}]: Transformed {} event(s) into appendable events",
            self.id,
            events.len(),
        );

        let res = event_store
            .append_to_stream(&self.stream, self.expected_version, events)
            .await;

        trace!("Appender[{}]: Successfully executed", self.id);

        res
    }
}
