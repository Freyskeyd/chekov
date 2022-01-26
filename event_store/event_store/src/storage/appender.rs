use crate::core::event::ParseEventError;
use crate::core::event::UnsavedEvent;
use crate::core::stream::Stream;
use crate::versions::ExpectedVersionResult;
use crate::Event;
use crate::EventStore;
use crate::EventStoreError;
use crate::ExpectedVersion;
use actix::prelude::*;
use event_store_core::storage::Storage;
use event_store_core::storage::StorageError;
use std::str::FromStr;
use tracing::trace;
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
/// # use actix::prelude::*;
/// use event_store::prelude::*;
/// # #[derive(serde::Deserialize, Serialize)]
/// # struct MyEvent {}
/// # impl Event for MyEvent {
/// #   fn event_type(&self) -> &'static str {
/// #      "MyEvent"
/// #   }
/// #   fn all_event_types() -> Vec<&'static str> {
/// #      vec!["MyEvent"]
/// #   }
/// # }
/// #
/// # impl std::convert::TryFrom<event_store::prelude::RecordedEvent> for MyEvent {
/// #   type Error = ();
/// #   fn try_from(e: event_store::prelude::RecordedEvent) -> Result<Self, Self::Error> {
/// #      serde_json::from_value(e.data).map_err(|_| ())
/// #   }
/// # }
/// #
/// # #[actix::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let es = EventStore::builder()
/// #   .storage(InMemoryStorage::default())
/// #   .build()
/// #   .await
/// #   .unwrap()
/// #   .start();
/// let my_event = MyEvent{};
/// let stream_name = "account-1";
///
/// event_store::append()
///     // Add an event to the append list
///     .event(&my_event)?
///     // Define which stream will be appended
///     .to(&stream_name)?
///     // Define that we expect the stream to be in version 1
///     .expected_version(ExpectedVersion::Version(1))
///     // Execute this appender on an eventstore
///     .execute(es)
///     .await;
/// #   Ok(())
/// # }
/// ```
pub struct Appender {
    id: Uuid,
    correlation_id: Uuid,
    span: tracing::Span,
    expected_version: ExpectedVersion,
    events: Vec<UnsavedEvent>,
    stream: String,
}

impl Default for Appender {
    fn default() -> Self {
        Self::with_correlation_id(Uuid::new_v4())
    }
}

