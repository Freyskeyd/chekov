use super::{EventNotification, Listener};
use crate::message::ResolveAndApplyMany;
use crate::Application;
use actix::{ActorFutureExt, Addr, AsyncContext, Context, Recipient, WrapFuture};
use event_store::prelude::RecordedEvent;
use std::any::TypeId;
use std::collections::BTreeMap;
use std::collections::HashMap;
use tracing::{trace, Instrument};

/// Deal with Application subscriptions
pub struct SubscriberManager<A: Application> {
    listener: Option<Addr<Listener<A>>>,
    _event_store: actix::Addr<event_store::EventStore<A::Storage>>,
    subscribers: BTreeMap<String, Vec<actix::Recipient<ResolveAndApplyMany>>>,
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
            subscribers: BTreeMap::new(),
        }
    }

    fn add_sub(&mut self, stream: &str, sub: Recipient<ResolveAndApplyMany>) {
        let stream_id = stream.to_string();
        if let Some(subs) = self.subscribers.get_mut(&stream_id) {
            trace!("Broker: Adding to subscription list.");
            subs.push(sub);
            return;
        }

        trace!("Broker: Creating subscription list.");
        self.subscribers.insert(stream_id, vec![sub]);
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

impl<A: Application> actix::Handler<crate::event::handler::Subscribe> for SubscriberManager<A> {
    type Result = ();
    fn handle(&mut self, subscription: crate::event::handler::Subscribe, _ctx: &mut Self::Context) {
        self.add_sub(&subscription.0, subscription.1);
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
        let stream_uuid = subscription.stream_uuid.clone();
        let fut = async move {
            match crate::event_store::EventStore::<A>::with_reader(
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
                    .collect::<Vec<RecordedEvent>>(),
                Err(_) => panic!(""),
            }
        }
        .in_current_span()
        .into_actor(self)
        .map(move |res, actor, _ctx| {
            if let Some(subs) = actor.subscribers.get(&stream_uuid) {
                for sub in subs {
                    let _ = sub.try_send(ResolveAndApplyMany(res.clone()));
                }
            }
        });

        ctx.spawn(fut);
    }
}
