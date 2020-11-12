use crate::message::EventEnvelope;
use actix::prelude::{ActorFuture, RecipientRequest, WrapFuture};
use std::default::Default;

#[derive(Default, Debug)]
pub struct Registry {
    pub(crate) registry: std::collections::HashMap<String, Vec<actix::Recipient<EventEnvelope>>>,
}

impl actix::registry::ArbiterService for Registry {}
impl actix::Supervised for Registry {}
impl actix::Actor for Registry {
    type Context = actix::Context<Self>;
}

impl actix::Handler<EventEnvelope> for Registry {
    type Result = actix::ResponseActFuture<Self, Result<usize, ()>>;

    fn handle(&mut self, msg: EventEnvelope, _: &mut actix::Context<Self>) -> Self::Result {
        let v: Vec<actix::Recipient<EventEnvelope>> = Vec::new();
        let futs = self
            .registry
            .get(&msg.0.stream_uuid)
            .get_or_insert_with(|| &v)
            .iter()
            .map(|r| r.send(msg.clone()))
            .collect::<Vec<RecipientRequest<EventEnvelope>>>();

        Box::pin(
            async { futures::future::join_all(futs).await }
                .into_actor(self)
                .map(|res, _, _| {
                    let res = res
                        .into_iter()
                        .map(Result::unwrap)
                        .map(Result::unwrap)
                        .sum::<usize>();

                    Ok(res)
                }),
        )
    }
}
