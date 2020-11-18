use crate::prelude::RecordedEvents;
use actix::prelude::*;

#[derive(Debug)]
pub struct Subscriber {
    pub recipient: Recipient<RecordedEvents>,
}

impl Actor for Subscriber {
    type Context = Context<Self>;
}
