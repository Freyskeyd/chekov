use crate::{
    core::stream::Stream, event::RecordedEvent, event::UnsavedEvent, EventStoreError,
    ExpectedVersion,
};
use actix::Message;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "Result<Stream, EventStoreError>")]
pub struct StreamInfo {
    pub correlation_id: Uuid,
    pub stream_uuid: String,
}

#[derive(Message)]
#[rtype(result = "Result<StreamForwardResult, EventStoreError>")]
pub struct StreamForward {
    pub correlation_id: Uuid,
    pub stream_uuid: String,
}

pub struct StreamForwardResult {
    pub stream: std::pin::Pin<
        Box<dyn futures::Stream<Item = Result<Vec<RecordedEvent>, EventStoreError>> + Send>,
    >,
}

#[derive(Message)]
#[rtype(result = "Result<Stream, EventStoreError>")]
pub struct CreateStream {
    pub(crate) correlation_id: Uuid,
    pub(crate) stream_uuid: String,
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Uuid>, EventStoreError>")]
pub struct Append {
    pub(crate) correlation_id: Uuid,
    pub(crate) stream: String,
    pub(crate) expected_version: ExpectedVersion,
    pub(crate) events: Vec<UnsavedEvent>,
}

#[derive(Message)]
#[rtype(result = "Result<Vec<RecordedEvent>, EventStoreError>")]
pub struct Read {
    pub(crate) correlation_id: Uuid,
    pub(crate) stream: String,
    pub(crate) version: usize,
    pub(crate) limit: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartPubSub {}
