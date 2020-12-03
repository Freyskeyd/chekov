use crate::{Application, EventResolver, SubscriberManager};
use actix::prelude::*;
use event_store::prelude::RecordedEvent;

pub(crate) struct InternalApplication<A: Application> {
    pub event_resolver: Box<dyn EventResolver<A>>,
}

impl<A> std::default::Default for InternalApplication<A>
where
    A: Application,
{
    fn default() -> Self {
        unimplemented!()
    }
}

impl<A> actix::Actor for InternalApplication<A>
where
    A: Application,
{
    type Context = Context<Self>;
}

impl<A> actix::SystemService for InternalApplication<A> where A: Application {}
impl<A> actix::Supervised for InternalApplication<A> where A: Application {}

impl<A> InternalApplication<A>
where
    A: Application,
{
    pub async fn resolve_many(events: Vec<RecordedEvent>) {
        for event in events.into_iter() {
            let _ = Self::from_registry().send(Resolve(event)).await;
        }
    }
}

#[derive(Message)]
#[rtype("()")]
struct Resolve(RecordedEvent);

impl<A> actix::Handler<Resolve> for InternalApplication<A>
where
    A: Application,
{
    type Result = ();

    fn handle(&mut self, msg: Resolve, _ctx: &mut Self::Context) -> Self::Result {
        self.event_resolver.resolve(
            SubscriberManager::<A>::from_registry(),
            &msg.0.event_type.clone(),
            msg.0,
        )
    }
}
