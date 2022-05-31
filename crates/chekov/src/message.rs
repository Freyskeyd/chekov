use std::marker::PhantomData;

use super::Application;
use super::Command;
use super::CommandExecutorError;
use crate::command::CommandMetadatas;
use crate::Aggregate;
use crate::Event;
use actix::prelude::*;
use event_store::core::storage::Storage;
use event_store::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<Vec<C::Event>, CommandExecutorError>")]
pub struct Dispatch<C: Command, A: Application> {
    pub metadatas: CommandMetadatas,
    pub storage: std::marker::PhantomData<A>,
    pub command: C,
}

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<(Vec<C::Event>, S), CommandExecutorError>")]
pub struct DispatchWithState<S: Aggregate, C: Command, A: Application> {
    pub metadatas: CommandMetadatas,
    pub storage: std::marker::PhantomData<A>,
    pub command: C,
    pub state: S,
}

impl<S: Aggregate, C: Command, A: Application> DispatchWithState<S, C, A> {
    pub(crate) fn from_dispatch(dispatch: Dispatch<C, A>, state: S) -> Self {
        Self {
            metadatas: dispatch.metadatas,
            storage: dispatch.storage,
            command: dispatch.command,
            state,
        }
    }
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

#[derive(Message)]
#[rtype("i64")]
pub(crate) struct AggregateVersion;

#[derive(Message)]
#[rtype("A")]
pub(crate) struct AggregateState<A: Aggregate>(pub(crate) PhantomData<A>);

#[derive(Message, Debug)]
#[rtype("()")]
pub(crate) struct StartListening;

#[derive(Message)]
#[rtype("Addr<event_store::EventStore<S>>")]
pub(crate) struct GetAddr<S: Storage> {
    pub(crate) _phantom: PhantomData<S>,
}
