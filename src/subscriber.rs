use super::message;
use actix::prelude::*;
use log::trace;

pub struct Subscriber {
    _stream: String,
    _subscriber: Recipient<message::EventEnvelope>,
}

impl Subscriber {
    pub(crate) fn new(subscriber: Recipient<message::EventEnvelope>, stream: &str) -> Addr<Self> {
        trace!("Create a new Subscriber for {}", stream);
        Subscriber::create(move |_ctx| Subscriber {
            _stream: stream.to_string(),
            _subscriber: subscriber,
        })
    }
}

impl actix::Actor for Subscriber {
    type Context = Context<Self>;
}
