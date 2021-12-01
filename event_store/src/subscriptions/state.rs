use std::collections::{HashMap, HashSet, VecDeque};

use crate::{event::RecordedEvent, storage::Storage, EventStore};

use super::{subscriber::Subscriber, StartFrom};
use actix::prelude::*;

#[derive(Debug)]
pub struct SubscriptionState<S: Storage> {
    pub(crate) subscriber: Option<Subscriber>,
    pub(crate) storage: Addr<EventStore<S>>,
    pub(crate) stream_uuid: String,
    pub(crate) start_from: StartFrom,
    pub(crate) subscription_name: String,
    pub(crate) last_received: i64,
    pub(crate) last_sent: i64,
    pub(crate) last_ack: i64,
    pub(crate) queue: VecDeque<RecordedEvent>,
    pub(crate) transient: bool,
    pub(crate) in_flight_event_numbers: Vec<i64>,
}

impl<S: Storage> SubscriptionState<S> {
    pub(crate) fn reset_event_tracking(&mut self) {}
}
