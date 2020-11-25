use super::subscriber::Subscriber;
use crate::prelude::*;
use actix::prelude::*;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct SubscriptionState {
    pub(crate) subscribers: HashMap<Recipient<RecordedEvents>, Addr<Subscriber>>,
    // :serializer,
    // :schema,
    stream_uuid: String,
    start_from: String,
    subscription_name: String,
    subscription_id: i64,
    // :selector,
    // :mapper,
    max_size: usize,
    partition_by: String,
    lock_ref: String,
    last_received: usize,
    last_sent: usize,
    last_ack: usize,
    queue_size: usize,
    buffer_size: usize,
    // partitions: %{},
    // processed_event_numbers: MapSet.new(),
    transient: bool,
}
