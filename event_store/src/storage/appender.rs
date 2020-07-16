use crate::event::ParseEventError;
use crate::event::UnsavedEvent;
use crate::expected_version::ExpectedVersionResult;
use crate::storage::StorageError;
use crate::stream::Stream;
use crate::Event;
use crate::EventStore;
use crate::EventStoreError;
use crate::ExpectedVersion;
use crate::Storage;
use log::trace;
use std::str::FromStr;
use uuid::Uuid;

/// An Appender defines the parameters of an append action
///
/// An append action can succed or fail
///
///
/// # Examples
///
/// ```rust
/// # use serde::Serialize;
/// # use std::str::FromStr;
/// use event_store::prelude::*;
/// # #[derive(Serialize)]
/// # struct MyEvent {}
/// # impl Event for MyEvent {
/// #   fn event_type(&self) -> &'static str {
/// #      "MyEvent"
/// #   }
/// # }
/// # #[actix_rt::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let eventstore = EventStore::builder().storage(InMemoryBackend::default()).build().await.unwrap();
/// let my_event = MyEvent{};
/// let stream_name = "account-1";
///
/// event_store::append()
///     // Add an event to the append list
///     .event(&my_event)?
///     // Define which stream will be appended
///     .to(stream_name)?
///     // Define that we expect the stream to be in version 1
///     .expected_version(ExpectedVersion::Version(1))
///     // Execute this appender on an eventstore
///     .execute(&eventstore)
///     .await;
/// #   Ok(())
/// # }
/// ```
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
    /// Add a list of `Event`s to the `Appender`
    ///
    /// Any struct that implement `Event` can be passed to this method.
    /// Each event will be converted to an `UnsavedEvent` type and added to the pipe.
    ///
    /// The Event order are kept.Event
    ///
    /// # Errors
    ///
    /// This function can fail if one event can't be converted
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

    /// Add a single `Event`s to the `Appender`
    ///
    /// Any struct that implement `Event` can be passed to this method.
    /// This event will be converted to an `UnsavedEvent` type and added to the pipe.
    ///
    /// # Errors
    ///
    /// This function can fail if the event can't be converted
    pub fn event<E: Event>(mut self, event: &E) -> Result<Self, EventStoreError> {
        trace!("Appender[{}]: Attemting to add 1 event", self.id);
        self.events.push(UnsavedEvent::try_from(event)?);

        trace!("Appender[{}]: Added 1 event", self.id);
        Ok(self)
    }

    /// Define which stream we are appending to
    ///
    /// # Errors
    ///
    /// Can fail if the stream doesn't have the expected format
    pub fn to<S: Into<String>>(mut self, stream: S) -> Result<Self, EventStoreError> {
        // TODO: validate stream name format
        self.stream = stream.into();

        trace!(
            "Appender[{}]: Defined stream {} as target",
            self.id,
            self.stream,
        );
        Ok(self)
    }

    /// Define the expected version of the stream we are appending to
    #[must_use]
    pub fn expected_version(mut self, version: ExpectedVersion) -> Self {
        self.expected_version = version;
        trace!("Appender[{}]: Defined {:?}", self.id, self.expected_version,);
        self
    }

    /// Execute the `Appender` against a `Storage` and returns the generated event ids
    ///
    /// # Errors
    ///
    /// The execution can fail in various cases such as `ExpectedVersionResult` failure
    pub async fn execute<S: Storage>(
        self,
        event_store: &EventStore<S>,
    ) -> Result<Vec<Uuid>, EventStoreError> {
        trace!("Appender[{}]: Attempting to execute", self.id);
        // Fetch stream informations
        if !Stream::validates_stream_id(&self.stream) {
            return Err(EventStoreError::Any);
        }

        let stream = event_store.stream_info(&self.stream).await;

        let expected_version_result = match stream {
            Ok(ref stream) => ExpectedVersion::verify(stream, &self.expected_version),
            Err(EventStoreError::Storage(StorageError::StreamDoesntExists)) => {
                ExpectedVersion::verify(
                    &Stream::from_str(&self.stream).unwrap(),
                    &self.expected_version,
                )
            }
            Err(_) => ExpectedVersionResult::WrongExpectedVersion,
        };

        let stream = match expected_version_result {
            ExpectedVersionResult::NeedCreation => {
                trace!(
                    "Appender[{}]: Stream {} does not exists",
                    self.id,
                    self.stream
                );

                event_store.create_stream(&self.stream).await?
            }
            ExpectedVersionResult::Ok => {
                trace!(
                    "Appender[{}]: Fetched stream {} informations",
                    self.id,
                    self.stream,
                );
                stream.unwrap()
            }
            _ => {
                trace!(
                    "Appender[{}]: Stream {} does not exists",
                    self.id,
                    self.stream
                );

                return Err(EventStoreError::Any);
            }
        };

        let events: Vec<UnsavedEvent> = self
            .events
            .into_iter()
            .enumerate()
            .map(|(index, mut event)| {
                event.stream_version = stream.stream_version + (index + 1) as i64;
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
