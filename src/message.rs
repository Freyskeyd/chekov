use super::Application;
use super::Command;
use super::CommandExecutorError;
use crate::command::CommandMetadatas;
use crate::Event;
use actix::prelude::*;
use event_store::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<Vec<C::Event>, CommandExecutorError>")]
pub struct Dispatch<C: Command, A: Application> {
    pub metadatas: CommandMetadatas,
    pub storage: std::marker::PhantomData<A>,
    // pub to: actix::Recipient<Dispatch<T, S>>,
    pub command: C,
}

#[derive(Debug, Clone)]
pub struct EventMetadatas {
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub stream_name: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Message)]
#[rtype(result = "Result(), ()>")]
pub struct EventEnvelope<E: Event> {
    pub event: E,
    pub meta: EventMetadatas,
}

#[doc(hidden)]
#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<(), ()>")]
pub struct ResolveAndApply(pub RecordedEvent);

#[doc(hidden)]
#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<(), ()>")]
pub struct ResolveAndApplyMany(pub Vec<RecordedEvent>);

#[derive(Message)]
#[rtype("Result<Vec<RecordedEvent>, event_store::prelude::EventStoreError>")]
pub(crate) struct ExecuteReader(pub(crate) event_store::prelude::Reader);

#[derive(Message)]
#[rtype("Result<Vec<Uuid>, event_store::prelude::EventStoreError>")]
pub(crate) struct ExecuteAppender(pub(crate) event_store::prelude::Appender);

#[derive(Message)]
#[rtype("Result<event_store::prelude::Stream, event_store::prelude::EventStoreError>")]
pub(crate) struct ExecuteStreamInfo(pub(crate) String);

// #[derive(Message)]
// #[rtype("Result<(), ()>")]
// pub struct Handle {
//     event: RecordedEvent,
// }
