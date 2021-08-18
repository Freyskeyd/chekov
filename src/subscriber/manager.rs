use super::{EventNotification, Listener, Subscribe};
use crate::application::InternalApplication;
use crate::{message::EventEnvelope, Application};
use actix::{ActorFutureExt, Addr, AsyncContext, Context, Recipient, WrapFuture};
use actix_interop::FutureInterop;
use fnv::FnvHasher;
use std::any::{Any, TypeId};
use std::{collections::HashMap, hash::BuildHasherDefault};
use tracing::{trace, Instrument};

type TypeMap<A> = HashMap<String, A, BuildHasherDefault<FnvHasher>>;

/// Deal with Application subscriptions
pub struct SubscriberManager<A: Application> {
    listener: Option<Addr<Listener<A>>>,
    _event_store: actix::Addr<event_store::EventStore<A::Storage>>,
    sub_map: TypeMap<Vec<(TypeId, Box<dyn Any>)>>,
}

impl<A: Application> std::default::Default for SubscriberManager<A> {
    fn default() -> Self {
        unimplemented!()
    }
}

impl<A: Application> SubscriberManager<A> {
    pub(crate) fn new(
        event_store: Addr<event_store::EventStore<A::Storage>>,
        _: HashMap<String, TypeId>,
    ) -> Self {
        Self {
            listener: None,
            _event_store: event_store,
            sub_map: TypeMap::default(),
        }
    }

    fn add_sub<M: crate::event::handler::BrokerMsg>(
        &mut self,
        stream: &str,
        sub: Recipient<M>,
        _id: &str,
    ) {
        let msg_id = TypeId::of::<M>();
        let stream_id = stream.to_string();
        let boxed = Box::new(sub);
        if let Some(subs) = self.sub_map.get_mut(&stream_id) {
            trace!("Broker: Adding to {:?} subscription list.", msg_id);
            subs.push((msg_id, boxed));
            return;
        }

        trace!("Broker: Creating {:?} subscription list.", msg_id);
        self.sub_map.insert(stream_id, vec![(msg_id, boxed)]);
    }
}

impl<A: Application> actix::registry::SystemService for SubscriberManager<A> {}
impl<A: Application> actix::Supervised for SubscriberManager<A> {}

impl<A: Application> actix::Actor for SubscriberManager<A> {
    type Context = Context<Self>;

    #[tracing::instrument(name = "SubscriberManager", skip(self, ctx), fields(app = %A::get_name()))]
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.wait(
            async {
                trace!("Create a new SubscriberManager");
                Listener::setup().await.unwrap()
            }
            .instrument(tracing::Span::current())
            .into_actor(self)
            .map(|res, actor, _| {
                actor.listener = Some(res);
            }),
        );
    }
}

impl<A: Application, M: crate::event::handler::BrokerMsg + std::fmt::Debug>
    actix::Handler<crate::event::handler::Subscribe<M>> for SubscriberManager<A>
{
    type Result = ();
    fn handle(
        &mut self,
        subscription: crate::event::handler::Subscribe<M>,
        _ctx: &mut Self::Context,
    ) {
        self.add_sub(&subscription.0, subscription.1, &subscription.2);
    }
}
impl<A, T> actix::Handler<EventEnvelope<T>> for SubscriberManager<A>
where
    A: Application,
    T: crate::event::Event + Clone + 'static,
{
    type Result = ();

    #[tracing::instrument(name = "SubscriberManager", skip(self, msg, _ctx), fields(app = %A::get_name()))]
    fn handle(&mut self, msg: EventEnvelope<T>, _ctx: &mut Self::Context) -> Self::Result {
        trace!("{:?}", msg.meta);
        if let Some(subs) = self.sub_map.get_mut(&msg.meta.stream_name) {
            subs.iter()
                .filter_map(|(id, addr)| {
                    if let Some(rec) = addr.downcast_ref::<Recipient<EventEnvelope<T>>>() {
                        Some((id, rec))
                    } else {
                        None
                    }
                })
                .for_each(|(_, addr)| {
                    let _ = addr.do_send(msg.clone());
                });
        }
    }
}
impl<A> actix::Handler<EventNotification> for SubscriberManager<A>
where
    A: Application,
{
    type Result = ();

    #[tracing::instrument(name = "SubscriberManager", skip(self, subscription, ctx), fields(app = %A::get_name()))]
    fn handle(&mut self, subscription: EventNotification, ctx: &mut Self::Context) {
        trace!(
            "Fetching Related RecordedEvents for {}",
            subscription.stream_uuid
        );
        let _ = ctx.address();
        let fut = async move {
            let result = match crate::event_store::EventStore::<A>::with_reader(
                event_store::read()
                    .stream(&subscription.stream_uuid)
                    .unwrap()
                    .from(event_store::prelude::ReadVersion::Version(
                        subscription.first_stream_version as i64,
                    ))
                    .limit(
                        (subscription.last_stream_version - subscription.first_stream_version + 1)
                            as usize,
                    ),
            )
            // TODO deal with mailbox error
            .await
            .unwrap()
            {
                Ok(events) => events
                    .into_iter()
                    .map(|mut event| {
                        // TODO big hack to have a working $all stream
                        event.stream_uuid = subscription.stream_uuid.to_string();
                        event
                    })
                    .collect(),
                Err(_) => panic!(""),
            };

            InternalApplication::<A>::resolve_many(result).await;
            // Call eventmapper with event name -> return event Real type,
            // try to build a Recipient<Type>
            // for event in result {
            // A::EventResolver::resolve(addr.clone(), &event.event_type.clone(), event);
            // }
        }
        .in_current_span()
        .interop_actor(self)
        .map(|_res, _actor, _ctx| {});

        ctx.spawn(fut);
    }
}

impl<A: Application> actix::Handler<Subscribe> for SubscriberManager<A> {
    type Result = ();
    fn handle(&mut self, subscription: Subscribe, _ctx: &mut Self::Context) {
        trace!(
            "Registering {} as subscriber on {}",
            subscription.subscriber_id,
            "$all"
        );
    }
}
