use super::EventNotification;
use actix::{Actor, Context, Handler};

pub struct Subscriber {
    _stream: String,
}

impl Actor for Subscriber {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {}
}

impl Handler<EventNotification> for Subscriber {
    type Result = ();

    fn handle(&mut self, _: EventNotification, _ctx: &mut Self::Context) -> Self::Result {}
}
