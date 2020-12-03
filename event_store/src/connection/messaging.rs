use crate::{
    event::RecordedEvent, event::UnsavedEvent, stream::Stream, EventStoreError, ExpectedVersion,
};
use actix::Message;
use std::borrow::Cow;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "Result<Cow<'static, Stream>, EventStoreError>")]
pub struct StreamInfo {
    pub correlation_id: Uuid,
    pub stream_uuid: String,
}

#[derive(Message)]
#[rtype(result = "Result<Cow<'static, Stream>, EventStoreError>")]
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
