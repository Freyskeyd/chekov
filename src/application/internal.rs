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
    // pub async fn resolve_many_events(events: Vec<RecordedEvent>) -> Vec<Box<dyn ErasedGeneric>> {
    //     Self::from_registry()
    //         .send(ResolveEvents(events))
    //         .await
    //         .unwrap()
    // }
}

#[derive(Message)]
#[rtype("()")]
struct Resolve(RecordedEvent);

// #[derive(Message)]
// #[rtype("Vec<Box<dyn ErasedGeneric>>")]
// struct ResolveEvents(Vec<RecordedEvent>);

// impl<A> actix::Handler<ResolveEvents> for InternalApplication<A>
// where
//     A: Application,
// {
//     type Result = Vec<Box<dyn ErasedGeneric>>;

//     fn handle(&mut self, msg: ResolveEvents, _ctx: &mut Self::Context) -> Self::Result {
//         msg.0
//             .into_iter()
//             .filter_map(|event| {
//                 self.event_resolver
//                     .resolve_event(&event.event_type, event.clone())
//             })
//             .collect()
//         // self.event_resolver.resolve_even(
//         //     SubscriberManager::<A>::from_registry(),
//         //     &msg.0.event_type.clone(),
//         //     msg.0,
//         // )
//     }
// }

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