impl Appender {
    #[tracing::instrument(name = "Appender")]
    pub fn with_correlation_id(correlation_id: Uuid) -> Self {
        let appender = Self {
            id: Uuid::new_v4(),
            correlation_id,
            span: tracing::Span::current(),
            expected_version: ExpectedVersion::AnyVersion,
            events: Vec::new(),
            stream: String::new(),
        };
        trace!(
            parent: &appender.span,
            "Created");

        appender
    }
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
            parent: &self.span,
            "Attemting to add {} event(s)", events.len());
        let events: Vec<Result<UnsavedEvent, ParseEventError>> = events
            .iter()
            .map(|event| UnsavedEvent::try_from(*event))
            .collect();

        if events.iter().any(Result::is_err) {
            trace!(
            parent: &self.span,
                "Failed to add {} event(s)", events.len(),);
            return Err(EventStoreError::Any);
        }

        let mut events: Vec<UnsavedEvent> = events.into_iter().map(Result::unwrap).collect();
        trace!(
            parent: &self.span,
            "Added {} event(s)", events.len());
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
        trace!(
            parent: &self.span,
            "Attemting to add 1 event");
        self.events.push(UnsavedEvent::try_from(event)?);

        trace!(
            parent: &self.span,
            "Added 1 event");
        Ok(self)
    }

    /// Define which stream we are appending to
    ///
    /// # Errors
    ///
    /// Can fail if the stream doesn't have the expected format
    pub fn to<S: ToString>(mut self, stream: &S) -> Result<Self, EventStoreError> {
        // TODO: validate stream name format
        self.stream = stream.to_string();

        trace!(
            parent: &self.span,
            "Defined stream {} as target", self.stream,);
        Ok(self)
    }

    /// Define the expected version of the stream we are appending to
    #[must_use]
    pub fn expected_version(mut self, version: ExpectedVersion) -> Self {
        self.expected_version = version;
        trace!(
            parent: &self.span,
            "Defined {:?}", self.expected_version,);
        self
    }

    /// Execute the `Appender` against a `Storage` and returns the generated event ids
    ///
    /// # Errors
    ///
    /// The execution can fail in various cases such as `ExpectedVersionResult` failure
    pub async fn execute<S: Storage>(
        self,
        event_store: Addr<EventStore<S>>,
    ) -> Result<Vec<Uuid>, EventStoreError> {
        trace!(
            parent: &self.span,
            "Attempting to execute");
        // Fetch stream informations
        if !Stream::validates_stream_id(&self.stream) {
            return Err(EventStoreError::InvalidStreamId);
        }

        let stream = event_store
            .send(crate::connection::StreamInfo {
                correlation_id: self.correlation_id,
                stream_uuid: self.stream.to_string(),
            })
            .await?;

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
                    parent: &self.span,
                    "Stream {} does not exists", self.stream);

                event_store
                    .send(crate::connection::CreateStream {
                        correlation_id: self.correlation_id,
                        stream_uuid: self.stream.to_string(),
                    })
                    .await??
            }
            ExpectedVersionResult::Ok => {
                trace!(
                    parent: &self.span,
                    "Fetched stream {} informations", self.stream,);
                stream.unwrap()
            }
            _ => {
                trace!(
                    parent: &self.span,
                    "Stream {} does not exists", self.stream);

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
            parent: &self.span,
            "Transformed {} event(s) into appendable events",
            events.len(),
        );

        let res = event_store
            .send(AppendToStreamRequest {
                correlation_id: self.correlation_id,
                stream: self.stream.to_string(),
                expected_version: self.expected_version,
                events,
            })
            .await?;

        trace!(parent: &self.span, "{}", match res {
            Ok(_) => "Successfully executed",
            Err(_) => "Unsuccessfully executed",
        });

        res
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<Vec<Uuid>, EventStoreError>")]
pub struct AppendToStreamRequest {
    pub correlation_id: Uuid,
    pub stream: String,
    pub expected_version: ExpectedVersion,
    pub events: Vec<UnsavedEvent>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::storage::InMemoryStorage;
    use uuid::Uuid;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct MyEvent {}
    impl Event for MyEvent {
        fn event_type(&self) -> &'static str {
            "MyEvent"
        }

        fn all_event_types() -> Vec<&'static str> {
            vec!["MyEvent"]
        }
    }

    impl std::convert::TryFrom<crate::prelude::RecordedEvent> for MyEvent {
        type Error = ();
        fn try_from(e: crate::prelude::RecordedEvent) -> Result<Self, Self::Error> {
            serde_json::from_value(e.data).map_err(|_| ())
        }
    }

    #[test]
    fn that_an_appender_can_be_configured() {
        let mut appender = Appender::default();

        appender = appender.to(&"stream_name").unwrap();
        assert_eq!(appender.stream, "stream_name");

        appender = appender.event(&MyEvent {}).unwrap();

        assert_eq!(appender.events.len(), 1);

        appender = appender.events(&[&MyEvent {}]).unwrap();

        assert_eq!(appender.events.len(), 2);

        appender = appender.expected_version(ExpectedVersion::AnyVersion);

        assert_eq!(appender.expected_version, ExpectedVersion::AnyVersion);
    }

    #[actix::test]
    async fn that_an_appender_can_be_executed() -> Result<(), EventStoreError> {
        let es = EventStore::builder()
            .storage(InMemoryStorage::default())
            .build()
            .await
            .unwrap();

        let addr = es.start();

        let uuid = Uuid::new_v4();

        let event = MyEvent {};

        let res = Appender::default()
            .to(&uuid)?
            .event(&event)?
            .expected_version(ExpectedVersion::StreamExists)
            .execute(addr.clone())
            .await;

        assert_eq!(res, Err(EventStoreError::Any));

        let res = Appender::default()
            .to(&uuid)?
            .event(&event)?
            .execute(addr)
            .await;

        assert!(res.is_ok());
        Ok(())
    }
}
