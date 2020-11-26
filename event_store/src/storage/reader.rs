use crate::stream::Stream;
use crate::EventStore;
use crate::EventStoreError;
use crate::ReadVersion;
use crate::RecordedEvent;
use crate::Storage;
use tracing::trace;
use uuid::Uuid;

use actix::prelude::*;
use futures::task::Poll;

pub struct Reader {
    correlation_id: Uuid,
    span: tracing::Span,
    read_version: ReadVersion,
    stream: String,
    limit: usize,
}

#[derive(Debug)]
pub struct StreamItem {
    events: Vec<RecordedEvent>,
}

struct StreamItemState<S: Storage> {
    version: usize,
    limit: usize,
    chunk: usize,
    stream: String,
    pending_size: usize,
    buffer: Vec<RecordedEvent>,
    _phantom: std::marker::PhantomData<S>,
}

use futures::Future;
use std::pin::Pin;

type ReadStreamState<S> =
    Option<Pin<Box<dyn Future<Output = (StreamItemState<S>, Option<StreamItem>)>>>>;

struct ReadStream<S: Storage> {
    aggregation_state: Option<Vec<RecordedEvent>>,
    item_state: ReadStreamState<S>,
}

impl<S> ReadStream<S>
where
    S: Storage,
{
    /// Computes a single item from a state. Returns the modified state and the item after completion.
    async fn compute_item(
        mut state: StreamItemState<S>,
    ) -> (StreamItemState<S>, Option<StreamItem>) {
        println!("compute version: {}, chunk: {}", state.version, state.chunk);

        match EventStore::<S>::from_registry()
            .send(ReadStreamRequest {
                correlation_id: Uuid::new_v4(),
                span: tracing::span!(tracing::Level::TRACE, "stream"),
                stream: state.stream.to_string(),
                version: state.version,
                limit: state.chunk as usize,
            })
            .await
        {
            Ok(Ok(events)) if !events.is_empty() => {
                // Update stream version target
                state.version += events.len();

                state.pending_size -= events.len();

                if state.pending_size < state.chunk {
                    state.chunk = state.pending_size;
                }

                (state, Some(StreamItem { events }))
            }
            _ => (state, None),
        }
    }

    /// Create a new instance of this amazing stream.
    pub fn new(stream: String, version: usize, limit: usize, chunk: usize) -> Self {
        let init_state = StreamItemState::<S> {
            version,
            limit,
            chunk,
            stream,
            pending_size: limit,
            buffer: vec![],
            _phantom: std::marker::PhantomData,
        };
        Self {
            aggregation_state: None,
            item_state: Some(Box::pin(Self::compute_item(init_state))),
        }
    }

    /// After all items have been processed this will return the aggregated data.
    pub fn aggregation(&self) -> Option<&Vec<RecordedEvent>> {
        self.aggregation_state.as_ref()
    }
}

impl<S> futures::Stream for ReadStream<S>
where
    S: Storage,
{
    type Item = Result<Option<StreamItem>, EventStoreError>;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context,
    ) -> core::task::Poll<Option<Self::Item>> {
        let inner_self = std::ops::DerefMut::deref_mut(&mut self);

        if let Some(fut) = inner_self.item_state.as_mut() {
            match Future::poll(fut.as_mut(), cx) {
                // return and keep waiting for result
                Poll::Pending => Poll::Pending,
                // item computation complete
                Poll::Ready((state, res)) if res.is_some() => {
                    inner_self.item_state = if state.pending_size > 0 {
                        Some(Box::pin(Self::compute_item(state)))
                    } else {
                        None
                    };

                    Poll::Ready(Some(Ok(res)))
                }
                Poll::Ready(..) => Poll::Ready(None),
            }
        } else {
            // no items left
            Poll::Ready(None)
        }
    }
}

pub struct StreamReader {
    id: Uuid,
    read_version: ReadVersion,
    stream: String,
    limit: usize,
    chunk_by: usize,
}

impl std::convert::From<Reader> for StreamReader {
    fn from(reader: Reader) -> Self {
        Self {
            id: reader.correlation_id,
            read_version: reader.read_version,
            stream: reader.stream,
            limit: reader.limit,
            chunk_by: reader.limit,
        }
    }
}

impl StreamReader {
    pub const fn chunk_by(mut self, chunk: usize) -> Self {
        self.chunk_by = chunk;

        self
    }

    pub async fn execute<S: Storage>(
        self,
    ) -> impl futures::Stream<Item = Result<Option<StreamItem>, EventStoreError>> {
        trace!("Reader[{}]: Attempting to execute", self.id);
        let s = self.stream.clone();
        ReadStream::<S>::new(s, 1, self.limit, self.chunk_by)
    }
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
    pub fn stream<S: Into<String>>(mut self, stream: S) -> Result<Self, EventStoreError> {
        // TODO: validate stream name format
        self.stream = stream.into();

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

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        trace!(
            parent: &self.span,
            "Defined {:?} limit", self.limit);

        self
    }

    pub fn into_stream(self) -> StreamReader {
        self.into()
    }

    pub async fn execute_async<S: Storage>(self) -> Result<Vec<RecordedEvent>, EventStoreError> {
        trace!(
            parent: &self.span,
            "Attempting to execute");

        if !Stream::validates_stream_id(&self.stream) {
            return Err(EventStoreError::Any);
        }

        EventStore::<S>::from_registry()
            .send(ReadStreamRequest {
                correlation_id: self.correlation_id.clone(),
                span: tracing::span!(parent: &self.span, tracing::Level::TRACE, "ReadStreamRequest", correlation_id = ?self.correlation_id),
                stream: self.stream,
                version: 0,
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
