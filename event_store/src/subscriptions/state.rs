use super::subscriber::Subscriber;
use actix::prelude::*;

#[derive(Default, Debug)]
pub struct SubscriptionState {
    // pub(crate) subscribers: HashMap<Recipient<SubscriptionNotification>, Addr<Subscriber>>,
    pub(crate) subscriber: Option<Addr<Subscriber>>,
    // :serializer,
    // :schema,
    pub(crate) stream_uuid: String,
    pub(crate) start_from: String,
    pub(crate) subscription_name: String,
    pub(crate) subscription_id: i64,
    // :selector,
    // :mapper,
    pub(crate) max_size: usize,
    pub(crate) partition_by: String,
    pub(crate) lock_ref: String,
    pub(crate) last_received: usize,
    pub(crate) last_sent: usize,
    pub(crate) last_ack: usize,
    pub(crate) queue_size: usize,
    pub(crate) buffer_size: usize,
    // partitions: %{},
    // processed_event_numbers: MapSet.new(),
    pub(crate) transient: bool,
}
