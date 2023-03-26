use std::{collections::VecDeque, sync::Arc};

use crate::{event::RecordedEvent, EventStore};

use super::{subscriber::Subscriber, StartFrom};
use actix::prelude::*;
use event_store_core::storage::Storage;

#[derive(Debug)]
pub struct SubscriptionState<S: Storage> {
    pub(crate) subscriber: Option<Subscriber>,
    pub(crate) storage: Addr<EventStore<S>>,
    pub(crate) stream_uuid: String,
    pub(crate) start_from: StartFrom,
    pub(crate) subscription_name: String,
    pub(crate) last_received: u64,
    pub(crate) last_sent: u64,
    pub(crate) last_ack: u64,
    pub(crate) queue: VecDeque<Arc<RecordedEvent>>,
    pub(crate) transient: bool,
    pub(crate) in_flight_event_numbers: Vec<u64>,
}

impl<S: Storage> SubscriptionState<S> {
    #[allow(clippy::unused_self)]
    pub(crate) fn reset_event_tracking(&mut self) {}
}
