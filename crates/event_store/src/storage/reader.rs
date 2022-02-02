use crate::core::event::RecordedEvent;
use crate::core::stream::Stream;
use crate::EventStore;
use crate::EventStoreError;
use crate::ReadVersion;
use event_store_core::storage::Storage;
use tracing::trace;
use uuid::Uuid;

use actix::prelude::*;

pub struct Reader {
    correlation_id: Uuid,
    span: tracing::Span,
    read_version: ReadVersion,
    stream: String,
    limit: usize,
}

impl Default for Reader {
    fn default() -> Self {
        Self::with_correlation_id(Uuid::new_v4())
    }
}

impl Reader {
    #[tracing::instrument(name = "Reader")]
    pub fn with_correlation_id(correlation_id: Uuid) -> Self {
        let reader = Self {
            correlation_id,
            span: tracing::Span::current(),
            read_version: ReadVersion::Origin,
            stream: String::new(),
            limit: 1_000,
        };

        trace!(
            parent: &reader.span,
            "Created");

        reader
    }
    /// Define which stream we are reading from
    ///
    /// # Errors
    ///
    /// Can fail if the stream doesn't have the expected format
    pub fn stream<S: ToString>(mut self, stream: S) -> Result<Self, EventStoreError> {
        // TODO: validate stream name format
        self.stream = stream.to_string();

        trace!(
            parent: &self.span,
            "Defined stream {} as target",
            self.stream,
        );

        Ok(self)
    }

    #[must_use]
    pub fn from(mut self, version: ReadVersion) -> Self {
        self.read_version = version;
        trace!(
            parent: &self.span,
            "Defined {:?}", self.read_version
        );
        self
    }

    #[must_use]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        trace!(
            parent: &self.span,
            "Defined {:?} limit", self.limit);

        self
    }

    pub async fn execute<S: Storage>(
        self,
        event_store: Addr<EventStore<S>>,
    ) -> Result<Vec<RecordedEvent>, EventStoreError> {
        trace!(
            parent: &self.span,
            "Attempting to execute");

        if !Stream::validates_stream_id(&self.stream) {
            return Err(EventStoreError::InvalidStreamId);
        }

        event_store
            .send(ReadStreamRequest {
                correlation_id: self.correlation_id,
                span: tracing::span!(parent: &self.span, tracing::Level::TRACE, "ReadStreamRequest", correlation_id = ?self.correlation_id),
                stream: self.stream,
                version: match self.read_version {
                    ReadVersion::Origin => 0,
                    // TODO: Should we switch to usize for version ?
                    ReadVersion::Version(version) => version as usize
                },
                limit: self.limit,
            })
            .await?
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<Vec<RecordedEvent>, EventStoreError>")]
pub struct ReadStreamRequest {
    pub correlation_id: Uuid,
    pub span: tracing::Span,
    pub stream: String,
    pub version: usize,
    pub limit: usize,
}
